use std::f32::consts::TAU;
use glam::Vec3;
use world::{MeshTarget, Quad};

/// Sustained `> 4 G` spacecraft proper-acceleration vascular tunnel vision without lethal blackout.
#[derive(Clone, Copy, Debug)]
pub struct HighGStrain {
    constriction: f32,
}

impl HighGStrain {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(constriction: f32) -> Self {
        Self { constriction }
    }

    pub const fn relaxed() -> Self {
        Self::new(0.0)
    }

    pub const fn constriction(self) -> f32 {
        self.constriction
    }

    pub fn sustain(self, proper_g: f32, dt: f32) -> Self {
        let step = dt.max(0.0);
        let (target, rate) = if proper_g > 4.0 {
            (((proper_g - 4.0) / 5.0).clamp(0.0, 1.0) * 0.64, 1.65)
        } else {
            (0.0, 2.40)
        };
        let next = self.constriction + (target - self.constriction) * (rate * step).clamp(0.0, 1.0);
        Self::new(next.clamp(0.0, 0.68))
    }

    pub fn fov_narrowing_rad(self) -> f32 {
        (24.0 * self.constriction).to_radians()
    }

    pub fn emit_tunnel_vision(self, target: &mut dyn MeshTarget) {
        let c = self.constriction;
        if c <= 0.015 {
            return;
        }
        let soft_inner = (0.82 * (1.0 - c)).max(0.14);
        let (outer_r, feather, edge_alpha) = (1.50, 0.78 + 0.30 * c, (c * 1.38).clamp(0.0, 0.92));
        let alpha_at = |r: f32| {
            let u = ((r - soft_inner * 0.48) / feather).clamp(0.0, 1.0);
            (u * u * (3.0 - 2.0 * u) * edge_alpha).clamp(0.0, 1.0)
        };
        let (bands, segs) = (8usize, 28usize);
        for b in 0..bands {
            let (t0, t1) = ((b as f32) / (bands as f32), ((b + 1) as f32) / (bands as f32));
            let (r0, r1, tm) = (outer_r * t0, outer_r * t1, (t0 + t1) * 0.5);
            let (a0, a1) = (alpha_at(r0), alpha_at(r1).max(1e-4));
            let tint = (1.0 - tm * 0.72) * (0.06 + 0.10 * c);
            let col = Vec3::new(tint * 0.85, tint * 0.30, tint * 0.45);
            let grad_norm = Vec3::new(r0, (r1 - r0).max(1e-4), (a0 / a1).clamp(0.0, 1.0));
            for s in 0..segs {
                let (ang0, ang1) = ((s as f32) * TAU / (segs as f32), ((s + 1) as f32) * TAU / (segs as f32));
                let (c0, s0, c1, s1) = (ang0.cos(), ang0.sin(), ang1.cos(), ang1.sin());
                let (p0, p1) = (Vec3::new(c0 * r0, s0 * r0, 0.0), Vec3::new(c1 * r0, s1 * r0, 0.0));
                let (p2, p3) = (Vec3::new(c1 * r1, s1 * r1, 0.0), Vec3::new(c0 * r1, s0 * r1, 0.0));
                target.push_quad(Quad::glowing(p0, p1, p2, p3, grad_norm, col, -a1));
            }
        }
    }
}
