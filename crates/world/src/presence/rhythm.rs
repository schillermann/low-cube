use std::f32::consts::{PI, TAU};

/// Immutable gait rhythm snapshot interpolating circular stride phase and forward swing arcs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresenceRhythm {
    phase: f32,
    blend: f32,
    right_cleave: f32,
    left_cleave: f32,
}

impl PresenceRhythm {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(phase: f32, blend: f32, right_cleave: f32, left_cleave: f32) -> Self {
        Self { phase, blend, right_cleave, left_cleave }
    }

    pub const fn from_wire(raw: (f32, f32, f32, f32)) -> Self {
        Self::new(raw.0, raw.1, raw.2, raw.3)
    }

    pub const fn wire_tuple(self) -> (f32, f32, f32, f32) {
        (self.phase, self.blend, self.right_cleave, self.left_cleave)
    }

    fn blend_cleave(from: f32, target: f32, t: f32) -> f32 {
        if target <= 0.0 && from > 0.5 {
            let next = from + (1.0 - from) * t;
            if next >= 0.999 { 0.0 } else { next }
        } else {
            from + (target - from) * t
        }
    }

    pub fn interpolated(self, target: Self, alpha: f32) -> Self {
        let t = alpha.clamp(0.0, 1.0);
        let d_phase = (target.phase - self.phase + PI).rem_euclid(TAU) - PI;
        let phase = (self.phase + d_phase * t).rem_euclid(TAU);
        let blend = self.blend + (target.blend - self.blend) * t;
        let right = Self::blend_cleave(self.right_cleave, target.right_cleave, t);
        let left = Self::blend_cleave(self.left_cleave, target.left_cleave, t);
        Self::new(phase, blend, right, left)
    }
}
