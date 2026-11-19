use world::{MeshTarget, PalmMolds, PresenceAttire};
use super::pose::BodyPose;
use super::pouch::BeltSatchel;
use super::proportion::YouthPhysique;
use super::relic::HeldRelic;
use super::sight::SightMode;
use super::YouthAttire;

/// Focused interface for avatar wardrobe appearance emission.
pub trait Wardrobe {
    fn emit(&self, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget);
}

/// Wardrobe decorator pairing `YouthAttire` with dual-palm block and field-camera orientations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoldedWardrobe {
    attire: YouthAttire,
    molds: PalmMolds,
    tray_open: bool,
}

impl MoldedWardrobe {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(attire: YouthAttire, molds: PalmMolds, tray_open: bool) -> Self {
        Self { attire, molds, tray_open }
    }

    /// Secondary constructor for closed-satchel rendering of local previews and multiplayer peers.
    pub const fn closed(attire: YouthAttire, molds: PalmMolds) -> Self {
        Self::new(attire, molds, false)
    }

    /// Secondary constructor decoding a companion's `PresenceAttire` directly without heap allocations.
    pub fn from_presence(presence: PresenceAttire) -> Self {
        presence.transfer(|garments, regions, molds| {
            let attire = YouthAttire::from_multiplayer_bytes(garments)
                .with_physique(YouthPhysique::from_region_bytes(regions));
            Self::closed(attire, molds)
        })
    }
}

impl Wardrobe for MoldedWardrobe {
    fn emit(&self, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget) {
        let active = if pose.driving_factor() > 0.5 {
            self.attire.with_relics(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm)
        } else {
            self.attire
        };
        active.emit_garments(pose, sight, self.molds, self.tray_open, target);
    }
}

/// Wardrobe decorator equipping a `BeltSatchel` and bare-hand digging posture onto `YouthAttire`.
pub struct SatchelWardrobe {
    attire: YouthAttire,
    satchel: BeltSatchel,
    dig_right: bool,
    dig_left: bool,
}

impl SatchelWardrobe {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(attire: YouthAttire, satchel: BeltSatchel, dig_right: bool, dig_left: bool) -> Self {
        Self { attire, satchel, dig_right, dig_left }
    }
}

impl Wardrobe for SatchelWardrobe {
    fn emit(&self, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget) {
        let (r, l) = self.attire.held_pair();
        let driving = pose.driving_factor() > 0.5;
        let active = self.attire.with_relics(
            if self.dig_right || driving { HeldRelic::EmptyPalm } else { r },
            if self.dig_left || driving { HeldRelic::EmptyPalm } else { l },
        );
        MoldedWardrobe::new(active, self.satchel.palm_molds(), self.satchel.open())
            .emit(pose, sight, target);
        let scaled_pose = active.scaled_pose(pose);
        self.satchel.emit_for_attire(&scaled_pose, active, target);
    }
}
