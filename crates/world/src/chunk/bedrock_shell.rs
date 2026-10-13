use glam::Vec3;
use super::VoxelBarrier;

/// Indestructible single-block core shell sealing the hollow planetary core.
///
/// Prevents avatars, tools, projectiles, and falling debris from breaching the 0-byte inner abyss,
/// serving as the rigid gravitational anchor and foundation for structural integrity.
pub struct BedrockShell {
    inner_half_extent: f32,
    thickness: f32,
}

impl BedrockShell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(inner_half_extent: f32, thickness: f32) -> Self {
        Self {
            inner_half_extent,
            thickness,
        }
    }

    /// Secondary constructor around an inner core void with unit 1-block thickness.
    pub fn boundary(inner_half_extent: f32) -> Self {
        Self::new(inner_half_extent, 1.0)
    }

    fn chebyshev_norm(&self, p: Vec3) -> f32 {
        p.x.abs().max(p.y.abs()).max(p.z.abs())
    }
}

impl VoxelBarrier for BedrockShell {
    fn contained(&self, position: Vec3) -> bool {
        self.chebyshev_norm(position) >= (self.inner_half_extent + self.thickness)
    }

    fn arrest_penetration(&self, origin: Vec3, destination: Vec3) -> Vec3 {
        let outer_boundary = self.inner_half_extent + self.thickness;
        let dest_norm = self.chebyshev_norm(destination);
        if dest_norm >= outer_boundary {
            return destination;
        }

        // Project back outward along the dominant axis of the origin or destination
        let mut arrested = destination;
        let (ax, ay, az) = (origin.x.abs(), origin.y.abs(), origin.z.abs());
        if ax >= ay && ax >= az {
            let sign = if origin.x >= 0.0 { 1.0 } else { -1.0 };
            arrested.x = sign * outer_boundary;
        } else if ay >= ax && ay >= az {
            let sign = if origin.y >= 0.0 { 1.0 } else { -1.0 };
            arrested.y = sign * outer_boundary;
        } else {
            let sign = if origin.z >= 0.0 { 1.0 } else { -1.0 };
            arrested.z = sign * outer_boundary;
        }
        arrested
    }

    fn indestructible_at(&self, position: Vec3) -> bool {
        let norm = self.chebyshev_norm(position);
        norm >= self.inner_half_extent && norm < (self.inner_half_extent + self.thickness)
    }
}
