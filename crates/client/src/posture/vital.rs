use std::f32::consts::{PI, TAU};
use glam::Vec3;

const BLACKOUT_DURATION: f32 = 2.6;
const HEARTBEAT_PERIOD: f32 = 0.85;

/// Polymorphic crew vital state avoiding `perished: bool` flags and dual-purpose timer fields.
#[derive(Clone, Copy, Debug)]
pub enum VitalStrain {
    Breathing,
    Perishing {
        elapsed: f32,
    },
}

impl VitalStrain {
    pub const fn calm() -> Self {
        Self::Breathing
    }

    pub const fn perished(self) -> bool {
        matches!(self, Self::Perishing { .. })
    }

    pub const fn perish(self) -> Self {
        match self {
            Self::Perishing { .. } => self,
            Self::Breathing => Self::Perishing { elapsed: 0.0 },
        }
    }

    pub const fn blackout_complete(self) -> bool {
        matches!(self, Self::Perishing { elapsed } if elapsed >= BLACKOUT_DURATION)
    }

    fn perish_cycles(timer: f32) -> f32 {
        ((1.0 + 3.2 * timer.max(0.0)).sqrt() - 1.0) / 0.56
    }

    pub fn advance_perish(self, dt: f32) -> (Self, bool) {
        let Self::Perishing { elapsed } = self else {
            return (self, false);
        };
        let next_elapsed = elapsed.max(0.0) + dt.max(0.0);
        let beat = elapsed <= 0.0
            || Self::perish_cycles(elapsed).floor() < Self::perish_cycles(next_elapsed).floor();
        (Self::Perishing { elapsed: next_elapsed }, beat)
    }

    fn beat_envelope(phase: f32) -> f32 {
        if phase < 0.16 {
            (phase / 0.16 * PI).sin().powi(2)
        } else if (0.20..0.36).contains(&phase) {
            0.78 * ((phase - 0.20) / 0.16 * PI).sin().powi(2)
        } else {
            0.0
        }
    }

    pub fn cardiac_throb(self) -> f32 {
        match self {
            Self::Perishing { elapsed } => {
                let phase = Self::perish_cycles(elapsed).fract() * HEARTBEAT_PERIOD;
                Self::beat_envelope(phase)
            }
            Self::Breathing => 0.0,
        }
    }

    pub fn cardiac_constriction(self) -> f32 {
        match self {
            Self::Perishing { elapsed } => {
                let fade = (elapsed.max(0.0) / 2.2).clamp(0.0, 1.0);
                (0.45 + 0.55 * fade).clamp(0.0, 1.0)
            }
            Self::Breathing => 0.0,
        }
    }

    pub fn cardiac_fov(self) -> f32 {
        match self {
            Self::Perishing { .. } => {
                (65.0 - 36.0 * self.cardiac_constriction() - 5.0 * self.cardiac_throb()).to_radians()
            }
            Self::Breathing => 65.0_f32.to_radians(),
        }
    }

    pub fn emit_reticle_at(self, crosshair: crate::menu::ScreenPoint, target: &mut dyn world::MeshTarget) {
        let constrict = self.cardiac_constriction();
        let dying = self.perished();
        if !dying && constrict < 0.65 {
            crosshair.emit_crosshair(target);
        }
        if constrict <= 0.01 {
            return;
        }
        let throb = self.cardiac_throb();
        let blackout = if dying { ((constrict - 0.58) / 0.42).clamp(0.0, 1.0) } else { 0.0 };
        let soft_inner = (0.78 * (1.0 - constrict) * (1.0 - 0.08 * throb)).max(0.0);
        let (outer_r, feather, edge_alpha, min_alpha) = (
            1.50,
            0.85 + 0.35 * constrict,
            (constrict * 1.48).clamp(if dying { 0.18 } else { 0.0 }, 0.96),
            blackout * blackout,
        );
        let alpha_at = |r: f32| {
            let u = ((r - soft_inner * 0.45) / feather).clamp(0.0, 1.0);
            let curve = u * u * (3.0 - 2.0 * u);
            (min_alpha + (1.0 - min_alpha) * curve * edge_alpha).clamp(0.0, 1.0)
        };
        let (bands, segs) = (10usize, 32usize);
        for b in 0..bands {
            let (t0, t1) = ((b as f32) / (bands as f32), ((b + 1) as f32) / (bands as f32));
            let (r0, r1, tm) = (outer_r * t0, outer_r * t1, (t0 + t1) * 0.5);
            let (a0, a1) = (alpha_at(r0), alpha_at(r1).max(1e-4));
            let red = (1.0 - blackout) * (1.0 - tm * 0.75) * (0.16 + 0.12 * throb);
            let (col, grad_norm) = (Vec3::new(red, 0.0, 0.0), Vec3::new(r0, (r1 - r0).max(1e-4), (a0 / a1).clamp(0.0, 1.0)));
            for s in 0..segs {
                let (ang0, ang1) = ((s as f32) * TAU / (segs as f32), ((s + 1) as f32) * TAU / (segs as f32));
                let (c0, s0, c1, s1) = (ang0.cos(), ang0.sin(), ang1.cos(), ang1.sin());
                let (p0, p1) = (Vec3::new(c0 * r0, s0 * r0, 0.0), Vec3::new(c1 * r0, s1 * r0, 0.0));
                let (p2, p3) = (Vec3::new(c1 * r1, s1 * r1, 0.0), Vec3::new(c0 * r1, s0 * r1, 0.0));
                target.push_quad(world::Quad::glowing(p0, p1, p2, p3, grad_norm, col, -a1));
            }
        }
    }
}
