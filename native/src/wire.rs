//! Sawbyte's bounded Java protocol framing and primitive codec.
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, generic_array::GenericArray};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{self, Read, Write};
use std::net::TcpStream;
pub const LIMIT: usize = 2 * 1024 * 1024;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub fn bad(text: &str) -> Box<dyn std::error::Error + Send + Sync> {
    text.to_owned().into()
}

pub struct Reader<'a>(pub &'a [u8]);
impl<'a> Reader<'a> {
    pub fn take(&mut self, count: usize) -> Result<&'a [u8]> {
        if count > self.0.len() {
            return Err(bad("truncated packet"));
        }
        let (head, tail) = self.0.split_at(count);
        self.0 = tail;
        Ok(head)
    }
    pub fn byte(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    pub fn var(&mut self) -> Result<i32> {
        Ok(sawbyte_protocol::read_varint(&mut self.0)?)
    }
    pub fn count(&mut self, max: usize) -> Result<usize> {
        let value = self.var()?;
        if value < 0 || value as usize > max {
            return Err(bad("length outside limit"));
        }
        Ok(value as usize)
    }
    pub fn string(&mut self) -> Result<String> {
        let n = self.count(32767 * 4)?;
        Ok(std::str::from_utf8(self.take(n)?)?.to_owned())
    }
    pub fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(self.take(4)?.try_into()?))
    }
    pub fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into()?))
    }
    pub fn f64(&mut self) -> Result<f64> {
        Ok(f64::from_bits(self.u64()?))
    }
    pub fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_bits(self.i32()? as u32))
    }
    pub fn blob(&mut self) -> Result<Vec<u8>> {
        let n = self.count(LIMIT)?;
        Ok(self.take(n)?.to_vec())
    }
    pub fn nbt(&mut self) -> Result<serde_json::Value> {
        let tag = self.byte()?;
        self.tag(tag, 0)
    }
    fn tag(&mut self, tag: u8, depth: usize) -> Result<serde_json::Value> {
        use serde_json::{Map, Value};
        if depth > 32 {
            return Err(bad("NBT depth limit"));
        }
        Ok(match tag {
            0 => Value::Null,
            1 => Value::from(self.byte()? as i8),
            2 => Value::from(i16::from_be_bytes(self.take(2)?.try_into()?)),
            3 => Value::from(self.i32()?),
            4 => Value::from(self.u64()? as i64),
            5 => Value::from(self.f32()?),
            6 => Value::from(self.f64()?),
            8 => {
                let n = u16::from_be_bytes(self.take(2)?.try_into()?) as usize;
                Value::String(String::from_utf8_lossy(self.take(n)?).into_owned())
            }
            7 | 11 | 12 => {
                let n = self.i32()?;
                if !(0..=262144).contains(&n) {
                    return Err(bad("NBT array limit"));
                }
                let count = n as usize;
                match tag {
                    7 => {
                        let raw = self.take(count)?;
                        Value::Array(raw.iter().map(|&b| Value::from(b as i8)).collect())
                    }
                    11 => {
                        let raw = self.take(count * 4)?;
                        Value::Array(
                                raw.chunks_exact(4)
                                    .map(|c| Value::from(i32::from_be_bytes([c[0], c[1], c[2], c[3]])))
                                    .collect(),
                            )
                    }
                    _ => {
                        let raw = self.take(count * 8)?;
                        Value::Array(
                                raw.chunks_exact(8)
                                    .map(|c| {
                                        Value::from(i64::from_be_bytes([
                                            c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7],
                                        ]))
                                    })
                                    .collect(),
                            )
                    }
                }
            }
            9 => {
                let kind = self.byte()?;
                let n = self.i32()?;
                if !(0..=16384).contains(&n) {
                    return Err(bad("NBT list limit"));
                }
                let mut items = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    let elem = self.tag(kind, depth + 1)?;
                    items.push(elem);
                }
                Value::Array(items)
            }
            10 => {
                let mut map = Map::new();
                loop {
                    let kind = self.byte()?;
                    if kind == 0 {
                        break;
                    }
                    if map.len() >= 4096 {
                        return Err(bad("NBT compound limit"));
                    }
                    let n = u16::from_be_bytes(self.take(2)?.try_into()?) as usize;
                    let key = String::from_utf8_lossy(self.take(n)?).into_owned();
                    map.insert(key, self.tag(kind, depth + 1)?);
                }
                Value::Object(map)
            }
            _ => return Err(bad("unknown NBT tag")),
        })
    }
}

pub fn var(value: i32, out: &mut Vec<u8>) {
    sawbyte_protocol::write_varint(value, out);
}
pub fn string(value: &str, out: &mut Vec<u8>) {
    var(value.len() as i32, out);
    out.extend(value.as_bytes());
}
pub fn blob(value: &[u8], out: &mut Vec<u8>) {
    var(value.len() as i32, out);
    out.extend(value);
}

pub struct Rx {
    pub stream: TcpStream,
    pub threshold: Option<usize>,
    cipher: Option<cfb8::Decryptor<aes::Aes128>>,
}
pub struct Tx {
    pub stream: TcpStream,
    pub threshold: Option<usize>,
    cipher: Option<cfb8::Encryptor<aes::Aes128>>,
}
pub fn connection(stream: TcpStream) -> io::Result<(Rx, Tx)> {
    Ok((
        Rx {
            stream: stream.try_clone()?,
            threshold: None,
            cipher: None,
        },
        Tx {
            stream,
            threshold: None,
            cipher: None,
        },
    ))
}
impl Rx {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            threshold: None,
            cipher: None,
        }
    }

    pub fn encrypt(&mut self, key: &[u8; 16]) {
        self.cipher = Some(cfb8::Decryptor::new(key.into(), key.into()));
    }
    fn exact(&mut self, bytes: &mut [u8]) -> Result<()> {
        self.stream.read_exact(bytes)?;
        if let Some(cipher) = &mut self.cipher {
            for byte in bytes {
                cipher.decrypt_block_mut(GenericArray::from_mut_slice(std::slice::from_mut(byte)));
            }
        }
        Ok(())
    }
    fn length(&mut self) -> Result<usize> {
        let mut bytes = [0u8; 5];
        for i in 0..5 {
            let mut byte = [0];
            self.exact(&mut byte)?;
            bytes[i] = byte[0];
            if byte[0] & 128 == 0 {
                return Reader(&bytes[..=i]).count(LIMIT);
            }
        }
        Err(bad("overlong packet length"))
    }
    pub fn packet(&mut self) -> Result<Vec<u8>> {
        let length = self.length()?;
        if length == 0 {
            return Err(bad("empty packet"));
        }
        let mut bytes = vec![0; length];
        self.exact(&mut bytes)?;
        if let Some(threshold) = self.threshold {
            let mut r = Reader(&bytes);
            let raw = r.count(LIMIT)?;
            if raw == 0 {
                if r.0.len() >= threshold {
                    return Err(bad("uncompressed packet exceeds threshold"));
                }
                return Ok(r.0.to_vec());
            }
            if raw < threshold {
                return Err(bad("compressed packet below threshold"));
            }
            let mut decoded = Vec::with_capacity(raw);
            ZlibDecoder::new(r.0)
                .take(raw as u64 + 1)
                .read_to_end(&mut decoded)?;
            if decoded.len() != raw {
                return Err(bad("decompressed length mismatch"));
            }
            Ok(decoded)
        } else {
            Ok(bytes)
        }
    }
}
impl Tx {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            threshold: None,
            cipher: None,
        }
    }

    pub fn encrypt(&mut self, key: &[u8; 16]) {
        self.cipher = Some(cfb8::Encryptor::new(key.into(), key.into()));
    }
    pub fn packet(&mut self, id: i32, payload: &[u8]) -> Result<()> {
        let mut body = Vec::with_capacity(payload.len() + 5);
        var(id, &mut body);
        body.extend(payload);
        if body.len() > LIMIT {
            return Err(bad("outgoing packet too large"));
        }
        let mut encoded = Vec::new();
        if let Some(threshold) = self.threshold {
            if body.len() >= threshold {
                var(body.len() as i32, &mut encoded);
                let mut z = ZlibEncoder::new(Vec::new(), Compression::fast());
                z.write_all(&body)?;
                encoded.extend(z.finish()?);
            } else {
                encoded.push(0);
                encoded.extend(body);
            }
        } else {
            encoded = body;
        }
        let mut frame = Vec::with_capacity(encoded.len() + 5);
        var(encoded.len() as i32, &mut frame);
        frame.extend(encoded);
        if let Some(cipher) = &mut self.cipher {
            for byte in &mut frame {
                cipher.encrypt_block_mut(GenericArray::from_mut_slice(std::slice::from_mut(byte)));
            }
        }
        self.stream.write_all(&frame)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nbt_large_arrays_and_lists_preserve_every_element() {
        let mut bytes = vec![7];
        bytes.extend_from_slice(&257i32.to_be_bytes());
        bytes.extend(std::iter::repeat_n(42, 257));
        let value = Reader(&bytes).nbt().unwrap();
        assert_eq!(value.as_array().unwrap().len(), 257);
        assert_eq!(value[256], 42);

        let mut list = vec![9, 1];
        list.extend_from_slice(&4097i32.to_be_bytes());
        list.extend(std::iter::repeat_n(7, 4097));
        let value = Reader(&list).nbt().unwrap();
        assert_eq!(value.as_array().unwrap().len(), 4097);
        assert_eq!(value[4096], 7);
    }
    use std::net::TcpListener;
    #[test]
    fn encrypted_compressed_packets_keep_stream_state() {
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = TcpStream::connect(server.local_addr().unwrap()).unwrap();
        let (peer, _) = server.accept().unwrap();
        let (_, mut tx) = connection(client).unwrap();
        let (mut rx, _) = connection(peer).unwrap();
        tx.encrypt(&[7; 16]);
        rx.encrypt(&[7; 16]);
        tx.threshold = Some(16);
        rx.threshold = Some(16);
        for payload in [vec![4; 3], vec![8; 300], vec![9; 2]] {
            tx.packet(42, &payload).unwrap();
            let decoded = rx.packet().unwrap();
            assert_eq!(decoded[0], 42);
            assert_eq!(&decoded[1..], &payload);
        }
    }
}
