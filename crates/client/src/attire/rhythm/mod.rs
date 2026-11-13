mod martial;

use glam::Vec3;
pub use self::martial::MartialStyle;

/// Immutable gait rhythm tracking stride phase, walking-to-sprinting blend, and independent right/left hand strike cycles.
#[derive(Clone, Copy, Debug)]
pub struct StrideRhythm {
    phase: f32,
    blend: f32,
    right_cleave: f32,
    left_cleave: f32,
}

impl StrideRhythm {
    pub fn with_dual_cleave(phase: f32, blend: f32, right_cleave: f32, left_cleave: f32) -> Self {
        Self { phase, blend, right_cleave, left_cleave }
    }

    pub fn with_cleave(phase: f32, blend: f32, cleave: f32) -> Self {
        Self::with_dual_cleave(phase, blend, cleave, cleave)
    }

    pub fn new(phase: f32, blend: f32) -> Self {
        Self::with_dual_cleave(phase, blend, 0.0, 0.0)
    }

    pub const fn wire_tuple(self) -> (f32, f32, f32, f32) {
        (self.phase, self.blend, self.right_cleave, self.left_cleave)
    }

    pub fn idle() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn trigger_right_strike(self) -> Self {
        Self::with_dual_cleave(self.phase, self.blend, 0.001, self.left_cleave)
    }

    pub fn trigger_left_strike(self) -> Self {
        Self::with_dual_cleave(self.phase, self.blend, self.right_cleave, 0.001)
    }

    pub fn hand_cleave(self, side: f32) -> f32 {
        if side > 0.0 { self.right_cleave } else { self.left_cleave }
    }

    pub fn breath_wave(self, walk: f32) -> f32 {
        self.phase.sin() * (1.0 - walk)
    }

    pub fn strike_progress(self) -> f32 {
        self.right_cleave.max(self.left_cleave)
    }

    fn step_cleave(c: f32, dt: f32) -> f32 {
        if c > 0.0 && c + dt * 1.65 < 1.0 { c + dt * 1.65 } else { 0.0 }
    }

    pub fn stance_bob(self) -> f32 {
        (2.0 * self.phase).cos() * 0.012 * self.blend
    }

    pub fn stance_twist(self) -> f32 {
        self.phase.sin() * (0.045 * self.blend)
            - 0.22 * (-self.cleave_arc(1.0)).max(0.0)
            + 0.22 * (-self.cleave_arc(-1.0)).max(0.0)
    }

    pub fn limb_gait(self, side_phase_offset: f32, scale: f32) -> (f32, f32, f32) {
        let walk = (self.blend / 0.45).clamp(0.0, 1.0) * scale;
        let run = ((self.blend - 0.48) / 0.52).clamp(0.0, 1.0) * scale;
        (self.phase + side_phase_offset, walk, run)
    }

    pub fn cleave_arc(self, side: f32) -> f32 {
        let c = self.hand_cleave(side);
        if c <= 0.0 {
            0.0
        } else if c < 0.34 {
            -0.5 * (1.0 - (c / 0.34 * std::f32::consts::PI).cos())
        } else if c < 0.68 {
            -((c - 0.34) / 0.34 * std::f32::consts::PI).cos()
        } else {
            0.5 * (1.0 + ((c - 0.68) / 0.32 * std::f32::consts::PI).cos())
        }
    }

    pub fn cleave_surge(self, side: f32) -> f32 {
        let c = self.hand_cleave(side);
        if c > 0.0 { (c * std::f32::consts::PI).sin() } else { 0.0 }
    }

    pub fn advance(self, previous: Vec3, next: Vec3, terrain_up: Vec3, dt: f32) -> Self {
        let delta = next - previous;
        let tangent_dist = (delta - terrain_up * delta.dot(terrain_up)).length();
        let speed = if dt > 1e-5 { tangent_dist / dt } else { 0.0 };
        let target_blend = if speed > 6.3 {
            0.45 + ((speed - 6.3) / 3.8).clamp(0.0, 0.55)
        } else if speed > 1.2 {
            (0.38 + (speed - 2.1).max(0.0) / 3.9 * 0.07).min(0.45)
        } else {
            (speed / 1.2).clamp(0.0, 1.0) * 0.38
        };
        let step_phase = if tangent_dist > 1e-4 { tangent_dist * 2.4 } else { dt * 1.5 };
        let next_phase = (self.phase + step_phase) % std::f32::consts::TAU;
        let next_blend = self.blend + (target_blend - self.blend) * (1.0 - (-7.0 * dt).exp());
        Self::with_dual_cleave(
            next_phase,
            next_blend,
            Self::step_cleave(self.right_cleave, dt),
            Self::step_cleave(self.left_cleave, dt),
        )
    }
}
