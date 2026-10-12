use glam::Vec3;
use crate::{ClimateZone, Face, MeshTarget, Quad};

/// Pure geometric planar face operating strictly on coordinate bases without integer switches.
pub struct PlanarFace {
    normal: Vec3,
    tangent: Vec3,
    bitangent: Vec3,
    color: Vec3,
    half_size: f32,
    divisions: u32,
}

impl PlanarFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        normal: Vec3,
        tangent: Vec3,
        bitangent: Vec3,
        color: Vec3,
        half_size: f32,
        divisions: u32,
    ) -> Self {
        Self {
            normal,
            tangent,
            bitangent,
            color,
            half_size,
            divisions,
        }
    }

    /// Secondary constructor creating a planar face from a planetary climate zone.
    pub fn in_zone(zone: ClimateZone, color: Vec3, half_size: f32, divisions: u32) -> Self {
        Self::new(
            zone.normal(),
            zone.tangent(),
            zone.bitangent(),
            color,
            half_size,
            divisions,
        )
    }

    /// Backwards-compatible alias for in_zone.
    pub fn on_cube(zone: ClimateZone, color: Vec3, half_size: f32, divisions: u32) -> Self {
        Self::in_zone(zone, color, half_size, divisions)
    }
}

impl Face for PlanarFace {
    fn matches(&self, point: Vec3) -> bool {
        let height = point.dot(self.normal);
        if height <= 0.0 {
            return false;
        }

        let u = point.dot(self.tangent).abs();
        let v = point.dot(self.bitangent).abs();

        height >= u && height >= v
    }

    fn surface_up(&self) -> Vec3 {
        self.normal
    }

    fn pull_direction(&self) -> Vec3 {
        -self.normal
    }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        let u = point.dot(self.tangent);
        let v = point.dot(self.bitangent);
        let h = self.half_size + distance;

        self.normal * h + self.tangent * u + self.bitangent * v
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        let step = (self.half_size * 2.0) / (self.divisions as f32);
        let base_center = self.normal * self.half_size;

        for i in 0..self.divisions {
            for j in 0..self.divisions {
                let u0 = -self.half_size + (i as f32) * step;
                let u1 = u0 + step;
                let v0 = -self.half_size + (j as f32) * step;
                let v1 = v0 + step;

                let pattern = ((i + j) % 2 == 0) as i32 as f32 * 0.08;
                let quad_color = Vec3::new(
                    (self.color.x + pattern).clamp(0.0, 1.0),
                    (self.color.y + pattern).clamp(0.0, 1.0),
                    (self.color.z + pattern).clamp(0.0, 1.0),
                );

                let p0 = base_center + self.tangent * u0 + self.bitangent * v0;
                let p1 = base_center + self.tangent * u1 + self.bitangent * v0;
                let p2 = base_center + self.tangent * u1 + self.bitangent * v1;
                let p3 = base_center + self.tangent * u0 + self.bitangent * v1;

                target.push_quad(Quad::new(p0, p1, p2, p3, self.normal, quad_color));
            }
        }
    }
}
