use std::f32::consts::{PI, TAU};
use glam::Vec3;

const BLACKOUT_DURATION: f32 = 2.6;
const HEARTBEAT_PERIOD: f32 = 0.85;
const SHOCK_DURATION: f32 = 1.15;
const SHOCK_BEAT_PERIOD: f32 = 0.55;
const VERTIGO_DURATION: f32 = 0.88;

/// Dying cardiac strain, tusk-wound pain pulses, vertigo sway, meal vigor, and blackout state.
#[derive(Clone, Copy, Debug)]
pub struct VitalStrain {
    cardiac_timer: f32,
    bruise: f32,
    vigor_timer: f32,
    perished: bool,
}

impl VitalStrain {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cardiac_timer: f32, bruise: f32, vigor_timer: f32, perished: bool) -> Self {
        Self { cardiac_timer, bruise, vigor_timer, perished }
    }

    pub const fn calm() -> Self {
        Self::new(0.0, 0.0, 0.0, false)
    }

    pub const fn perished(self) -> bool {
        self.perished
    }

    pub const fn shock_active(self) -> bool {
        !self.perished && self.cardiac_timer > 0.0
    }

    pub fn perish(self) -> Self {
        if self.perished {
            self
        } else {
            Self::new(0.0, 1.0, 0.0, true)
        }
    }

    pub fn gore(self, severity: f32) -> Self {
        if self.perished {
            self
        } else {
            let next = (self.bruise + severity.max(0.0)).clamp(0.0, 1.0);
            if next >= 1.0 {
                self.perish()
            } else {
                Self::new(SHOCK_DURATION, next, self.vigor_timer, false)
            }
        }
    }

    pub fn nourish(self) -> Self {
        if self.perished {
            self
        } else {
            Self::new(0.0, (self.bruise - 0.75).max(0.0), 12.0, false)
        }
    }

    pub fn step_recovery(self, dt: f32) -> (Self, bool) {
        if self.perished {
            return (self, false);
        }
        let step = dt.max(0.0);
        let next_shock = (self.cardiac_timer - step).max(0.0);
        let second_beat = self.cardiac_timer > 0.60 && next_shock <= 0.60;
        let next = Self::new(
            next_shock,
            (self.bruise - 0.015 * step).max(0.0),
            (self.vigor_timer - step).max(0.0),
            false,
        );
        (next, second_beat)
    }

    pub fn pace_boost(self) -> f32 {
        if !self.perished && self.vigor_timer > 0.0 { 1.18 } else { 1.0 }
    }

    pub fn vertigo_sway(self) -> f32 {
        if self.perished || self.cardiac_timer <= 0.0 {
            return 0.0;
        }
        let elapsed = (SHOCK_DURATION - self.cardiac_timer).max(0.0);
        if elapsed >= VERTIGO_DURATION {
            return 0.0;
        }
        let u = elapsed / VERTIGO_DURATION;
        let env = (1.0 - u).powi(2) * (u * 14.0).clamp(0.0, 1.0);
        0.19 * env * (u * TAU * 1.65).sin()
    }

    #[cfg(test)]
    pub const fn bruised(self) -> bool {
        self.bruise > 0.05
    }

    #[cfg(test)]
    pub const fn invigorated(self) -> bool {
        self.vigor_timer > 0.05
    }

    pub fn blackout_complete(self) -> bool {
        self.perished && self.cardiac_timer >= BLACKOUT_DURATION
    }

    fn perish_cycles(timer: f32) -> f32 {
        ((1.0 + 3.2 * timer.max(0.0)).sqrt() - 1.0) / 0.56
    }

    pub fn advance_perish(self, dt: f32) -> (Self, bool) {
        if !self.perished {
            return (self, false);
        }
        let next_timer = self.cardiac_timer.max(0.0) + dt.max(0.0);
        let beat = self.cardiac_timer <= 0.0
            || Self::perish_cycles(self.cardiac_timer).floor() < Self::perish_cycles(next_timer).floor();
        (Self::new(next_timer, 1.0, 0.0, true), beat)
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
        if self.perished {
            let phase = Self::perish_cycles(self.cardiac_timer).fract() * HEARTBEAT_PERIOD;
            Self::beat_envelope(phase)
        } else if self.cardiac_timer > 0.0 {
            let elapsed = (SHOCK_DURATION - self.cardiac_timer).max(0.0);
            if elapsed >= SHOCK_BEAT_PERIOD * 2.0 {
                return 0.0;
            }
            let gain = if elapsed < SHOCK_BEAT_PERIOD { 1.0 } else { 0.74 };
            gain * Self::beat_envelope(elapsed % SHOCK_BEAT_PERIOD)
        } else {
            0.0
        }
    }

    pub fn cardiac_constriction(self) -> f32 {
        if self.perished {
            let fade = (self.cardiac_timer.max(0.0) / 2.2).clamp(0.0, 1.0);
            (0.45 + 0.55 * fade).clamp(0.0, 1.0)
        } else {
            let shock = (self.cardiac_timer / SHOCK_DURATION).clamp(0.0, 1.0);
            let wound_base = self.bruise * 0.24;
            let pulse = shock * (0.22 + 0.24 * self.cardiac_throb());
            (wound_base + pulse).clamp(0.0, 0.56)
        }
    }

    pub fn cardiac_fov(self) -> f32 {
        if self.perished {
            (65.0 - 36.0 * self.cardiac_constriction() - 5.0 * self.cardiac_throb()).to_radians()
        } else {
            let shock = (self.cardiac_timer / SHOCK_DURATION).clamp(0.0, 1.0);
            (65.0 - 28.0 * self.cardiac_constriction() - 12.0 * shock * self.cardiac_throb()).to_radians()
        }
    }

    pub fn emit_reticle(self, target: &mut dyn world::MeshTarget) {
        let constrict = self.cardiac_constriction();
        if !self.perished && constrict < 0.65 {
            crate::menu::ScreenPoint::center().emit_crosshair(target);
        }
        if constrict <= 0.01 {
            return;
        }
        let throb = self.cardiac_throb();
        let blackout = if self.perished { ((constrict - 0.58) / 0.42).clamp(0.0, 1.0) } else { 0.0 };
        let soft_inner = (0.78 * (1.0 - constrict) * (1.0 - 0.08 * throb)).max(0.0);
        let (outer_r, feather, edge_alpha, min_alpha) = (
            1.50,
            0.85 + 0.35 * constrict,
            (constrict * 1.48).clamp(if self.perished { 0.18 } else { 0.0 }, 0.96),
            blackout * blackout,
        );
        let shock = if self.perished { 0.0 } else { (self.cardiac_timer / SHOCK_DURATION).clamp(0.0, 1.0) };
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
            let red = if self.perished {
                (1.0 - blackout) * (1.0 - tm * 0.75) * (0.16 + 0.12 * throb)
            } else {
                (1.0 - tm * 0.68) * (0.14 + 0.22 * throb + 0.08 * shock)
            };
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
