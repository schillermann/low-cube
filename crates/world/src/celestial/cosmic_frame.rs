use glam::Vec3;
use super::catalog::BodyId;

/// Polymorphic local reference frame anchoring physics to a moving Hill bubble or `4,096 m` deep-space sector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CosmicFrame {
    Body(BodyId),
    BeltAsteroid { id: BodyId, cell: (i32, i32, i32) },
    DeepSpace([i64; 3]),
}

impl CosmicFrame {
    pub const fn earth() -> Self {
        Self::Body(BodyId::EARTH)
    }

    pub const fn earth_centered(self) -> bool {
        matches!(self, Self::Body(id) if id.raw_code() == BodyId::EARTH.raw_code())
    }

    pub const fn anchored_body(self) -> Option<BodyId> {
        match self {
            Self::Body(id) | Self::BeltAsteroid { id, .. } => Some(id),
            Self::DeepSpace(_) => None,
        }
    }

    pub const fn atmospheric_sound(self) -> bool {
        match self {
            Self::Body(id) => id.atmospheric_sound(),
            Self::BeltAsteroid { .. } | Self::DeepSpace(_) => false,
        }
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        match *self {
            Self::Body(id) => {
                out.push(0);
                out.extend_from_slice(&id.raw_code().to_le_bytes());
            }
            Self::BeltAsteroid { id, cell } => {
                out.push(1);
                out.extend_from_slice(&id.raw_code().to_le_bytes());
                for c in [cell.0, cell.1, cell.2] {
                    out.extend_from_slice(&c.to_le_bytes());
                }
            }
            Self::DeepSpace(sec) => {
                out.push(2);
                for s in sec {
                    out.extend_from_slice(&s.to_le_bytes());
                }
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        match *bytes.first()? {
            0 if bytes.len() >= 3 => {
                let code = u16::from_le_bytes([bytes[1], bytes[2]]);
                Some((Self::Body(BodyId::from_raw(code)?), 3))
            }
            1 if bytes.len() >= 15 => {
                let code = u16::from_le_bytes([bytes[1], bytes[2]]);
                let id = BodyId::from_raw(code)?;
                let c0 = i32::from_le_bytes(bytes[3..7].try_into().ok()?);
                let c1 = i32::from_le_bytes(bytes[7..11].try_into().ok()?);
                let c2 = i32::from_le_bytes(bytes[11..15].try_into().ok()?);
                Some((Self::BeltAsteroid { id, cell: (c0, c1, c2) }, 15))
            }
            2 if bytes.len() >= 25 => {
                let s0 = i64::from_le_bytes(bytes[1..9].try_into().ok()?);
                let s1 = i64::from_le_bytes(bytes[9..17].try_into().ok()?);
                let s2 = i64::from_le_bytes(bytes[17..25].try_into().ok()?);
                Some((Self::DeepSpace([s0, s1, s2]), 25))
            }
            _ => None,
        }
    }
}

/// Exact `i64`-differenced (`0.1 mm`) coordinate and velocity transition across two `CosmicFrame`s (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameHandoff {
    next_frame: CosmicFrame,
    next_pos: Vec3,
    next_vel: Vec3,
    pos_shift: Vec3,
}

impl FrameHandoff {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(next_frame: CosmicFrame, next_pos: Vec3, next_vel: Vec3, pos_shift: Vec3) -> Self {
        Self { next_frame, next_pos, next_vel, pos_shift }
    }

    pub const fn next_frame(self) -> CosmicFrame { self.next_frame }
    pub const fn next_pos(self) -> Vec3 { self.next_pos }
    pub const fn next_vel(self) -> Vec3 { self.next_vel }
    pub const fn pos_shift(self) -> Vec3 { self.pos_shift }
}
