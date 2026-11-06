//! X/Y/Z and combined XZ density volume slicing without Rust 2021 let-chain incompatibilities.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SliceAxis {
    X,
    Y,
    Z,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VolumeBounds {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub size_x: usize,
    pub size_y: usize,
    pub size_z: usize,
}

impl VolumeBounds {
    pub const fn new(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        size_x: usize,
        size_y: usize,
        size_z: usize,
    ) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            size_x,
            size_y,
            size_z,
        }
    }

    pub const fn cell_count(&self) -> usize {
        self.size_x
            .saturating_mul(self.size_y)
            .saturating_mul(self.size_z)
    }

    pub fn reduce_axis(&self, axis: SliceAxis, fixed_coord: Option<i32>) -> Option<Self> {
        let Some(coord) = fixed_coord else {
            return None;
        };
        match axis {
            SliceAxis::X => Some(Self {
                min_x: coord,
                size_x: 1,
                ..*self
            }),
            SliceAxis::Y => Some(Self {
                min_y: coord,
                size_y: 1,
                ..*self
            }),
            SliceAxis::Z => Some(Self {
                min_z: coord,
                size_z: 1,
                ..*self
            }),
        }
    }

    pub fn reduce_xz(&self, fixed_x: Option<i32>, fixed_z: Option<i32>) -> Self {
        if let (Some(x), Some(z)) = (fixed_x, fixed_z) {
            return Self {
                min_x: x,
                min_z: z,
                size_x: 1,
                size_z: 1,
                ..*self
            };
        }
        if let Some(x) = fixed_x {
            return Self {
                min_x: x,
                size_x: 1,
                ..*self
            };
        }
        if let Some(z) = fixed_z {
            return Self {
                min_z: z,
                size_z: 1,
                ..*self
            };
        }
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_bounds_reduce_xz_handles_combined_and_single_slices() {
        let bounds = VolumeBounds::new(0, -64, 0, 4, 24, 4);
        let reduced = bounds.reduce_xz(Some(2), Some(3));
        assert_eq!(reduced.min_x, 2);
        assert_eq!(reduced.min_z, 3);
        assert_eq!(reduced.size_x, 1);
        assert_eq!(reduced.size_z, 1);
        assert_eq!(reduced.cell_count(), 24);
    }
}
