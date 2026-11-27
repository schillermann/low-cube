use world::{MeshTarget, PalmMolds, PresenceAttire};
use super::pose::BodyPose;
use super::proportion::CrewPhysique;
use super::relic::HeldRelic;
use super::sight::SightMode;
use super::EvaAttire;

/// Focused interface for crew EVA wardrobe appearance emission.
pub trait Wardrobe {
    fn emit(&self, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget);
}

/// Wardrobe decorator pairing `EvaAttire` with dual-palm block and field-camera orientations (`3` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoldedWardrobe {
    attire: EvaAttire,
    molds: PalmMolds,
    tray_open: bool,
}

impl MoldedWardrobe {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(attire: EvaAttire, molds: PalmMolds, tray_open: bool) -> Self {
        Self { attire, molds, tray_open }
    }

    /// Secondary constructor for closed-satchel rendering of local previews and multiplayer peers.
    pub const fn closed(attire: EvaAttire, molds: PalmMolds) -> Self {
        Self::new(attire, molds, false)
    }

    /// Secondary constructor decoding a companion's `PresenceAttire` directly without heap allocations.
    pub fn from_presence(presence: PresenceAttire) -> Self {
        presence.transfer(|garments, regions, molds| {
            let attire = EvaAttire::from_multiplayer_bytes(garments)
                .with_physique(CrewPhysique::from_region_bytes(regions));
            Self::closed(attire, molds)
        })
    }
}

impl Wardrobe for MoldedWardrobe {
    fn emit(&self, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget) {
        let active = if pose.driving_factor() > 0.5 && !self.tray_open {
            self.attire.with_relics(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm)
        } else {
            self.attire
        };
        active.emit_suit(pose, sight, self.molds, self.tray_open, target);
    }
}
