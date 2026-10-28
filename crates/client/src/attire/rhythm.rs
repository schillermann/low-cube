use glam::Vec3;

/// Polymorphic martial action style bound to the held relic or active bare-hand strike.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MartialStyle {
    Unarmed,
    BarePalmDig,
    PalmCradle,
    LumberjackCleave,
    SwordplaySlash,
    SpadeDig,
}

impl MartialStyle {
    pub const fn active_for(self, striking: bool) -> Self {
        match (self, striking) {
            (Self::Unarmed, true) => Self::BarePalmDig,
            _ => self,
        }
    }

    pub fn cleave_impulse(self, rhythm: StrideRhythm, side: f32) -> (f32, f32, f32, f32) {
        if matches!(self, Self::Unarmed) {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let arc = rhythm.cleave_arc(side);
        let surge = rhythm.cleave_surge(side);
        ((-arc).max(0.0), arc.max(0.0), (1.0 - arc * arc) * surge, surge)
    }

    pub fn external_drive(
        self,
        wind: f32,
        strike: f32,
        bow: f32,
        surge: f32,
        crouch: f32,
    ) -> (f32, f32, Vec3, f32, f32, f32) {
        let abduct = match self {
            Self::SwordplaySlash => 0.0,
            Self::SpadeDig | Self::BarePalmDig => 0.22 * wind + 0.08 * bow,
            _ => 0.36 * wind + 0.42 * bow,
        };
        let swing = match self {
            Self::SwordplaySlash => -0.18 * wind + 0.62 * strike + 0.30 * surge,
            Self::BarePalmDig => 0.42 * wind + 0.68 * bow + 0.62 * strike,
            Self::SpadeDig => 0.22 * wind + 0.42 * bow + 0.38 * strike,
            _ => 0.38 * wind + 0.55 * bow + 0.52 * strike,
        };
        let elbow_delta = match self {
            Self::BarePalmDig => Vec3::new(0.06 * wind + 0.02 * bow - 0.11 * strike, 0.0, 0.06 * wind + 0.14 * bow + 0.16 * strike),
            Self::SpadeDig => Vec3::new(0.05 * wind + 0.02 * bow - 0.04 * strike, -0.06 * wind - 0.10 * bow - 0.10 * strike, 0.06 * wind + 0.14 * bow + 0.16 * strike),
            Self::SwordplaySlash => Vec3::new(-0.02 * wind - 0.03 * strike, 0.0, 0.12 * strike + 0.08 * surge),
            _ => Vec3::new(0.04 * wind + 0.05 * bow - 0.06 * strike, 0.08 * wind, 0.08 * bow + 0.12 * strike),
        };
        let elbow_bend = match self {
            Self::SwordplaySlash => 0.35 * wind - 0.55 * strike,
            Self::BarePalmDig => 0.45 * wind - 0.12 * bow - 0.58 * strike,
            Self::SpadeDig => 0.65 * wind + 0.18 * bow - 0.36 * strike,
            _ => 0.85 * wind - 0.15 * bow - 0.52 * strike,
        };
        let wrist_dx = match self {
            Self::BarePalmDig => 0.02 * wind - 0.05 * bow - (0.075 - 0.14 * crouch) * strike,
            Self::SwordplaySlash | Self::SpadeDig => -0.02 * wind - 0.04 * strike,
            _ => 0.04 * wind + 0.10 * bow + 0.04 * strike,
        };
        let wrist_dz = match self {
            Self::SpadeDig | Self::BarePalmDig => 0.10 * bow + 0.12 * strike,
            _ => 0.0,
        };
        (abduct, swing, elbow_delta, elbow_bend, wrist_dx, wrist_dz)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn external_wrist_orient(
        self,
        side: f32,
        hold_factor: f32,
        crouch: f32,
        wind: f32,
        strike: f32,
        bow: f32,
        surge: f32,
    ) -> (f32, f32, f32, f32) {
        let (rx, ry) = match self {
            Self::SpadeDig => {
                let a = 0.75 * wind + 1.95 * bow + 2.35 * strike;
                (a.cos(), side * a.sin())
            }
            Self::PalmCradle => (0.0, -side),
            Self::BarePalmDig => {
                let a = 0.75 * wind + 1.50 * bow + 1.57 * strike;
                (a.cos(), side * a.sin())
            }
            Self::SwordplaySlash => (1.0, side * (0.10 * hold_factor + 0.16 * wind + 0.08 * strike)),
            _ => (1.0, side * (0.08 * hold_factor + 0.55 * crouch - 0.25 * wind - 0.55 * bow - 0.68 * strike)),
        };
        let wrist_yaw = match self {
            Self::BarePalmDig => side * (crouch * 0.35 * (1.0 - surge - strike).max(0.0) + 0.12 * bow + 0.72 * strike),
            _ => side * crouch * 0.35,
        };
        let wrist_snap = match self {
            Self::SwordplaySlash => -0.18 * wind + 1.12 * strike + 0.16 * surge,
            Self::PalmCradle => -0.14 + 0.36 * strike,
            Self::BarePalmDig => -0.10 * wind,
            Self::SpadeDig => -0.12 * wind + 0.48 * bow + 0.78 * strike,
            _ => -0.20 * wind + 0.35 * bow + 0.62 * strike,
        };
        (rx, ry, wrist_yaw, wrist_snap)
    }

    pub fn first_person_swing(
        self,
        wind: f32,
        strike: f32,
        bow: f32,
        surge: f32,
    ) -> (f32, f32, f32, f32, Vec3, f32) {
        let (sw_x, sw_y, sw_z, sw_yaw) = match self {
            Self::SwordplaySlash => (
                -0.015 * wind - 0.0350 * strike,
                0.14 * wind + 0.2121 * strike,
                -0.05 * wind + 0.18 * strike + 0.04 * surge,
                -0.05 * wind - 0.0221 * strike,
            ),
            Self::BarePalmDig => (
                -0.03 * wind - 0.12 * bow - 0.165 * strike,
                0.14 * wind + 0.06 * bow - 0.10 * strike,
                0.06 * wind + 0.22 * bow + 0.20 * strike,
                0.12 * wind + 0.22 * bow + 0.28 * strike,
            ),
            Self::SpadeDig => (
                -0.02 * wind - 0.08 * bow - 0.14 * strike,
                0.14 * wind + 0.06 * bow - 0.10 * strike,
                0.05 * wind + 0.20 * bow + 0.18 * strike,
                -0.06 * bow - 0.12 * strike,
            ),
            _ => (
                0.04 * wind + 0.11 * bow - 0.2307 * strike,
                0.30 * wind + 0.21 * bow + 0.1516 * strike,
                -0.03 * wind + 0.025 * bow + 0.025 * strike,
                -0.10 * wind - 0.16 * bow - 0.2044 * strike,
            ),
        };
        let elbow_add = match self {
            Self::SwordplaySlash => Vec3::new(-0.015 * wind - 0.02 * strike, 0.06 * wind - 0.02 * strike, 0.0),
            Self::SpadeDig | Self::BarePalmDig => Vec3::new(0.06 * wind + 0.02 * bow - 0.04 * strike, 0.02 * strike, 0.0),
            _ => Vec3::new(0.05 * wind + 0.06 * bow, 0.06 * wind - 0.02 * strike, 0.0),
        };
        let aim_dip = match self {
            Self::SwordplaySlash | Self::SpadeDig | Self::BarePalmDig => 0.0,
            _ => 0.14 * wind + 0.05 * bow,
        };
        (sw_x, sw_y, sw_z, sw_yaw, elbow_add, aim_dip)
    }

    pub fn first_person_wrist_orient(self, side: f32, wind: f32, strike: f32, bow: f32) -> (f32, f32, f32, f32) {
        let (rx, ry) = match self {
            Self::SpadeDig => {
                let a = 0.75 * wind + 1.95 * bow + 2.35 * strike;
                (a.cos(), side * a.sin())
            }
            Self::PalmCradle => (0.0, -side),
            Self::BarePalmDig => {
                let a = 0.75 * wind + 1.50 * bow + 1.57 * strike;
                (a.cos(), side * a.sin())
            }
            Self::SwordplaySlash => (1.0, side * (0.10 + 0.16 * wind - 0.04 * strike)),
            _ => (1.0, side * (0.08 - 0.35 * wind - 0.62 * bow - std::f32::consts::FRAC_PI_4 * strike)),
        };
        let wrist_yaw = match self {
            Self::BarePalmDig => side * (0.12 * bow + 0.72 * strike),
            _ => 0.0,
        };
        let wrist_snap = match self {
            Self::SwordplaySlash => -0.18 * wind + 1.32 * strike,
            Self::PalmCradle => -0.16 + 0.38 * strike,
            Self::SpadeDig => -0.12 * wind + 0.52 * bow + 0.85 * strike,
            Self::BarePalmDig => -0.10 * wind,
            _ => -0.22 * wind + 0.34 * bow + 0.550 * strike,
        };
        (rx, ry, wrist_yaw, wrist_snap)
    }
}

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

    fn step_cleave(c: f32, dt: f32) -> f32 {
        if c > 0.0 && c + dt * 1.65 < 1.0 { c + dt * 1.65 } else { 0.0 }
    }

    pub fn stance_bob(self, hover: f32) -> f32 {
        (2.0 * self.phase).cos() * 0.012 * self.blend * (1.0 - hover) - self.phase.sin() * 0.045 * hover
    }

    pub fn stance_twist(self, hover: f32) -> f32 {
        self.phase.sin() * (0.045 * self.blend + 0.06 * hover)
            - 0.22 * (-self.cleave_arc(1.0)).max(0.0)
            + 0.22 * (-self.cleave_arc(-1.0)).max(0.0)
    }

    pub fn limb_gait(self, side_phase_offset: f32, scale: f32) -> (f32, f32, f32) {
        let walk = (self.blend / 0.45).clamp(0.0, 1.0) * scale;
        let run = ((self.blend - 0.48) / 0.52).clamp(0.0, 1.0) * scale;
        (self.phase + side_phase_offset, walk, run)
    }

    pub fn breath_wave(self, walk: f32) -> f32 {
        self.phase.sin() * (1.0 - walk)
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

    pub fn strike_progress(self) -> f32 {
        self.right_cleave.max(self.left_cleave)
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
