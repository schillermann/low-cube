use glam::Vec3;
use super::super::limb::LimbFrame;
use super::StrideRhythm;

/// Instantaneous windup, strike, bow, and surge factors of a single hand's tool motion cycle.
#[derive(Clone, Copy, Debug)]
pub struct CleaveImpulse {
    wind: f32,
    strike: f32,
    bow: f32,
    surge: f32,
}

impl CleaveImpulse {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(wind: f32, strike: f32, bow: f32, surge: f32) -> Self {
        Self { wind, strike, bow, surge }
    }

    pub const fn calm() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub const fn wind(self) -> f32 {
        self.wind
    }

    pub const fn strike(self) -> f32 {
        self.strike
    }

    pub(super) const fn factors(self) -> (f32, f32, f32, f32) {
        (self.wind, self.strike, self.bow, self.surge)
    }
}

/// Wrist orientation posture mapping forearm direction and local up-axis into a `LimbFrame`.
#[derive(Clone, Copy, Debug)]
pub struct WristPosture {
    rx: f32,
    ry: f32,
    yaw: f32,
    snap: f32,
}

impl WristPosture {
    pub const fn new(rx: f32, ry: f32, yaw: f32, snap: f32) -> Self {
        Self { rx, ry, yaw, snap }
    }

    pub fn orient_hand(self, raw: &LimbFrame, wrist_w: Vec3, forearm_dir: Vec3) -> LimbFrame {
        let hand_up = (raw.point(self.rx, self.ry, 0.0) - raw.point(0.0, 0.0, 0.0))
            .cross(forearm_dir)
            .normalize();
        LimbFrame::new(wrist_w, forearm_dir.cross(hand_up).normalize(), hand_up, forearm_dir)
            .yawed(self.yaw)
            .pitched(self.snap)
    }
}

/// Polymorphic EVA hand action style bound to the held tool, radio, block, or gloved-palm dig.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MartialStyle {
    Unarmed,
    BarePalmDig,
    PalmCradle,
    RigTorque,
    RadioSignal,
}

impl MartialStyle {
    pub const fn active_for(self, striking: bool) -> Self {
        match (self, striking) {
            (Self::Unarmed, true) => Self::BarePalmDig,
            _ => self,
        }
    }

    pub fn cleave_impulse(self, rhythm: StrideRhythm, side: f32) -> CleaveImpulse {
        if matches!(self, Self::Unarmed) {
            return CleaveImpulse::calm();
        }
        let arc = rhythm.cleave_arc(side);
        let surge = rhythm.cleave_surge(side);
        CleaveImpulse::new((-arc).max(0.0), arc.max(0.0), (1.0 - arc * arc) * surge, surge)
    }

    pub fn external_drive(self, imp: CleaveImpulse, crouch: f32) -> (f32, f32, Vec3, f32, f32, f32) {
        let (wind, strike, bow, surge) = imp.factors();
        let abduct = match self {
            Self::RadioSignal | Self::PalmCradle => 0.0,
            Self::BarePalmDig => 0.22 * wind + 0.08 * bow,
            _ => 0.36 * wind + 0.42 * bow,
        };
        let swing = match self {
            Self::RadioSignal => -0.18 * wind + 0.62 * strike + 0.30 * surge,
            Self::PalmCradle => 0.12 * bow + 0.10 * strike,
            Self::BarePalmDig => 0.42 * wind + 0.68 * bow + 0.62 * strike,
            _ => 0.38 * wind + 0.55 * bow + 0.52 * strike,
        };
        let elbow_delta = match self {
            Self::PalmCradle => Vec3::new(0.0, 0.0, 0.04 * bow + 0.04 * strike),
            Self::BarePalmDig => Vec3::new(0.06 * wind + 0.02 * bow - 0.11 * strike, 0.0, 0.06 * wind + 0.14 * bow + 0.16 * strike),
            Self::RadioSignal => Vec3::new(-0.02 * wind - 0.03 * strike, 0.0, 0.12 * strike + 0.08 * surge),
            _ => Vec3::new(0.04 * wind + 0.05 * bow - 0.06 * strike, 0.08 * wind, 0.08 * bow + 0.12 * strike),
        };
        let elbow_bend = match self {
            Self::PalmCradle => 0.10 * wind - 0.12 * strike,
            Self::RadioSignal => 0.35 * wind - 0.55 * strike,
            Self::BarePalmDig => 0.45 * wind - 0.12 * bow - 0.58 * strike,
            _ => 0.85 * wind - 0.15 * bow - 0.52 * strike,
        };
        let wrist_dx = match self {
            Self::PalmCradle => 0.0,
            Self::BarePalmDig => 0.02 * wind - 0.05 * bow - (0.075 - 0.14 * crouch) * strike,
            Self::RadioSignal => -0.02 * wind - 0.04 * strike,
            _ => 0.04 * wind + 0.10 * bow + 0.04 * strike,
        };
        let wrist_dz = match self {
            Self::PalmCradle => 0.04 * bow + 0.04 * strike,
            Self::BarePalmDig => 0.10 * bow + 0.12 * strike,
            _ => 0.0,
        };
        (abduct, swing, elbow_delta, elbow_bend, wrist_dx, wrist_dz)
    }

    pub fn external_wrist_orient(self, side: f32, hold_factor: f32, crouch: f32, imp: CleaveImpulse) -> WristPosture {
        let (wind, strike, bow, surge) = imp.factors();
        let (rx, ry) = match self {
            Self::PalmCradle => (0.0, -side),
            Self::BarePalmDig => {
                let a = 0.75 * wind + 1.50 * bow + 1.57 * strike;
                (a.cos(), side * a.sin())
            }
            Self::RadioSignal => (1.0, side * (0.10 * hold_factor + 0.16 * wind + 0.08 * strike)),
            _ => (1.0, side * (0.08 * hold_factor + 0.55 * crouch - 0.25 * wind - 0.55 * bow - 0.68 * strike)),
        };
        let wrist_yaw = match self {
            Self::PalmCradle => 0.0,
            Self::BarePalmDig => side * (crouch * 0.35 * (1.0 - surge - strike).max(0.0) + 0.12 * bow + 0.72 * strike),
            _ => side * crouch * 0.35,
        };
        let wrist_snap = match self {
            Self::RadioSignal => -0.18 * wind + 1.12 * strike + 0.16 * surge,
            Self::PalmCradle => -0.14,
            Self::BarePalmDig => -0.10 * wind,
            _ => -0.20 * wind + 0.35 * bow + 0.62 * strike,
        };
        WristPosture::new(rx, ry, wrist_yaw, wrist_snap)
    }

    pub fn first_person_swing(self, imp: CleaveImpulse) -> (f32, f32, f32, f32, Vec3, f32) {
        let (wind, strike, bow, surge) = imp.factors();
        let (sw_x, sw_y, sw_z, sw_yaw) = match self {
            Self::PalmCradle => (0.0, -0.04 * bow - 0.03 * strike, 0.05 * bow + 0.04 * strike, 0.0),
            Self::RadioSignal => (-0.015 * wind - 0.0350 * strike, 0.14 * wind + 0.2121 * strike, -0.05 * wind + 0.18 * strike + 0.04 * surge, -0.05 * wind - 0.0221 * strike),
            Self::BarePalmDig => (-0.03 * wind - 0.12 * bow - 0.165 * strike, 0.14 * wind + 0.06 * bow - 0.10 * strike, 0.06 * wind + 0.22 * bow + 0.20 * strike, 0.12 * wind + 0.22 * bow + 0.28 * strike),
            _ => (0.04 * wind + 0.11 * bow - 0.2307 * strike, 0.30 * wind + 0.21 * bow + 0.1516 * strike, -0.03 * wind + 0.025 * bow + 0.025 * strike, -0.10 * wind - 0.16 * bow - 0.2044 * strike),
        };
        let elbow_add = match self {
            Self::PalmCradle => Vec3::ZERO,
            Self::RadioSignal => Vec3::new(-0.015 * wind - 0.02 * strike, 0.06 * wind - 0.02 * strike, 0.0),
            Self::BarePalmDig => Vec3::new(0.06 * wind + 0.02 * bow - 0.04 * strike, 0.02 * strike, 0.0),
            _ => Vec3::new(0.05 * wind + 0.06 * bow, 0.06 * wind - 0.02 * strike, 0.0),
        };
        let aim_dip = match self {
            Self::PalmCradle | Self::RadioSignal | Self::BarePalmDig => 0.0,
            _ => 0.14 * wind + 0.05 * bow,
        };
        (sw_x, sw_y, sw_z, sw_yaw, elbow_add, aim_dip)
    }

    pub fn first_person_wrist_orient(self, side: f32, imp: CleaveImpulse) -> WristPosture {
        let (wind, strike, bow, _) = imp.factors();
        let (rx, ry) = match self {
            Self::PalmCradle => (0.0, -side),
            Self::BarePalmDig => {
                let a = 0.75 * wind + 1.50 * bow + 1.57 * strike;
                (a.cos(), side * a.sin())
            }
            Self::RadioSignal => (1.0, side * (0.10 + 0.16 * wind - 0.04 * strike)),
            _ => (1.0, side * (0.08 - 0.35 * wind - 0.62 * bow - std::f32::consts::FRAC_PI_4 * strike)),
        };
        let wrist_yaw = match self {
            Self::BarePalmDig => side * (0.12 * bow + 0.72 * strike),
            _ => 0.0,
        };
        let wrist_snap = match self {
            Self::RadioSignal => -0.18 * wind + 1.32 * strike,
            Self::PalmCradle => -0.16,
            Self::BarePalmDig => -0.10 * wind,
            _ => -0.22 * wind + 0.34 * bow + 0.550 * strike,
        };
        WristPosture::new(rx, ry, wrist_yaw, wrist_snap)
    }
}
