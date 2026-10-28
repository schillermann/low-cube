use winit::keyboard::KeyCode;
use world::MeshTarget;

mod accoutrement;
mod crown;
mod file_attire;
mod footwear;
mod garments;
mod hands;
mod headgear;
mod limb;
mod palette;
mod pose;
mod pouch;
mod proportion;
mod relic;
mod rhythm;
mod sight;
mod tome;
#[cfg(test)]
mod tests;

pub use crown::CrownStyle;
pub use file_attire::{AttireProfile, FileAttire};
pub use footwear::FootwearStyle;
pub use garments::{GirdleStyle, TunicStyle};
pub use hands::VambraceStyle;
pub use limb::{FacetedSolid, LimbFrame, TaperedPrism};
pub use palette::{ComplexionSwatch, PigmentSwatch};
pub use pose::BodyPose;
pub use pouch::BeltSatchel;
pub use proportion::{BodyRegion, PartProportion, YouthPhysique};
pub use relic::{HeldRelic, MartialStyle};
pub use rhythm::StrideRhythm;
pub use sight::SightMode;
pub use tome::ArtisanTome;

/// Interface for avatar wardrobe appearance emission.
pub trait Wardrobe {
    fn emit(&self, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GarmentEnsemble {
    tunic: TunicStyle,
    vambrace: VambraceStyle,
    girdle: GirdleStyle,
    footwear: FootwearStyle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IdentityTraits {
    crown: CrownStyle,
    pigment: PigmentSwatch,
    complexion: ComplexionSwatch,
    physique: YouthPhysique,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HeldHands {
    relic: HeldRelic,
    offhand: HeldRelic,
}

/// Immutable medieval youth character appearance combining body traits, garments, dyes, and dual-hand held relics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YouthAttire {
    garments: GarmentEnsemble,
    identity: IdentityTraits,
    hands: HeldHands,
}

impl YouthAttire {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        tunic: TunicStyle,
        vambrace: VambraceStyle,
        girdle: GirdleStyle,
        footwear: FootwearStyle,
        crown: CrownStyle,
        relic: HeldRelic,
        offhand: HeldRelic,
        pigment: PigmentSwatch,
        complexion: ComplexionSwatch,
        physique: YouthPhysique,
    ) -> Self {
        Self {
            garments: GarmentEnsemble { tunic, vambrace, girdle, footwear },
            identity: IdentityTraits { crown, pigment, complexion, physique },
            hands: HeldHands { relic, offhand },
        }
    }

    pub const fn squire() -> Self {
        Self::new(
            TunicStyle::LinenDoublet,
            VambraceStyle::RawhideWraps,
            GirdleStyle::KnottedCord,
            FootwearStyle::CuffedBoots,
            CrownStyle::TousledMop,
            HeldRelic::EmptyPalm,
            HeldRelic::EmptyPalm,
            PigmentSwatch::UnbleachedLinen,
            ComplexionSwatch::Birch,
            YouthPhysique::SlenderSquire,
        )
    }

    pub fn with_next_tunic(mut self) -> Self { self.garments.tunic = self.garments.tunic.next(); self }
    pub fn with_next_vambrace(mut self) -> Self { self.garments.vambrace = self.garments.vambrace.next(); self }
    pub fn with_next_girdle(mut self) -> Self { self.garments.girdle = self.garments.girdle.next(); self }
    pub fn with_next_footwear(mut self) -> Self { self.garments.footwear = self.garments.footwear.next(); self }
    pub fn with_next_crown(mut self) -> Self { self.identity.crown = self.identity.crown.next(); self }
    #[cfg(test)]
    pub fn with_next_relic(mut self) -> Self { self.hands.relic = self.hands.relic.next(); self }
    pub fn with_relics(mut self, relic: HeldRelic, offhand: HeldRelic) -> Self { self.hands = HeldHands { relic, offhand }; self }
    pub fn with_complexion(mut self, complexion: ComplexionSwatch) -> Self { self.identity.complexion = complexion; self }
    pub fn with_physique(mut self, physique: YouthPhysique) -> Self { self.identity.physique = physique; self }
    pub fn with_feminine(mut self, feminine: bool) -> Self { self.identity.physique = self.identity.physique.as_feminine(feminine); self }
    pub fn feminine(&self) -> bool { self.identity.physique.feminine() }

    pub fn with_adjusted_region(mut self, region: BodyRegion, delta_size: i8, delta_girth: i8) -> Self {
        self.identity.physique = self.identity.physique.adjusted(region, delta_size, delta_girth);
        self
    }

    pub fn region_proportion(&self, region: BodyRegion) -> PartProportion { self.identity.physique.region_proportion(region) }
    pub fn shaft_yaw_play(&self) -> f32 { self.identity.physique.shaft_yaw_play() }
    pub fn body_envelope(&self) -> crate::envelope::BodyEnvelope { crate::envelope::BodyEnvelope::for_physique(self.identity.physique) }
    pub fn region_bytes(&self) -> [u8; 4] { self.identity.physique.region_bytes() }
    pub fn selected_complexion(&self, candidate: ComplexionSwatch) -> bool { self.identity.complexion == candidate }
    pub fn right_armed(&self) -> bool { !matches!(self.hands.relic.martial_style(), MartialStyle::Unarmed) }
    pub fn left_armed(&self) -> bool { !matches!(self.hands.offhand.martial_style(), MartialStyle::Unarmed) }
    pub fn armed(&self) -> bool { self.right_armed() || self.left_armed() }
    pub fn held_pair(&self) -> (HeldRelic, HeldRelic) { (self.hands.relic, self.hands.offhand) }
    pub fn with_next_pigment(mut self) -> Self { self.identity.pigment = self.identity.pigment.next(); self }

    pub fn with_next_complexion(mut self) -> Self {
        self.identity.complexion = self.identity.complexion.next();
        self.identity.physique = self.identity.physique.next();
        self
    }

    pub fn with_wardrobe_key(self, code: KeyCode) -> Option<Self> {
        let updated = match code {
            KeyCode::Digit1 => self.with_next_tunic(),
            KeyCode::Digit2 => self.with_next_vambrace(),
            KeyCode::Digit3 => self.with_next_girdle(),
            KeyCode::Digit4 => self.with_next_footwear(),
            KeyCode::Digit5 => self.with_next_crown(),
            KeyCode::Digit6 => self.with_next_pigment(),
            KeyCode::Digit7 => self.with_next_complexion(),
            KeyCode::KeyF => self.with_feminine(!self.feminine()),
            _ => return None,
        };
        Some(updated)
    }

    pub fn cycled_by_key(self, code: KeyCode) -> Option<Self> {
        let updated = self.with_wardrobe_key(code)?;
        FileAttire::profile().save(&updated);
        Some(updated)
    }

    pub fn multiplayer_bytes(&self) -> [u8; 9] {
        [
            self.garments.tunic.code(),
            self.garments.vambrace.code(),
            self.garments.girdle.code(),
            self.garments.footwear.code(),
            self.identity.crown.code(),
            self.hands.relic.code() + self.hands.offhand.code() * 16,
            self.identity.pigment.code(),
            self.identity.complexion.code(),
            self.identity.physique.code(),
        ]
    }

    #[cfg(test)]
    pub fn emit_peer(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.emit(pose, SightMode::LookingGlassSight, target);
    }

    pub fn emit_satchel(&self, pose: &BodyPose, sight: SightMode, satchel: BeltSatchel, target: &mut dyn MeshTarget) {
        let scaled_pose = pose.with_physique(self.identity.physique);
        self.garments.tunic.emit(&scaled_pose, self.identity.pigment, target);
        self.garments.girdle.emit(&scaled_pose, target);
        self.garments.footwear.emit(&scaled_pose, self.identity.pigment, target);
        self.garments.vambrace.emit(
            &scaled_pose,
            sight,
            self.identity.complexion,
            self.identity.pigment,
            (self.hands.relic, self.hands.offhand),
            satchel,
            target,
        );
        satchel.emit_on_waist(&scaled_pose, target);
        sight.emit_crown(self.identity.crown, &scaled_pose, self.identity.complexion, self.identity.pigment, target);
    }
}

impl Wardrobe for YouthAttire {
    fn emit(&self, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget) {
        self.emit_satchel(pose, sight, BeltSatchel::empty(), target);
    }
}

/// Wardrobe decorator equipping a BeltSatchel and bare-hand digging posture onto YouthAttire.
pub struct SatchelWardrobe {
    attire: YouthAttire,
    satchel: BeltSatchel,
    dig_right: bool,
    dig_left: bool,
}

impl SatchelWardrobe {
    pub const fn new(attire: YouthAttire, satchel: BeltSatchel, dig_right: bool, dig_left: bool) -> Self {
        Self { attire, satchel, dig_right, dig_left }
    }
}

impl Wardrobe for SatchelWardrobe {
    fn emit(&self, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget) {
        let (r, l) = self.attire.held_pair();
        let active = self.attire.with_relics(
            if self.dig_right { HeldRelic::EmptyPalm } else { r },
            if self.dig_left { HeldRelic::EmptyPalm } else { l },
        );
        active.emit_satchel(pose, sight, self.satchel, target);
    }
}
