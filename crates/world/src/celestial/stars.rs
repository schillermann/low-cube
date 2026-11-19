use std::f32::consts::TAU;
use glam::Vec3;
use crate::{MeshTarget, Quad};

/// Deterministic 3D celestial star sphere emitting glowing diamond quads across all sky directions.
#[derive(Clone, Copy, Debug)]
pub struct StarVault {
    star_count: u32,
    radius: f32,
}

impl StarVault {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(star_count: u32, radius: f32) -> Self {
        Self { star_count, radius }
    }

    pub const fn cosmic() -> Self {
        Self::new(360, 1850.0)
    }

    fn hash_unit(seed: u32) -> f32 {
        let mut h = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
        h = ((h >> ((h >> 28).wrapping_add(4))) ^ h).wrapping_mul(277_803_737);
        h ^= h >> 22;
        ((h & 0x00FF_FFFF) as f32) / 16_777_215.0
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        for idx in 0..self.star_count {
            let s0 = Self::hash_unit(idx * 3 + 11);
            let s1 = Self::hash_unit(idx * 3 + 23);
            let s2 = Self::hash_unit(idx * 3 + 37);
            let z = s0 * 2.0 - 1.0;
            let ring = (1.0 - z * z).max(0.0).sqrt();
            let phi = s1 * TAU;
            let dir = Vec3::new(ring * phi.cos(), z, ring * phi.sin()).normalize_or_zero();
            let ref_up = if dir.y.abs() > 0.9 { Vec3::X } else { Vec3::Y };
            let tang = dir.cross(ref_up).normalize_or_zero();
            let bitang = tang.cross(dir).normalize_or_zero();
            let span = self.radius * (0.0022 + 0.0028 * s2);
            let center = dir * self.radius;
            let col = match idx % 4 {
                0 => Vec3::new(0.84, 0.92, 1.00),
                1 => Vec3::new(1.00, 0.96, 0.86),
                2 => Vec3::new(1.00, 0.84, 0.66),
                _ => Vec3::new(0.76, 0.88, 1.00),
            };
            target.push_quad(Quad::glowing(
                center - tang * span,
                center - bitang * span,
                center + tang * span,
                center + bitang * span,
                -dir,
                col,
                0.98,
            ));
        }
    }
}
