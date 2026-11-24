use crate::relief::{CraftIdentity, CraftPose};
use crate::PlanetaryWorld;

/// Network-replicated 80-byte active vehicle pose piloted by a multiplayer companion (`O(1)` wire footprint).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrivenCraft {
    slot: u32,
    identity: CraftIdentity,
    pose: CraftPose,
}

impl DrivenCraft {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(slot: u32, identity: CraftIdentity, pose: CraftPose) -> Self {
        Self { slot, identity, pose }
    }

    pub const fn same_slot(self, slot: u32) -> bool {
        self.slot == slot
    }

    pub const fn pilot_slot(self) -> u32 {
        self.slot
    }

    pub const fn identity(self) -> CraftIdentity {
        self.identity
    }

    pub const fn pose(self) -> CraftPose {
        self.pose
    }

    pub const fn with_slot(self, slot: u32) -> Self {
        Self::new(slot, self.identity, self.pose)
    }

    pub const fn with_origin(self, origin: glam::Vec3) -> Self {
        Self::new(self.slot, self.identity, self.pose.with_origin(origin))
    }

    pub fn shifted(self, delta: glam::Vec3) -> Self {
        Self::new(self.slot, self.identity, self.pose.with_origin(self.pose.origin() + delta))
    }

    pub fn within_reach(self, anchor: glam::Vec3, max_dist: f32) -> bool {
        (anchor - self.pose.origin()).length_squared() <= max_dist * max_dist
    }

    pub fn valid_on(self, world: &dyn PlanetaryWorld) -> bool {
        let pos = self.pose.origin();
        let clamped = world.confine_beneath_crust(pos);
        self.identity.assigned() && clamped.is_finite() && (clamped - pos).length_squared() <= 64.0
    }

    pub fn interpolated(self, target: Self, alpha: f32) -> Self {
        if self.identity != target.identity {
            return target;
        }
        Self::new(target.slot, target.identity, self.pose.interpolated(target.pose, alpha))
    }

    pub fn park_on(self, world: &mut dyn PlanetaryWorld) -> bool {
        world.park_craft_by_id(self.identity, self.pose)
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.slot.to_le_bytes());
        self.identity.inscribe(out);
        self.pose.inscribe(out);
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 80 {
            return None;
        }
        let slot = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
        let (identity, _) = CraftIdentity::decode(&bytes[4..12])?;
        let (pose, _) = CraftPose::decode(&bytes[12..80])?;
        Some((Self::new(slot, identity, pose), 80))
    }
}

/// Polymorphic companion vehicle state avoiding `Option<DrivenCraft>`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PilotCraft {
    Pedestrian,
    Driving(DrivenCraft),
}

impl PilotCraft {
    pub const fn with_slot(self, slot: u32) -> Self {
        match self {
            Self::Pedestrian => Self::Pedestrian,
            Self::Driving(craft) => Self::Driving(craft.with_slot(slot)),
        }
    }

    pub fn shifted(self, delta: glam::Vec3) -> Self {
        match self {
            Self::Pedestrian => Self::Pedestrian,
            Self::Driving(craft) => Self::Driving(craft.shifted(delta)),
        }
    }

    pub fn reconciled(self, slot: u32, incoming: Self, world: &dyn PlanetaryWorld) -> Self {
        match incoming {
            Self::Pedestrian => Self::Pedestrian,
            Self::Driving(craft) => {
                if craft.valid_on(world) {
                    Self::Driving(craft.with_slot(slot))
                } else {
                    self.with_slot(slot)
                }
            }
        }
    }

    pub fn interpolated(self, target: Self, alpha: f32) -> Self {
        match (self, target) {
            (Self::Driving(a), Self::Driving(b)) => Self::Driving(a.interpolated(b, alpha)),
            (_, Self::Driving(b)) => Self::Driving(b),
            (_, Self::Pedestrian) => Self::Pedestrian,
        }
    }

    pub fn push_active(self, out: &mut Vec<DrivenCraft>) {
        if let Self::Driving(craft) = self {
            out.push(craft);
        }
    }

    pub fn park_if_driving(self, world: &mut dyn PlanetaryWorld) -> bool {
        match self {
            Self::Pedestrian => false,
            Self::Driving(craft) => craft.park_on(world),
        }
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        match self {
            Self::Pedestrian => out.push(0),
            Self::Driving(craft) => {
                out.push(1);
                craft.inscribe(out);
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        let tag = *bytes.first()?;
        match tag {
            0 => Some((Self::Pedestrian, 1)),
            1 => {
                let (craft, used) = DrivenCraft::decode(&bytes[1..])?;
                Some((Self::Driving(craft), 1 + used))
            }
            _ => None,
        }
    }
}
