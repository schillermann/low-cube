use glam::Vec3;
use crate::MeshTarget;

/// Encapsulated geometric quad entity with vertices, normal, and surface color.
#[derive(Clone, Copy, Debug)]
pub struct Quad {
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    normal: Vec3,
    color: Vec3,
}

impl Quad {
    /// Primary constructor: assigns fields directly without computation.
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, normal: Vec3, color: Vec3) -> Self {
        Self { v0, v1, v2, v3, normal, color }
    }

    /// Feeds its encapsulated data to a target without exposing raw getters.
    pub fn transfer<T>(&self, consumer: impl FnOnce(Vec3, Vec3, Vec3, Vec3, Vec3, Vec3) -> T) -> T {
        consumer(self.v0, self.v1, self.v2, self.v3, self.normal, self.color)
    }

    /// Emits this quad directly when stage == 0, or subdivides it into a recessed 3D digging crater for stages 1..=3.
    pub fn emit_crater(&self, stage: u8, target: &mut dyn MeshTarget) {
        if stage == 0 {
            target.push_quad(*self);
            return;
        }
        let bilerp = |fu: f32, fv: f32| {
            self.v0 * ((1.0 - fu) * (1.0 - fv))
                + self.v1 * (fu * (1.0 - fv))
                + self.v2 * (fu * fv)
                + self.v3 * ((1.0 - fu) * fv)
        };
        let s = stage.min(3) as f32;
        let (r0, depth) = (0.40 - s * 0.10, s * 0.09);
        let (r1, f0, f1, drop) = (1.0 - r0, r0 + 0.08, 0.92 - r0, self.normal * depth);
        let o = [self.v0, self.v1, self.v2, self.v3];
        let m = [bilerp(r0, r0), bilerp(r1, r0), bilerp(r1, r1), bilerp(r0, r1)];
        let b = [
            bilerp(f0, f0) - drop,
            bilerp(f1, f0) - drop,
            bilerp(f1, f1) - drop,
            bilerp(f0, f1) - drop,
        ];
        let (wall_col, floor_col) = (
            (self.color * 0.72).clamp(Vec3::ZERO, Vec3::ONE),
            (self.color * 0.52).clamp(Vec3::ZERO, Vec3::ONE),
        );
        for i in 0..4 {
            let j = (i + 1) & 3;
            target.push_quad(Self::new(o[i], o[j], m[j], m[i], self.normal, self.color));
            let wn = (b[j] - b[i]).cross(m[i] - b[i]).normalize_or_zero();
            target.push_quad(Self::new(
                b[i],
                b[j],
                m[j],
                m[i],
                if wn.length_squared() > 0.5 { wn } else { self.normal },
                wall_col,
            ));
        }
        target.push_quad(Self::new(b[0], b[1], b[2], b[3], self.normal, floor_col));
    }
}
