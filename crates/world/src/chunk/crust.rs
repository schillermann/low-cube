use glam::Vec3;
use super::{BedrockShell, VoxelBarrier, VOLUME_EDGE_LENGTH};

/// Encapsulated geological crust configuration governing planetary depth and bedrock foundation.
pub struct PlanetaryCrust {
    dimension_blocks: usize,
    crust_depth_blocks: usize,
}

impl PlanetaryCrust {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(dimension_blocks: usize, crust_depth_blocks: usize) -> Self {
        Self {
            dimension_blocks,
            crust_depth_blocks,
        }
    }

    /// Secondary constructor calibrating authentic planetary crust thickness based on world scale.
    pub fn calibrated(dimension_blocks: usize) -> Self {
        let crust_depth = if dimension_blocks <= 128 {
            16
        } else if dimension_blocks <= 1024 {
            32
        } else {
            48
        };
        Self::new(dimension_blocks, crust_depth)
    }

    /// Crust thickness in discrete 16-block volume increments.
    pub fn crust_depth_volumes(&self) -> usize {
        self.crust_depth_blocks.div_ceil(VOLUME_EDGE_LENGTH)
    }

    /// Half-extent of the inner hollow core void in spatial meters.
    pub fn inner_void_half_extent(&self) -> f32 {
        let half_world = (self.dimension_blocks as f32) * 0.5;
        (half_world - self.crust_depth_blocks as f32).max(0.0)
    }

    /// Constructs the impenetrable bedrock barrier sealing the inner void.
    pub fn bedrock_barrier(&self) -> Box<dyn VoxelBarrier> {
        Box::new(BedrockShell::boundary(self.inner_void_half_extent()))
    }

    /// Enforces containment by testing whether a point is safely in the crust or atmosphere.
    pub fn contained(&self, position: Vec3) -> bool {
        self.bedrock_barrier().contained(position)
    }
}
