use crate::wire::{Reader, Result, bad};
use std::collections::{HashMap, HashSet};
pub struct Palette {
    bits: usize,
    ids: Vec<u32>,
    words: Vec<u64>,
}
impl Palette {
    fn read(r: &mut Reader<'_>, entries: usize, indirect_max: usize) -> Result<Self> {
        let bits = r.byte()? as usize;
        if bits > 32 {
            return Err(bad("invalid palette width"));
        }
        let ids = if bits == 0 {
            vec![r.count(1_000_000)? as u32]
        } else if bits <= indirect_max {
            let n = r.count(1 << indirect_max)?;
            if n == 0 {
                return Err(bad("empty indirect palette"));
            }
            (0..n)
                .map(|_| r.count(1_000_000).map(|v| v as u32))
                .collect::<Result<Vec<_>>>()?
        } else {
            Vec::new()
        };
        let count = if bits == 0 {
            0
        } else {
            entries.div_ceil(64 / bits)
        };
        let words = (0..count).map(|_| r.u64()).collect::<Result<Vec<_>>>()?;
        Ok(Self { bits, ids, words })
    }
    fn get(&self, i: usize) -> Result<u32> {
        if self.bits == 0 {
            return Ok(self.ids[0]);
        }
        let per = 64 / self.bits;
        let value =
            ((self.words[i / per] >> ((i % per) * self.bits)) & ((1u64 << self.bits) - 1)) as usize;
        if self.ids.is_empty() {
            Ok(value as u32)
        } else {
            self.ids
                .get(value)
                .copied()
                .ok_or_else(|| bad("invalid palette index"))
        }
    }
}

pub struct World {
    pub min_y: i32,
    pub height: i32,
    pub chunks: HashMap<(i32, i32), Vec<Palette>>,
    air: HashSet<u32>,
    solid: HashSet<u32>,
}
impl World {
    pub fn new(profile: &serde_json::Value) -> Result<Self> {
        if profile["minecraft"] != "26.3" {
            return Err(bad("block profile must match Minecraft 26.3"));
        }
        let ids = |name: &str| -> Result<HashSet<u32>> {
            profile[name]
                .as_array()
                .ok_or_else(|| bad("invalid block profile"))?
                .iter()
                .map(|v| {
                    v.as_u64()
                        .filter(|n| *n < 1_000_000)
                        .map(|n| n as u32)
                        .ok_or_else(|| bad("invalid block ID"))
                })
                .collect()
        };
        let air = ids("air")?;
        let solid = ids("solid")?;
        if air.is_empty() || solid.is_empty() || !air.is_disjoint(&solid) {
            return Err(bad("block profile needs distinct air and solid states"));
        }
        Ok(Self {
            min_y: -64,
            height: 384,
            chunks: HashMap::new(),
            air,
            solid,
        })
    }
    pub fn chunk(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let x = r.i32()?;
        let z = r.i32()?;
        let maps = r.count(32)?;
        for _ in 0..maps {
            r.var()?;
            let n = r.count(1024)?;
            r.take(n * 8)?;
        }
        let data = r.blob()?;
        let mut sections = Reader(&data);
        let mut result = Vec::new();
        for _ in 0..self.height / 16 {
            sections.take(4)?; // non-air block and fluid counts
            result.push(Palette::read(&mut sections, 4096, 8)?);
            Palette::read(&mut sections, 64, 3)?;
        }
        if !sections.0.is_empty() {
            return Err(bad("unexpected chunk section data"));
        }
        if self.chunks.len() >= 256 && !self.chunks.contains_key(&(x, z)) {
            return Err(bad("chunk budget exceeded"));
        }
        self.chunks.insert((x, z), result);
        Ok(())
    }
    pub fn solid(&self, x: i32, y: i32, z: i32) -> Result<bool> {
        if y < self.min_y || y >= self.min_y + self.height {
            return Err(bad("outside known dimension"));
        }
        let chunk = self
            .chunks
            .get(&(x.div_euclid(16), z.div_euclid(16)))
            .ok_or_else(|| bad("collision data not loaded"))?;
        let index =
            ((y.rem_euclid(16) * 256) + (z.rem_euclid(16) * 16) + x.rem_euclid(16)) as usize;
        let id = chunk[((y - self.min_y) / 16) as usize].get(index)?;
        if self.air.contains(&id) {
            Ok(false)
        } else if self.solid.contains(&id) {
            Ok(true)
        } else {
            Err(bad(&format!(
                "unsupported collision state {id}; movement stopped"
            )))
        }
    }
    pub fn loaded(&self, x: f64, z: f64) -> bool {
        self.chunks.contains_key(&(
            (x.floor() as i32).div_euclid(16),
            (z.floor() as i32).div_euclid(16),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn palette_padding_and_invalid_indices() {
        // Five-bit values leave four unused bits in each 64-bit word.
        let mut data = vec![5, 2, 7, 9];
        data.extend((1u64 << 55).to_be_bytes());
        data.extend(1u64.to_be_bytes());
        let palette = Palette::read(&mut Reader(&data), 24, 8).unwrap();
        assert_eq!(palette.get(10).unwrap(), 7);
        assert_eq!(palette.get(11).unwrap(), 9);
        assert_eq!(palette.get(12).unwrap(), 9);
        assert_eq!(palette.get(13).unwrap(), 7);
        assert!(Palette::read(&mut Reader(&data[..data.len() - 1]), 24, 8).is_err());
        data[19] = 2;
        let invalid = Palette::read(&mut Reader(&data), 24, 8).unwrap();
        assert!(invalid.get(12).is_err());
    }

    #[test]
    fn collision_requires_known_state_and_loaded_chunk() {
        let mut world = World::new(&json!({"minecraft":"26.3","air":[0],"solid":[1]})).unwrap();
        world.min_y = -16;
        world.height = 16;
        world.chunks.insert(
            (-1, -1),
            vec![Palette {
                bits: 0,
                ids: vec![1],
                words: vec![],
            }],
        );
        assert!(world.solid(-1, -1, -1).unwrap());
        assert!(world.solid(0, -1, 0).is_err());
        assert!(world.solid(-1, -17, -1).is_err());
        world.chunks.get_mut(&(-1, -1)).unwrap()[0].ids[0] = 2;
        assert!(world.solid(-1, -1, -1).is_err());
        assert!(World::new(&json!({"minecraft":"26.3","air":[0],"solid":[0]})).is_err());
        assert!(World::new(&json!({"minecraft":"26.2","air":[0],"solid":[1]})).is_err());
    }
}
