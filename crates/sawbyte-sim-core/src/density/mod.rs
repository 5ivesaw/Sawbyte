//! Density function slice, volume evaluation, and graph compiler primitives for Minecraft 26.3 worldgen.

pub mod slice;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleError {
    OutOfBounds,
    UnboundContextField,
    InvalidGraphNode,
}

pub type DensityValue = f64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Mul,
    Min,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldField {
    Continents,
    Erosion,
    Depth,
    Ridges,
}

#[derive(Clone, Debug, Default)]
pub struct WorldFields {
    pub continents: f64,
    pub erosion: f64,
    pub depth: f64,
    pub ridges: f64,
}

pub trait DensitySampler: Send + Sync {
    fn sample(&self, x: i32, y: i32, z: i32) -> Result<DensityValue, SampleError>;
}

#[derive(Clone, Debug)]
pub struct BinarySampler {
    pub op: BinaryOp,
    pub left: f64,
    pub right: f64,
}

#[derive(Clone, Debug, Default)]
pub struct BlendDensitySampler {
    pub weight: f64,
}

pub mod graph {
    use super::{DensityValue, SampleError};

    #[derive(Clone, Debug, Default)]
    pub struct DensityGraph {
        pub node_count: usize,
    }

    impl DensityGraph {
        pub fn evaluate_constant(&self, val: DensityValue) -> Result<DensityValue, SampleError> {
            Ok(val)
        }
    }
}
