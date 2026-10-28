use glam::Vec3;
use crate::{ClimateZone, Face, MeshTarget, Quad, SurfaceFrame};

/// Pure geometric planar face operating strictly on coordinate bases without integer switches.
pub struct PlanarFace {
    frame: SurfaceFrame,
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
            frame: SurfaceFrame::new(tangent, bitangent, normal, half_size),
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
        let height = self.frame.height_of(point);
        if height <= 0.0 {
            return false;
        }
        let (u, v) = self.frame.decompose(point);
        height >= u.abs() && height >= v.abs()
    }

    fn surface_up(&self) -> Vec3 {
        self.frame.normal()
    }

    fn pull_direction(&self) -> Vec3 {
        -self.frame.normal()
    }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        let (u, v) = self.frame.decompose(point);
        self.frame.point_at(u, v, distance)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        let step = (self.half_size * 2.0) / (self.divisions as f32);
        let normal = self.frame.normal();

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

                let p0 = self.frame.point_at(u0, v0, 0.0);
                let p1 = self.frame.point_at(u1, v0, 0.0);
                let p2 = self.frame.point_at(u1, v1, 0.0);
                let p3 = self.frame.point_at(u0, v1, 0.0);

                target.push_quad(Quad::new(p0, p1, p2, p3, normal, quad_color));
            }
        }
    }
}
