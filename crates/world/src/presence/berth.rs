use glam::Vec3;
use crate::relief::CraftIdentity;

const MAX_CABIN_OFFSET_METERS: f32 = 256.0;

/// Polymorphic companion deck attachment (`Planetary`, `Aboard` a `GridBody`, or `Nested` across docked `GridBody`s).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VesselBerth {
    Planetary,
    Aboard(CraftIdentity),
    Nested { body: CraftIdentity, root_host: CraftIdentity },
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

    pub fn nested(body: CraftIdentity, root_host: CraftIdentity) -> Self {
        if !body.assigned() {
            Self::Planetary
        } else if !root_host.assigned() || body == root_host {
            Self::Aboard(body)
        } else {
            Self::Nested { body, root_host }
        }
    }

    pub const fn craft_id(self) -> Option<CraftIdentity> {
        match self {
            Self::Planetary => None,
            Self::Aboard(id) | Self::Nested { body: id, .. } => Some(id),
        }
    }

    pub const fn root_host_id(self) -> Option<CraftIdentity> {
        match self {
            Self::Planetary => None,
            Self::Aboard(id) => Some(id),
            Self::Nested { root_host, .. } => Some(root_host),
        }
    }

    pub const fn aboard_any(self) -> bool {
        !matches!(self, Self::Planetary)
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
            Self::Nested { body, root_host } => {
                out.push(2);
                body.inscribe(out);
                root_host.inscribe(out);
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
            2 if bytes.len() >= 17 => {
                let (body, _) = CraftIdentity::decode(&bytes[1..9])?;
                let (root, _) = CraftIdentity::decode(&bytes[9..17])?;
                Some((Self::nested(body, root), 17))
            }
            _ => None,
        }
    }
}
