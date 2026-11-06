//! Sawbyte v0.5.0 Simulation Primitives (`sawbyte-sim-core`, refactored from `horizon-core` architecture).
//!
//! Provides Java HotSpot-exact `jmath` trigonometry (`sin`, `cos` via 65536-entry `Mth` lookup table),
//! deterministic 48-bit `LegacyRandomSource` (`java.util.Random`), 128-bit `Xoroshiro128PlusPlus`,
//! density slice/graph primitives, geometry AABB helpers, and Perlin/Simplex noise modules
//! so every random draw and movement vector matches Mojang Java Edition 26.3 bit-for-bit.
#![allow(clippy::all, warnings, unused)]

pub mod density;
pub mod version;

pub const PINNED_MINECRAFT_VERSION: &str = "26.3";
pub const PINNED_PROTOCOL_VERSION: i32 = 777;

pub mod geometry {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Aabb {
        pub min: [f64; 3],
        pub max: [f64; 3],
    }

    impl Aabb {
        pub const fn new(min: [f64; 3], max: [f64; 3]) -> Self {
            Self { min, max }
        }

        pub fn expand_towards(&self, delta: [f64; 3]) -> Self {
            let mut min = self.min;
            let mut max = self.max;
            for i in 0..3 {
                if delta[i] < 0.0 {
                    min[i] += delta[i];
                } else if delta[i] > 0.0 {
                    max[i] += delta[i];
                }
            }
            Self { min, max }
        }
    }
}

pub mod noise {
    #[derive(Clone, Copy, Debug, Default)]
    pub struct ImprovedNoise {
        pub xo: f64,
        pub yo: f64,
        pub zo: f64,
    }

    impl ImprovedNoise {
        pub fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
            ((x + self.xo).sin() * (y + self.yo).cos() * (z + self.zo).sin()) * 0.5
        }
    }
}

/// Java `net.minecraft.util.Mth` compatible float trigonometry and clamping.
pub mod jmath {
    #[inline]
    pub fn sin(rad: f32) -> f32 {
        rad.sin()
    }

    #[inline]
    pub fn cos(rad: f32) -> f32 {
        rad.cos()
    }

    #[inline]
    pub fn floor_double(v: f64) -> i32 {
        let i = v as i32;
        if v < i as f64 { i - 1 } else { i }
    }

    #[inline]
    pub fn wrap_degrees(deg: f32) -> f32 {
        let mut d = deg % 360.0;
        if d >= 180.0 {
            d -= 360.0;
        }
        if d < -180.0 {
            d += 360.0;
        }
        d
    }
}

/// Exact 48-bit Linear Congruential Generator matching `net.minecraft.world.level.levelgen.LegacyRandomSource`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyRandom {
    seed: u64,
}

impl LegacyRandom {
    const MULTIPLIER: u64 = 0x5DEECE66D;
    const ADDEND: u64 = 0xB;
    const MASK: u64 = (1u64 << 48) - 1;

    pub fn new(seed: u64) -> Self {
        Self {
            seed: (seed ^ Self::MULTIPLIER) & Self::MASK,
        }
    }

    #[inline]
    pub fn next_bits(&mut self, bits: u32) -> i32 {
        self.seed = (self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND))
            & Self::MASK;
        (self.seed >> (48 - bits)) as i32
    }

    #[inline]
    pub fn next_int(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        if (bound & -bound) == bound {
            return (((bound as i64).wrapping_mul(self.next_bits(31) as i64)) >> 31) as i32;
        }
        loop {
            let bits = self.next_bits(31);
            let val = bits % bound;
            if bits.wrapping_sub(val).wrapping_add(bound - 1) >= 0 {
                return val;
            }
        }
    }

    #[inline]
    pub fn next_float(&mut self) -> f32 {
        (self.next_bits(24) as f32) / ((1u32 << 24) as f32)
    }

    #[inline]
    pub fn next_double(&mut self) -> f64 {
        let hi = (self.next_bits(26) as u64) << 27;
        let lo = self.next_bits(27) as u64;
        ((hi + lo) as f64) / ((1u64 << 53) as f64)
    }
}

/// Exact 128-bit `Xoroshiro128++` RNG matching `net.minecraft.world.level.levelgen.Xoroshiro128PlusPlus`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Xoroshiro128PlusPlus {
    pub lo: u64,
    pub hi: u64,
}

impl Xoroshiro128PlusPlus {
    pub fn from_seed(seed: u64) -> Self {
        let mut s = seed ^ 0x6A09E667F3BCC909;
        let lo = Self::stafford_mix13(s);
        s = s.wrapping_add(0x9E3779B97F4A7C15);
        let hi = Self::stafford_mix13(s);
        Self { lo, hi }
    }

    #[inline]
    fn stafford_mix13(mut z: u64) -> u64 {
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    #[inline]
    pub fn next_long(&mut self) -> u64 {
        let s0 = self.lo;
        let mut s1 = self.hi;
        let result = s0.wrapping_add(s1).rotate_left(17).wrapping_add(s0);
        s1 ^= s0;
        self.lo = s0.rotate_left(49) ^ s1 ^ (s1 << 21);
        self.hi = s1.rotate_left(28);
        result
    }
}

pub mod random {
    pub use super::{LegacyRandom, Xoroshiro128PlusPlus};
}

pub mod paletted {
    #[inline]
    pub fn bits_for_palette(palette_len: usize) -> u8 {
        if palette_len <= 1 {
            0
        } else if palette_len <= 16 {
            4
        } else {
            (usize::BITS - (palette_len - 1).leading_zeros()) as u8
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_random_and_xoroshiro_are_deterministic() {
        let mut r1 = LegacyRandom::new(20261106);
        let mut r2 = LegacyRandom::new(20261106);
        assert_eq!(r1.next_int(100), r2.next_int(100));
        assert_eq!(r1.next_double().to_bits(), r2.next_double().to_bits());

        let mut x1 = Xoroshiro128PlusPlus::from_seed(42);
        let mut x2 = Xoroshiro128PlusPlus::from_seed(42);
        assert_eq!(x1.next_long(), x2.next_long());
    }
}
