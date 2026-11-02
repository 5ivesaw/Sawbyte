//! Bounded Minecraft Java status handshake. This is not a play/login client.

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

const MAX_FRAME: usize = 131_072;

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub fn write_varint(value: i32, out: &mut Vec<u8>) {
    let mut value = value as u32;
    loop {
        if value & !0x7f == 0 {
            out.push(value as u8);
            return;
        }
        out.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
}

pub fn read_varint(reader: &mut impl Read) -> io::Result<i32> {
    let mut value = 0_u32;
    for index in 0..5 {
        let mut byte = [0];
        reader.read_exact(&mut byte)?;
        if index == 4 && byte[0] & 0xf0 != 0 {
            return Err(invalid("VarInt overflows 32 bits"));
        }
        value |= u32::from(byte[0] & 0x7f) << (index * 7);
        if byte[0] & 0x80 == 0 {
            return Ok(value as i32);
        }
    }
    Err(invalid("VarInt is too long"))
}

fn read_frame(reader: &mut impl Read) -> io::Result<Vec<u8>> {
    let length = read_varint(reader)?;
    if length <= 0 || length as usize > MAX_FRAME {
        return Err(invalid("packet length outside limit"));
    }
    let mut packet = vec![0; length as usize];
    reader.read_exact(&mut packet)?;
    Ok(packet)
}

fn write_frame(writer: &mut impl Write, payload: &[u8]) -> io::Result<()> {
    if payload.len() > MAX_FRAME {
        return Err(invalid("outgoing packet too large"));
    }
    let mut prefix = Vec::with_capacity(5);
    write_varint(payload.len() as i32, &mut prefix);
    writer.write_all(&prefix)?;
    writer.write_all(payload)
}

fn status_text(packet: &[u8]) -> io::Result<String> {
    let mut bytes = packet;
    if read_varint(&mut bytes)? != 0 {
        return Err(invalid("expected status response"));
    }
    let length = read_varint(&mut bytes)?;
    if length < 0 || length as usize != bytes.len() {
        return Err(invalid("invalid status string length"));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| invalid("status is not UTF-8"))?;
    if text.encode_utf16().count() > 32767 {
        return Err(invalid("status string exceeds protocol limit"));
    }
    // Intentionally opaque: protocol framing does not imply valid JSON.
    Ok(text.to_owned())
}

struct DeadlineStream {
    stream: TcpStream,
    deadline: Instant,
}

impl DeadlineStream {
    fn remaining(&self) -> io::Result<Duration> {
        self.deadline
            .checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "status deadline exceeded"))
    }
}

impl Read for DeadlineStream {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.stream.set_read_timeout(Some(self.remaining()?))?;
        self.stream.read(bytes)
    }
}

impl Write for DeadlineStream {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.stream.set_write_timeout(Some(self.remaining()?))?;
        self.stream.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

#[derive(Debug)]
pub struct Status {
    /// Untrusted server-provided text, conventionally JSON. Not parsed here.
    pub document: String,
    pub round_trip: Duration,
}

/// Uses a numeric endpoint so DNS resolution cannot escape the request deadline.
/// `hostname` is the original virtual host sent in the handshake (no SRV lookup).
pub fn query_status(
    endpoint: SocketAddr,
    hostname: &str,
    protocol: i32,
    timeout: Duration,
) -> io::Result<Status> {
    if timeout.is_zero()
        || hostname.is_empty()
        || hostname.encode_utf16().count() > 255
        || hostname.chars().any(char::is_control)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid timeout or hostname",
        ));
    }
    let deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "timeout too large"))?;
    let stream = TcpStream::connect_timeout(&endpoint, timeout)?;
    stream.set_nodelay(true)?;
    let mut stream = DeadlineStream { stream, deadline };
    let mut handshake = Vec::with_capacity(hostname.len() + 16);
    write_varint(0, &mut handshake);
    write_varint(protocol, &mut handshake);
    write_varint(hostname.len() as i32, &mut handshake);
    handshake.extend_from_slice(hostname.as_bytes());
    handshake.extend_from_slice(&endpoint.port().to_be_bytes());
    write_varint(1, &mut handshake); // next state: status
    write_frame(&mut stream, &handshake)?;
    write_frame(&mut stream, &[0])?;
    let document = status_text(&read_frame(&mut stream)?)?;
    // Echo payload is a correlation value, not authentication.
    let mut ping = vec![1];
    ping.extend_from_slice(&0x5341574259544501_i64.to_be_bytes());
    let started = Instant::now();
    write_frame(&mut stream, &ping)?;
    if read_frame(&mut stream)? != ping {
        return Err(invalid("pong does not match ping"));
    }
    Ok(Status {
        document,
        round_trip: started.elapsed(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn varint_known_vectors() {
        for (value, expected) in [
            (0, vec![0]),
            (127, vec![127]),
            (128, vec![128, 1]),
            (25565, vec![221, 199, 1]),
            (-1, vec![255, 255, 255, 255, 15]),
            (i32::MIN, vec![128, 128, 128, 128, 8]),
        ] {
            let mut bytes = Vec::new();
            write_varint(value, &mut bytes);
            assert_eq!(bytes, expected);
            assert_eq!(read_varint(&mut expected.as_slice()).unwrap(), value);
        }
    }

    #[test]
    fn rejects_overflow_truncation_and_unbounded_allocation() {
        for bytes in [vec![128], vec![255; 6], vec![128, 128, 128, 128, 16]] {
            assert!(read_varint(&mut bytes.as_slice()).is_err());
        }
        for length in [-1, 0, MAX_FRAME as i32 + 1] {
            let mut bytes = Vec::new();
            write_varint(length, &mut bytes);
            assert!(read_frame(&mut bytes.as_slice()).is_err());
        }
        assert!(status_text(&[0, 1, 0xff]).is_err());
        assert!(status_text(&[0, 1, b'a', b'b']).is_err());
        assert!(status_text(&[1, 0]).is_err());
    }

    fn fixture(bad_pong: bool) -> (SocketAddr, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let handshake = read_frame(&mut stream).unwrap();
            let mut bytes = handshake.as_slice();
            assert_eq!(read_varint(&mut bytes).unwrap(), 0);
            assert_eq!(read_varint(&mut bytes).unwrap(), -1);
            let length = read_varint(&mut bytes).unwrap() as usize;
            assert_eq!(&bytes[..length], b"example.test");
            bytes = &bytes[length..];
            assert_eq!(&bytes[..2], &endpoint.port().to_be_bytes());
            assert_eq!(&bytes[2..], &[1]);
            assert_eq!(read_frame(&mut stream).unwrap(), [0]);
            // Deliberate fragmentation: no assumption that a read yields a packet.
            for byte in [4, 0, 2, b'{', b'}'] {
                stream.write_all(&[byte]).unwrap();
            }
            let mut ping = read_frame(&mut stream).unwrap();
            assert_eq!(ping.len(), 9);
            assert_eq!(ping[0], 1);
            if bad_pong {
                ping[8] ^= 1;
            }
            write_frame(&mut stream, &ping).unwrap();
        });
        (endpoint, handle)
    }

    #[test]
    fn status_handshake_over_real_tcp() {
        let (endpoint, server) = fixture(false);
        let result = query_status(endpoint, "example.test", -1, Duration::from_secs(5)).unwrap();
        assert_eq!(result.document, "{}");
        server.join().unwrap();
    }

    #[test]
    fn rejects_mismatched_pong() {
        let (endpoint, server) = fixture(true);
        assert!(query_status(endpoint, "example.test", -1, Duration::from_secs(5)).is_err());
        server.join().unwrap();
    }

    #[test]
    fn stalled_peer_has_a_deadline() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (_stream, _) = listener.accept().unwrap();
            thread::sleep(Duration::from_millis(300));
        });
        let error =
            query_status(endpoint, "example.test", -1, Duration::from_millis(50)).unwrap_err();
        assert!(matches!(
            error.kind(),
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
        ));
        server.join().unwrap();
    }
}
