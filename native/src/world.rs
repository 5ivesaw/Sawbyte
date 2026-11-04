#![allow(dead_code)]
use crate::wire::{Reader, Result, bad};
use sawbyte_core::tasks::VoxelQuery;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Palette {
    bits: usize,
    ids: Vec<u32>,
    words: Vec<u64>,
}

impl Palette {
    pub fn single(id: u32) -> Self {
        Self {
            bits: 0,
            ids: vec![id],
            words: Vec::new(),
        }
    }

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

    pub fn set(&mut self, i: usize, state_id: u32) -> Result<()> {
        if i >= 4096 {
            return Err(bad("block section index out of range"));
        }
        if state_id >= 1_000_000 {
            return Err(bad("block state ID out of range"));
        }
        if self.bits == 0 {
            if self.ids[0] == state_id {
                return Ok(());
            }
            // Promote 0-bit single-value section to a 4-bit indirect palette
            self.bits = 4;
            self.ids.push(state_id);
            let per = 64 / self.bits;
            self.words = vec![0u64; 4096usize.div_ceil(per)];
            let word_idx = i / per;
            let bit_offset = (i % per) * self.bits;
            self.words[word_idx] |= 1u64 << bit_offset;
            return Ok(());
        }
        let palette_index = if self.ids.is_empty() {
            if (state_id as usize) >= (1usize << self.bits) {
                let new_bits = (32 - state_id.leading_zeros() as usize).max(self.bits);
                let mut existing = Vec::with_capacity(4096);
                for idx in 0..4096 {
                    existing.push(self.get(idx)?);
                }
                existing[i] = state_id;
                self.bits = new_bits;
                let per = 64 / self.bits;
                self.words = vec![0u64; 4096usize.div_ceil(per)];
                for (idx, val) in existing.into_iter().enumerate() {
                    let w = idx / per;
                    let off = (idx % per) * self.bits;
                    self.words[w] |= (val as u64) << off;
                }
                return Ok(());
            }
            state_id as usize
        } else if let Some(pos) = self.ids.iter().position(|&id| id == state_id) {
            pos
        } else if self.ids.len() < (1 << self.bits) {
            self.ids.push(state_id);
            self.ids.len() - 1
        } else {
            // Promote to direct global palette when indirect capacity is exceeded
            let mut existing = Vec::with_capacity(4096);
            for idx in 0..4096 {
                existing.push(self.get(idx)?);
            }
            existing[i] = state_id;
            let max_id = existing.iter().copied().max().unwrap_or(0).max(state_id);
            let direct_bits = (32 - max_id.leading_zeros() as usize).max(15);
            self.bits = direct_bits;
            self.ids.clear();
            let per = 64 / self.bits;
            self.words = vec![0u64; 4096usize.div_ceil(per)];
            for (idx, val) in existing.into_iter().enumerate() {
                let w = idx / per;
                let off = (idx % per) * self.bits;
                self.words[w] |= (val as u64) << off;
            }
            return Ok(());
        };
        let per = 64 / self.bits;
        let mask = (1u64 << self.bits) - 1;
        let w = i / per;
        let off = (i % per) * self.bits;
        if w >= self.words.len() {
            return Err(bad("palette storage truncated"));
        }
        self.words[w] = (self.words[w] & !(mask << off)) | (((palette_index as u64) & mask) << off);
        Ok(())
    }
}

pub struct World {
    pub min_y: i32,
    pub height: i32,
    pub chunks: HashMap<(i32, i32), Vec<Palette>>,
    air: HashSet<u32>,
    solid: HashSet<u32>,
    pub bottom_slabs: HashSet<u32>,
    pub top_slabs: HashSet<u32>,
    pub stairs: HashSet<u32>,
    pub fences: HashSet<u32>,
    pub carpets: HashSet<u32>,
    pub climbable: HashSet<u32>,
    pub water: HashSet<u32>,
    pub lava: HashSet<u32>,
    pub cobwebs: HashSet<u32>,
    pub soul_sand: HashSet<u32>,
    pub ice: HashSet<u32>,
    pub hardness_map: HashMap<u32, f32>,
    pub last_unknown_state: Option<(i32, i32, i32, u32)>,
}

/// Returns a complete, built-in 26.3 (`protocol 777`) default block profile with standard air and solid states.
pub fn default_26_3_profile() -> serde_json::Value {
    serde_json::json!({
        "minecraft": "26.3",
        "air": [0],
        "solid": [1],
        "hardness": {
            "1": 1.5
        }
    })
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
        let opt_ids = |name: &str| -> HashSet<u32> {
            profile
                .get(name)
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_u64().filter(|n| *n < 1_000_000).map(|n| n as u32))
                        .collect()
                })
                .unwrap_or_default()
        };
        let air = ids("air")?;
        let solid = ids("solid")?;
        if air.is_empty() || solid.is_empty() || !air.is_disjoint(&solid) {
            return Err(bad("block profile needs distinct air and solid states"));
        }
        let mut hardness_map = HashMap::new();
        if let Some(h_obj) = profile.get("hardness").and_then(|v| v.as_object()) {
            for (k, v) in h_obj {
                if let (Ok(id), Some(h)) = (k.parse::<u32>(), v.as_f64()) {
                    hardness_map.insert(id, h as f32);
                }
            }
        }
        Ok(Self {
            min_y: -64,
            height: 384,
            chunks: HashMap::new(),
            air,
            solid,
            bottom_slabs: opt_ids("bottom_slabs"),
            top_slabs: opt_ids("top_slabs"),
            stairs: opt_ids("stairs"),
            fences: opt_ids("fences"),
            carpets: opt_ids("carpets"),
            climbable: opt_ids("climbable"),
            water: opt_ids("water"),
            lava: opt_ids("lava"),
            cobwebs: opt_ids("cobwebs"),
            soul_sand: opt_ids("soul_sand"),
            ice: opt_ids("ice"),
            hardness_map,
            last_unknown_state: None,
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

    pub fn block_state(&self, x: i32, y: i32, z: i32) -> Result<u32> {
        if y < self.min_y || y >= self.min_y + self.height {
            return Err(bad("outside known dimension"));
        }
        let chunk = self
            .chunks
            .get(&(x.div_euclid(16), z.div_euclid(16)))
            .ok_or_else(|| bad("collision data not loaded"))?;
        let index =
            ((y.rem_euclid(16) * 256) + (z.rem_euclid(16) * 16) + x.rem_euclid(16)) as usize;
        chunk[((y - self.min_y) / 16) as usize].get(index)
    }

    /// Updates a single block state in a loaded chunk (`ClientboundBlockUpdatePacket`, packet 8).
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, state_id: u32) -> Result<()> {
        if y < self.min_y || y >= self.min_y + self.height {
            return Ok(());
        }
        let Some(chunk) = self.chunks.get_mut(&(x.div_euclid(16), z.div_euclid(16))) else {
            return Ok(());
        };
        let sec_idx = ((y - self.min_y) / 16) as usize;
        if let Some(section) = chunk.get_mut(sec_idx) {
            let index =
                ((y.rem_euclid(16) * 256) + (z.rem_euclid(16) * 16) + x.rem_euclid(16)) as usize;
            section.set(index, state_id)?;
        }
        Ok(())
    }

    /// Decodes a packed 64-bit `BlockPos` (`x: 26b`, `z: 26b`, `y: 12b`) and applies packet 8 (`BlockUpdate`).
    pub fn apply_block_update_packet(&mut self, r: &mut Reader<'_>) -> Result<([i32; 3], u32)> {
        let packed = r.u64()? as i64;
        let x = (packed >> 38) as i32;
        let y = ((packed << 52) >> 52) as i32;
        let z = ((packed << 26) >> 38) as i32;
        let state_id = r.count(1_000_000)? as u32;
        self.set_block(x, y, z, state_id)?;
        Ok(([x, y, z], state_id))
    }

    pub fn is_air_state(&self, state_id: u32) -> bool {
        self.air.contains(&state_id)
    }

    /// Decodes packet 86 (`SectionBlocksUpdate`) and applies all local block state changes.
    pub fn apply_section_blocks_update_packet(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let section_pos = r.u64()? as i64;
        let sec_x = (section_pos >> 42) as i32;
        let sec_z = ((section_pos << 22) >> 42) as i32;
        let sec_y = ((section_pos << 44) >> 44) as i32;
        let count = r.count(4096)?;
        for _ in 0..count {
            let mut val = 0u64;
            for shift in 0..10 {
                let b = r.byte()?;
                val |= u64::from(b & 0x7f) << (shift * 7);
                if b & 0x80 == 0 {
                    break;
                }
            }
            let local = (val & 0xfff) as i32;
            let state_id = (val >> 12) as u32;
            let lx = (local >> 8) & 15;
            let lz = (local >> 4) & 15;
            let ly = local & 15;
            self.set_block(sec_x * 16 + lx, sec_y * 16 + ly, sec_z * 16 + lz, state_id)?;
        }
        Ok(())
    }

    pub fn is_known_state(&self, id: u32) -> bool {
        self.air.contains(&id)
            || self.solid.contains(&id)
            || self.bottom_slabs.contains(&id)
            || self.top_slabs.contains(&id)
            || self.stairs.contains(&id)
            || self.fences.contains(&id)
            || self.carpets.contains(&id)
            || self.climbable.contains(&id)
            || self.water.contains(&id)
            || self.lava.contains(&id)
            || self.cobwebs.contains(&id)
            || self.soul_sand.contains(&id)
            || self.ice.contains(&id)
    }

    pub fn check_region_known(&self, aabb: [f64; 6]) -> std::result::Result<(), (i32, i32, i32, u32)> {
        let min_x = aabb[0].floor() as i32;
        let max_x = (aabb[3] - 1e-7).floor() as i32;
        let min_y = aabb[1].floor() as i32;
        let max_y = (aabb[4] - 1e-7).floor() as i32;
        let min_z = aabb[2].floor() as i32;
        let max_z = (aabb[5] - 1e-7).floor() as i32;
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                for x in min_x..=max_x {
                    if let Ok(id) = self.block_state(x, y, z) {
                        if !self.is_known_state(id) {
                            return Err((x, y, z, id));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn block_hardness(&self, x: i32, y: i32, z: i32) -> Option<f32> {
        let id = self.block_state(x, y, z).ok()?;
        if self.air.contains(&id) || self.water.contains(&id) || self.lava.contains(&id) {
            return Some(0.0);
        }
        if let Some(&h) = self.hardness_map.get(&id) {
            return Some(h);
        }
        if self.is_known_state(id) {
            Some(1.5)
        } else {
            None
        }
    }

    pub fn solid(&self, x: i32, y: i32, z: i32) -> Result<bool> {
        let id = self.block_state(x, y, z)?;
        if self.air.contains(&id)
            || self.climbable.contains(&id)
            || self.water.contains(&id)
            || self.lava.contains(&id)
            || self.cobwebs.contains(&id)
        {
            Ok(false)
        } else if self.solid.contains(&id)
            || self.bottom_slabs.contains(&id)
            || self.top_slabs.contains(&id)
            || self.stairs.contains(&id)
            || self.fences.contains(&id)
            || self.carpets.contains(&id)
            || self.soul_sand.contains(&id)
            || self.ice.contains(&id)
        {
            Ok(true)
        } else {
            // Safe non-panicking solid fallback so player never falls through unknown geometry
            Ok(true)
        }
    }

    /// Returns exact world-space AABBs `[min_x, min_y, min_z, max_x, max_y, max_z]` for any supported block shape.
    pub fn collision_boxes(&self, x: i32, y: i32, z: i32) -> Result<Vec<[f64; 6]>> {
        let id = self.block_state(x, y, z)?;
        let bx = x as f64;
        let by = y as f64;
        let bz = z as f64;
        if self.air.contains(&id)
            || self.climbable.contains(&id)
            || self.water.contains(&id)
            || self.lava.contains(&id)
            || self.cobwebs.contains(&id)
        {
            Ok(Vec::new())
        } else if self.solid.contains(&id) || self.ice.contains(&id) {
            Ok(vec![[bx, by, bz, bx + 1.0, by + 1.0, bz + 1.0]])
        } else if self.bottom_slabs.contains(&id) {
            Ok(vec![[bx, by, bz, bx + 1.0, by + 0.5, bz + 1.0]])
        } else if self.top_slabs.contains(&id) {
            Ok(vec![[bx, by + 0.5, bz, bx + 1.0, by + 1.0, bz + 1.0]])
        } else if self.stairs.contains(&id) {
            Ok(vec![
                [bx, by, bz, bx + 1.0, by + 0.5, bz + 1.0],
                [bx, by + 0.5, bz + 0.5, bx + 1.0, by + 1.0, bz + 1.0],
            ])
        } else if self.fences.contains(&id) {
            Ok(vec![[
                bx + 0.375,
                by,
                bz + 0.375,
                bx + 0.625,
                by + 1.5,
                bz + 0.625,
            ]])
        } else if self.carpets.contains(&id) {
            Ok(vec![[bx, by, bz, bx + 1.0, by + 0.0625, bz + 1.0]])
        } else if self.soul_sand.contains(&id) {
            Ok(vec![[bx, by, bz, bx + 1.0, by + 0.875, bz + 1.0]])
        } else {
            // Safe non-panicking solid unit cube fallback so player never falls through floor
            Ok(vec![[bx, by, bz, bx + 1.0, by + 1.0, bz + 1.0]])
        }
    }

    pub fn is_climbable(&self, x: i32, y: i32, z: i32) -> bool {
        self.block_state(x, y, z)
            .is_ok_and(|id| self.climbable.contains(&id))
    }

    pub fn is_water(&self, x: i32, y: i32, z: i32) -> bool {
        self.block_state(x, y, z)
            .is_ok_and(|id| self.water.contains(&id))
    }

    pub fn is_lava(&self, x: i32, y: i32, z: i32) -> bool {
        self.block_state(x, y, z)
            .is_ok_and(|id| self.lava.contains(&id))
    }

    pub fn is_cobweb(&self, x: i32, y: i32, z: i32) -> bool {
        self.block_state(x, y, z)
            .is_ok_and(|id| self.cobwebs.contains(&id))
    }

    pub fn slipperiness(&self, x: i32, y: i32, z: i32) -> f32 {
        match self.block_state(x, y, z) {
            Ok(id) if self.ice.contains(&id) => 0.98,
            _ => 0.6,
        }
    }

    pub fn loaded(&self, x: f64, z: f64) -> bool {
        self.chunks.contains_key(&(
            (x.floor() as i32).div_euclid(16),
            (z.floor() as i32).div_euclid(16),
        ))
    }

    /// Writes a compact, tensor-ready local voxel observation patch around `center` into `out`
    /// (`0 = air`, `1 = solid`, `2 = water`, `3 = climbable`, `4 = lava/hazard`) without intermediate heap allocations.
    pub fn extract_voxel_patch(
        &self,
        center: [i32; 3],
        rx: i32,
        ry: i32,
        rz: i32,
        out: &mut Vec<u8>,
    ) {
        out.clear();
        let cap = ((2 * rx + 1) * (2 * ry + 1) * (2 * rz + 1)).max(0) as usize;
        if out.capacity() < cap {
            out.reserve(cap - out.capacity());
        }
        for dy in -ry..=ry {
            for dz in -rz..=rz {
                for dx in -rx..=rx {
                    let x = center[0] + dx;
                    let y = center[1] + dy;
                    let z = center[2] + dz;
                    let code = if self.is_lava(x, y, z) {
                        4
                    } else if self.is_climbable(x, y, z) {
                        3
                    } else if self.is_water(x, y, z) {
                        2
                    } else if self.solid(x, y, z).unwrap_or(true) {
                        1
                    } else {
                        0
                    };
                    out.push(code);
                }
            }
        }
    }

    /// Renders a top-down 2D ASCII view directly from the live server's loaded chunk sections.
    pub fn render_live_ascii_map(
        &self,
        player_pos: [f64; 3],
        yaw: f32,
        hotbar: u8,
        entities: &[(i32, [f64; 3])],
        radius: i32,
    ) -> String {
        let px = player_pos[0].floor() as i32;
        let py = player_pos[1].floor() as i32;
        let pz = player_pos[2].floor() as i32;
        let mut out = format!(
            "Sawbyte Live 2D World Inspector (pos=({:.2}, {:.2}, {:.2}), yaw={:.1}°, hotbar={}, loaded_chunks={})\n",
            player_pos[0],
            player_pos[1],
            player_pos[2],
            yaw,
            hotbar,
            self.chunks.len()
        );
        for z in (pz - radius)..=(pz + radius) {
            for x in (px - radius)..=(px + radius) {
                if x == px && z == pz {
                    out.push('@');
                } else if entities
                    .iter()
                    .any(|(_, pos)| pos[0].floor() as i32 == x && pos[2].floor() as i32 == z)
                {
                    out.push('E');
                } else if !self.loaded(x as f64, z as f64) {
                    out.push('?');
                } else if <Self as VoxelQuery>::is_solid(self, x, py + 1, z) {
                    out.push('#');
                } else if <Self as VoxelQuery>::is_solid(self, x, py, z) {
                    out.push('^');
                } else if self.is_water(x, py, z) {
                    out.push('~');
                } else if <Self as VoxelQuery>::is_solid(self, x, py - 1, z) {
                    out.push('.');
                } else {
                    out.push(' ');
                }
            }
            out.push('\n');
        }
        out.push_str(
            "Legend: @=Player  E=Entity  #=Wall(2+)  ^=Step(1)  ~=Water  .=Ground  ?=Unloaded\n",
        );
        out
    }
}

impl VoxelQuery for World {
    fn is_solid(&self, x: i32, y: i32, z: i32) -> bool {
        // Treat unloaded chunks or unknown block states as impassable so pathfinding never steps into void.
        self.solid(x, y, z).unwrap_or(true)
    }

    fn is_climbable(&self, x: i32, y: i32, z: i32) -> bool {
        World::is_climbable(self, x, y, z)
    }

    fn is_water(&self, x: i32, y: i32, z: i32) -> bool {
        World::is_water(self, x, y, z)
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
        // Unknown block state is safely treated as solid to avoid falling through geometry
        assert!(world.solid(-1, -1, -1).unwrap());
        assert_eq!(world.collision_boxes(-1, -1, -1).unwrap(), vec![[-1.0, -1.0, -1.0, 0.0, 0.0, 0.0]]);
        assert_eq!(world.check_region_known([-1.0, -1.0, -1.0, 0.0, 0.0, 0.0]), Err((-1, -1, -1, 2)));
        assert_eq!(world.block_hardness(-1, -1, -1), None);
        assert!(World::new(&json!({"minecraft":"26.3","air":[0],"solid":[0]})).is_err());
        assert!(World::new(&json!({"minecraft":"26.2","air":[0],"solid":[1]})).is_err());
    }

    #[test]
    fn dynamic_block_update_promotes_palette_and_supports_slabs() {
        let mut world = World::new(
            &json!({"minecraft":"26.3","air":[0],"solid":[1],"bottom_slabs":[2],"climbable":[3]}),
        )
        .unwrap();
        world.min_y = -64;
        world.height = 16;
        world.chunks.insert(
            (0, 0),
            vec![Palette {
                bits: 0,
                ids: vec![0],
                words: vec![],
            }],
        );
        assert!(!world.solid(2, -60, 2).unwrap());
        world.set_block(2, -60, 2, 2).unwrap();
        assert!(world.solid(2, -60, 2).unwrap());
        let boxes = world.collision_boxes(2, -60, 2).unwrap();
        assert_eq!(boxes, vec![[2.0, -60.0, 2.0, 3.0, -59.5, 3.0]]);
        world.set_block(2, -59, 2, 3).unwrap();
        assert!(world.is_climbable(2, -59, 2));
    }

    #[test]
    fn palette_set_promotes_to_direct_with_large_state_ids() {
        let mut p = Palette::single(0);
        for id in 1..=20 {
            p.set(id as usize, id * 2000).unwrap();
        }
        for id in 1..=20 {
            assert_eq!(p.get(id as usize).unwrap(), id * 2000);
        }
        assert_eq!(p.get(0).unwrap(), 0);
        assert_eq!(p.get(21).unwrap(), 0);
    }
}
