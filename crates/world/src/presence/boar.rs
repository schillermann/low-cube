use std::f32::consts::{PI, TAU};
use glam::Vec3;
use crate::wire_vector::{WireSlice, WireVector};

const BOAR_HIT_RADIUS: f32 = 0.56;

/// Polymorphic behavioral stance of a network-replicated European wild boar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoarStance {
    Rooting,
    Bristled,
    Charging,
    Trotting,
    Fallen,
}

impl BoarStance {
    pub const fn code(self) -> u8 {
        match self {
            Self::Rooting => 0,
            Self::Bristled => 1,
            Self::Charging => 2,
            Self::Trotting => 3,
            Self::Fallen => 4,
        }
    }

    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Rooting),
            1 => Some(Self::Bristled),
            2 => Some(Self::Charging),
            3 => Some(Self::Trotting),
            4 => Some(Self::Fallen),
            _ => None,
        }
    }

    pub const fn alive(self) -> bool {
        !matches!(self, Self::Fallen)
    }

    pub const fn vocal(self) -> bool {
        matches!(self, Self::Bristled | Self::Charging)
    }
}

/// Network-replicated 3D kinematics, stride phase, hit flash, and stance of a wild boar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoarPresence {
    position: Vec3,
    heading: Vec3,
    phase_and_flash: (f32, f32),
    stance: BoarStance,
}

impl BoarPresence {
    pub const WIRE_SPAN: usize = 30;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(position: Vec3, heading: Vec3, phase_and_flash: (f32, f32), stance: BoarStance) -> Self {
        Self { position, heading, phase_and_flash, stance }
    }

    pub const fn position(self) -> Vec3 { self.position }
    pub const fn stance(self) -> BoarStance { self.stance }
    pub const fn alive(self) -> bool { self.stance.alive() }

    pub const fn with_flash(self, flash: f32) -> Self {
        let next = if flash > self.phase_and_flash.1 { flash } else { self.phase_and_flash.1 };
        Self::new(self.position, self.heading, (self.phase_and_flash.0, next), self.stance)
    }

    pub fn intercept_ray(self, up: Vec3, eye: Vec3, dir: Vec3, max_dist: f32) -> Option<f32> {
        if !self.alive() {
            return None;
        }
        let ray = dir.normalize_or_zero();
        let center = self.position + up * 0.48;
        let oc = eye - center;
        let b = oc.dot(ray);
        let c = oc.length_squared() - BOAR_HIT_RADIUS * BOAR_HIT_RADIUS;
        let disc = b * b - c;
        if disc < 0.0 {
            return None;
        }
        let t = (-b - disc.sqrt()).max(0.0);
        (t <= max_dist && (oc + ray * t).length_squared() <= (BOAR_HIT_RADIUS + 0.02).powi(2)).then_some(t)
    }

    pub fn interpolated(self, target: Self, alpha: f32) -> Self {
        let t = alpha.clamp(0.0, 1.0);
        if !self.alive() || !target.alive() || (target.position - self.position).length_squared() > 36.0 {
            return target;
        }
        let pos = self.position.lerp(target.position, t);
        let blended_h = self.heading.lerp(target.heading, t).normalize_or_zero();
        let heading = if blended_h.length_squared() > 0.2 { blended_h } else { target.heading };
        let d_phase = (target.phase_and_flash.0 - self.phase_and_flash.0 + PI).rem_euclid(TAU) - PI;
        let phase = (self.phase_and_flash.0 + d_phase * t).rem_euclid(TAU);
        let flash = self.phase_and_flash.1 + (target.phase_and_flash.1 - self.phase_and_flash.1) * t;
        Self::new(pos, heading, (phase, flash), target.stance)
    }

    pub fn transfer<T>(self, f: impl FnOnce(Vec3, Vec3, f32, f32, BoarStance) -> T) -> T {
        f(self.position, self.heading, self.phase_and_flash.0, self.phase_and_flash.1, self.stance)
    }

    pub fn inscribe(self, out: &mut Vec<u8>) {
        WireVector::new(self.position).inscribe(out);
        WireVector::new(self.heading).inscribe(out);
        out.extend_from_slice(&self.phase_and_flash.0.to_le_bytes());
        let flash_byte = (self.phase_and_flash.1.clamp(0.0, 1.0) * 14.0).round() as u8;
        out.push(flash_byte);
        out.push(self.stance.code());
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < Self::WIRE_SPAN {
            return None;
        }
        let pos = WireSlice::new(&bytes[0..12]).vec3()?;
        let heading = WireSlice::new(&bytes[12..24]).vec3()?;
        let phase = f32::from_le_bytes(bytes[24..28].try_into().ok()?);
        if !phase.is_finite() {
            return None;
        }
        let flash = (f32::from(bytes[28].min(14))) / 14.0;
        let stance = BoarStance::from_code(bytes[29])?;
        Some((Self::new(pos, heading, (phase, flash), stance), Self::WIRE_SPAN))
    }
}

/// Polymorphic network snapshot of the 3-boar Temperate sounder without boolean flags.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SounderPresence {
    Unseen,
    Sighted([BoarPresence; 3]),
}

impl SounderPresence {
    pub fn interpolated(self, target: Self, alpha: f32) -> Self {
        match (self, target) {
            (Self::Sighted(a), Self::Sighted(b)) => Self::Sighted([
                a[0].interpolated(b[0], alpha),
                a[1].interpolated(b[1], alpha),
                a[2].interpolated(b[2], alpha),
            ]),
            (_, next) => next,
        }
    }

    pub fn inscribe(self, out: &mut Vec<u8>) {
        match self {
            Self::Unseen => out.push(0),
            Self::Sighted(boars) => {
                out.push(1);
                for boar in boars {
                    boar.inscribe(out);
                }
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        match *bytes.first()? {
            0 => Some((Self::Unseen, 1)),
            1 if bytes.len() >= 1 + 3 * BoarPresence::WIRE_SPAN => {
                let (b0, _) = BoarPresence::decode(&bytes[1..31])?;
                let (b1, _) = BoarPresence::decode(&bytes[31..61])?;
                let (b2, _) = BoarPresence::decode(&bytes[61..91])?;
                Some((Self::Sighted([b0, b1, b2]), 91))
            }
            _ => None,
        }
    }
}

/// Loss-tolerant monotonic tusk-impact counter and knockback impulse for a companion slot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GoreStamp {
    seq: u16,
    impulse: Vec3,
}

impl GoreStamp {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(seq: u16, impulse: Vec3) -> Self {
        Self { seq, impulse }
    }

    pub const fn calm() -> Self {
        Self::new(0, Vec3::ZERO)
    }

    pub const fn advanced(self, impulse: Vec3) -> Self {
        let next = self.seq.wrapping_add(1);
        Self::new(if next == 0 { 1 } else { next }, impulse)
    }

    pub fn absorb_into(self, seen_seq: &mut u16) -> Option<Vec3> {
        if self.seq != 0 && self.seq != *seen_seq {
            *seen_seq = self.seq;
            Some(self.impulse)
        } else {
            None
        }
    }

    pub fn inscribe(self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.seq.to_le_bytes());
        if self.seq != 0 {
            WireVector::new(self.impulse).inscribe(out);
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 2 {
            return None;
        }
        let seq = u16::from_le_bytes([bytes[0], bytes[1]]);
        if seq == 0 {
            Some((Self::calm(), 2))
        } else if bytes.len() >= 14 {
            let impulse = WireSlice::new(&bytes[2..14]).vec3()?;
            Some((Self::new(seq, impulse), 14))
        } else {
            None
        }
    }
}

/// Combined authoritative wild-boar sounder snapshot and per-recipient tusk knockback stamp.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WildlifePulse {
    sounder: SounderPresence,
    gore: GoreStamp,
}

impl WildlifePulse {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sounder: SounderPresence, gore: GoreStamp) -> Self {
        Self { sounder, gore }
    }

    pub const fn calm() -> Self {
        Self::new(SounderPresence::Unseen, GoreStamp::calm())
    }

    pub const fn sounder(self) -> SounderPresence { self.sounder }
    pub const fn gore(self) -> GoreStamp { self.gore }

    pub fn inscribe(self, out: &mut Vec<u8>) {
        self.gore.inscribe(out);
        self.sounder.inscribe(out);
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        let (gore, g_used) = GoreStamp::decode(bytes)?;
        let (sounder, s_used) = SounderPresence::decode(&bytes[g_used..])?;
        Some((Self::new(sounder, gore), g_used + s_used))
    }
}
