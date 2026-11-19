use std::f32::consts::PI;
use glam::Vec3;
use crate::{CelestialSystem, MeshTarget, Quad};

/// Floating 3D holographic Earth-Moon-Spacecraft orrery projected above a helmed `FlightConsole`.
pub struct ConsoleOrrery {
    ship_origin: Vec3,
    console_base: Vec3,
    axes: (Vec3, Vec3),
}

impl ConsoleOrrery {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ship_origin: Vec3, console_base: Vec3, axes: (Vec3, Vec3)) -> Self {
        Self { ship_origin, console_base, axes }
    }

    fn emit_strut(a: Vec3, b: Vec3, radius: f32, col: Vec3, glow: f32, ref_up: Vec3, target: &mut dyn MeshTarget) {
        let dir = b - a;
        if dir.length_squared() <= 1e-8 {
            return;
        }
        let axis = dir.normalize();
        let pivot = if axis.dot(ref_up).abs() < 0.9 { ref_up } else { Vec3::X };
        let sa = axis.cross(pivot).normalize_or_zero() * radius;
        let sb = axis.cross(sa).normalize_or_zero() * radius;
        for (o0, o1, norm) in [
            (-sa, sa, -sb.normalize_or_zero()),
            (sa, sb, sa.normalize_or_zero()),
            (sb, -sa, sb.normalize_or_zero()),
            (-sa, -sb, -sa.normalize_or_zero()),
        ] {
            target.push_quad(Quad::glowing(a + o0, a + o1, b + o1, b + o0, norm, col, glow));
        }
    }

    fn emit_wire_cube(center: Vec3, half: f32, thickness: f32, col: Vec3, target: &mut dyn MeshTarget) {
        let corners = [
            Vec3::new(-half, -half, -half),
            Vec3::new(half, -half, -half),
            Vec3::new(half, -half, half),
            Vec3::new(-half, -half, half),
            Vec3::new(-half, half, -half),
            Vec3::new(half, half, -half),
            Vec3::new(half, half, half),
            Vec3::new(-half, half, half),
        ];
        for (i, j) in [
            (0, 1), (1, 2), (2, 3), (3, 0),
            (4, 5), (5, 6), (6, 7), (7, 4),
            (0, 4), (1, 5), (2, 6), (3, 7),
        ] {
            Self::emit_strut(center + corners[i], center + corners[j], thickness, col, 0.96, Vec3::Y, target);
        }
    }

    fn map_celestial_offset(&self, world_body: Vec3) -> Vec3 {
        let rel = world_body - self.ship_origin;
        let dist = rel.length();
        if dist <= 1e-4 {
            return Vec3::ZERO;
        }
        let holo_r = 0.108 * (dist / (dist + 135.0));
        (rel / dist) * holo_r
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        let (raw_fwd, raw_up) = self.axes;
        let up = raw_up.normalize_or_zero();
        let fwd = (raw_fwd - up * raw_fwd.dot(up)).normalize_or_zero();
        let right = fwd.cross(up).normalize_or_zero();
        let holo_center = self.console_base + up * 0.61 + fwd * 0.03;

        let ring_col = Vec3::new(0.14, 0.88, 0.76);
        let ring_base = self.console_base + up * 0.485 + fwd * 0.05;
        let (segs, step) = (8usize, (2.0 * PI) / 8.0);
        for i in 0..segs {
            let ((s0, c0), (s1, c1)) = (((i as f32) * step).sin_cos(), (((i + 1) as f32) * step).sin_cos());
            let p0 = ring_base + (right * c0 + fwd * s0) * 0.088;
            let p1 = ring_base + (right * c1 + fwd * s1) * 0.088;
            Self::emit_strut(p0, p1, 0.0022, ring_col, 0.90, up, target);
        }

        let earth_holo = holo_center + self.map_celestial_offset(Vec3::ZERO);
        let moon_holo = holo_center + self.map_celestial_offset(CelestialSystem::moon_center(32));

        let baseline_col = Vec3::new(0.18, 0.62, 0.78);
        Self::emit_strut(earth_holo, moon_holo, 0.0012, baseline_col, 0.85, up, target);

        Self::emit_wire_cube(earth_holo, 0.015, 0.0018, Vec3::new(0.18, 0.96, 0.74), target);
        Self::emit_wire_cube(moon_holo, 0.010, 0.0016, Vec3::new(0.86, 0.90, 0.98), target);

        let ship_col = Vec3::new(1.00, 0.82, 0.24);
        let vector_col = Vec3::new(0.98, 0.34, 0.78);
        Self::emit_wire_cube(holo_center, 0.0055, 0.0015, ship_col, target);
        Self::emit_strut(holo_center, holo_center + up * 0.044, 0.0018, vector_col, 0.98, right, target);
    }
}
