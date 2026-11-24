use glam::Vec3;
use crate::relief::CraftIdentity;

const MAX_CABIN_OFFSET_METERS: f32 = 64.0;

/// Polymorphic companion deck attachment (`Planetary` world coordinates vs. `Aboard` a `CraftIdentity` hull).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VesselBerth {
    Planetary,
    Aboard(CraftIdentity),
}

impl VesselBerth {
    pub const fn planetary() -> Self {
        Self::Planetary
    }

    pub const fn aboard(craft: CraftIdentity) -> Self {
        if craft.assigned() {
            Self::Aboard(craft)
        } else {
            Self::Planetary
        }
    }

    pub const fn craft_id(self) -> Option<CraftIdentity> {
        match self {
            Self::Planetary => None,
            Self::Aboard(id) => Some(id),
        }
    }

    pub const fn aboard_any(self) -> bool {
        matches!(self, Self::Aboard(_))
    }

    pub fn valid_local_offset(self, offset: Vec3) -> bool {
        offset.is_finite() && offset.abs().max_element() <= MAX_CABIN_OFFSET_METERS
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        match *self {
            Self::Planetary => out.push(0),
            Self::Aboard(id) => {
                out.push(1);
                id.inscribe(out);
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        match *bytes.first()? {
            0 => Some((Self::Planetary, 1)),
            1 if bytes.len() >= 9 => {
                let (id, _) = CraftIdentity::decode(&bytes[1..9])?;
                Some((Self::aboard(id), 9))
            }
            _ => None,
        }
    }
}
