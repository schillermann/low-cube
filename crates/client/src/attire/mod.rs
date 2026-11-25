use winit::keyboard::KeyCode;
use world::{MeshTarget, PalmMolds};

mod accoutrement;
mod beard;
mod burden;
mod coiffure;
mod compartment;
mod crown;
mod file_attire;
mod footwear;
mod garments;
mod hands;
mod headgear;
mod intercom;
mod limb;
mod palette;
mod pose;
mod pouch;
mod proportion;
mod relic;
mod rhythm;
mod sack;
mod sight;
mod tome;
mod tray;
mod visage;
mod wardrobe;
#[cfg(test)]
mod tests;

pub use burden::SatchelBurden;
pub use crown::CrownStyle;
pub use file_attire::{AttireProfile, FileAttire};
pub use footwear::FootwearStyle;
pub use garments::{GirdleStyle, TunicStyle};
pub use hands::{HeldPalms, VambraceStyle};
pub use intercom::IntercomBust;
pub use limb::{FacetedSolid, LimbFrame, TaperedPrism};
pub use palette::{ComplexionSwatch, PigmentSwatch};
pub use pose::{BodyPose, RocketControls};
pub use pouch::BeltSatchel;
pub use proportion::{BodyRegion, PartProportion, YouthPhysique};
pub use relic::{HeldRelic, MartialStyle};
pub use rhythm::StrideRhythm;
pub use sight::{CameraMount, PhotoShutter, SightMode};
pub use tome::ArtisanTome;
pub use wardrobe::{MoldedWardrobe, Wardrobe};

/// Encapsulated four-piece medieval garment selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GarmentEnsemble {
    tunic: TunicStyle,
    vambrace: VambraceStyle,
    girdle: GirdleStyle,
    footwear: FootwearStyle,
}

impl GarmentEnsemble {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        tunic: TunicStyle,
        vambrace: VambraceStyle,
        girdle: GirdleStyle,
        footwear: FootwearStyle,
    ) -> Self {
        Self { tunic, vambrace, girdle, footwear }
    }
}

/// Encapsulated character crown, dye pigment, skin complexion, and body proportions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentityTraits {
    crown: CrownStyle,
    pigment: PigmentSwatch,
    complexion: ComplexionSwatch,
    physique: YouthPhysique,
}

impl IdentityTraits {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        crown: CrownStyle,
        pigment: PigmentSwatch,
        complexion: ComplexionSwatch,
        physique: YouthPhysique,
    ) -> Self {
        Self { crown, pigment, complexion, physique }
    }
}

/// Encapsulated right-palm and left-palm held relics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeldHands {
    relic: HeldRelic,
    offhand: HeldRelic,
}

impl HeldHands {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(relic: HeldRelic, offhand: HeldRelic) -> Self {
        Self { relic, offhand }
    }

    /// Secondary constructor initializing two empty palms.
    pub const fn empty() -> Self {
        Self::new(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm)
    }
}

/// Immutable medieval youth character appearance combining garments, identity traits, and dual-hand relics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YouthAttire {
    garments: GarmentEnsemble,
    identity: IdentityTraits,
    hands: HeldHands,
}

impl YouthAttire {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(garments: GarmentEnsemble, identity: IdentityTraits, hands: HeldHands) -> Self {
        Self { garments, identity, hands }
    }

    pub const fn squire() -> Self {
        Self::new(
            GarmentEnsemble::new(
                TunicStyle::LinenDoublet,
                VambraceStyle::RawhideWraps,
                GirdleStyle::KnottedCord,
                FootwearStyle::CuffedBoots,
            ),
            IdentityTraits::new(
                CrownStyle::TousledMop,
                PigmentSwatch::UnbleachedLinen,
                ComplexionSwatch::Birch,
                YouthPhysique::SlenderSquire,
            ),
            HeldHands::empty(),
        )
    }

    pub fn from_multiplayer_bytes(bytes: [u8; 9]) -> Self {
        Self::new(
            GarmentEnsemble::new(
                TunicStyle::from_code(bytes[0]),
                VambraceStyle::from_code(bytes[1]),
                GirdleStyle::from_code(bytes[2]),
                FootwearStyle::from_code(bytes[3]),
            ),
            IdentityTraits::new(
                CrownStyle::from_code(bytes[4] & 0x0F),
                PigmentSwatch::from_code(bytes[6]),
                ComplexionSwatch::from_code(bytes[7]),
                YouthPhysique::from_code(bytes[8]),
            ),
            HeldHands::new(
                HeldRelic::from_code((bytes[5] & 0x0F) | (((bytes[4] >> 4) & 1) << 4) | (((bytes[4] >> 6) & 1) << 5)),
                HeldRelic::from_code((bytes[5] >> 4) | (((bytes[4] >> 5) & 1) << 4) | (((bytes[4] >> 7) & 1) << 5)),
            ),
        )
    }

    pub fn with_next_tunic(mut self) -> Self { self.garments.tunic = self.garments.tunic.next(); self }
    pub fn with_next_vambrace(mut self) -> Self { self.garments.vambrace = self.garments.vambrace.next(); self }
    pub fn with_next_girdle(mut self) -> Self { self.garments.girdle = self.garments.girdle.next(); self }
    pub fn with_next_footwear(mut self) -> Self { self.garments.footwear = self.garments.footwear.next(); self }
    pub fn with_next_crown(mut self) -> Self { self.identity.crown = self.identity.crown.next(); self }
    #[cfg(test)]
    pub fn with_next_relic(mut self) -> Self { self.hands.relic = self.hands.relic.next(); self }
    pub fn with_relics(mut self, relic: HeldRelic, offhand: HeldRelic) -> Self { self.hands = HeldHands::new(relic, offhand); self }
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
    pub const fn held_pair(&self) -> (HeldRelic, HeldRelic) { (self.hands.relic, self.hands.offhand) }
    pub const fn hand_pair(&self, side: f32) -> (HeldRelic, HeldRelic) {
        if side > 0.0 { (self.hands.relic, self.hands.offhand) } else { (self.hands.offhand, self.hands.relic) }
    }
    pub const fn hunting_hand(self, side: f32) -> (HeldRelic, f32) {
        let (active, other) = self.hand_pair(side);
        if active.hunting_power() > 1 {
            (active, side)
        } else if active.free_palm() && other.hunting_power() > 1 {
            (other, -side)
        } else {
            (active, side)
        }
    }
    pub const fn with_hand_relic(mut self, side: f32, relic: HeldRelic) -> Self {
        if side > 0.0 { self.hands.relic = relic; } else { self.hands.offhand = relic; }
        self
    }
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
        let (r_code, l_code) = (self.hands.relic.code(), self.hands.offhand.code());
        [
            self.garments.tunic.code(),
            self.garments.vambrace.code(),
            self.garments.girdle.code(),
            self.garments.footwear.code(),
            (self.identity.crown.code() & 0x0F)
                | (((r_code >> 4) & 1) << 4)
                | (((l_code >> 4) & 1) << 5)
                | (((r_code >> 5) & 1) << 6)
                | (((l_code >> 5) & 1) << 7),
            (r_code & 0x0F) | ((l_code & 0x0F) << 4),
            self.identity.pigment.code(),
            self.identity.complexion.code(),
            self.identity.physique.code(),
        ]
    }

    pub(super) fn scaled_pose(&self, pose: &BodyPose) -> BodyPose {
        pose.with_physique(self.identity.physique)
    }

    pub(super) fn emit_bust_with_headset(&self, pose: &BodyPose, target: &mut dyn MeshTarget) -> BodyPose {
        use self::crown::HeadSculpture;
        let scaled = self.scaled_pose(pose);
        self.garments.tunic.emit(&scaled, self.identity.pigment, target);
        let crown = self.identity.crown.sculpture(self.identity.complexion, self.identity.pigment);
        intercom::HeadsetHead::aviation(crown).emit(&scaled, target);
        scaled
    }

    pub(super) fn emit_garments(
        &self,
        pose: &BodyPose,
        sight: SightMode,
        molds: PalmMolds,
        tray_open: bool,
        target: &mut dyn MeshTarget,
    ) {
        let scaled_pose = self.scaled_pose(pose);
        self.garments.tunic.emit(&scaled_pose, self.identity.pigment, target);
        self.garments.girdle.emit(&scaled_pose, target);
        self.garments.footwear.emit(&scaled_pose, self.identity.pigment, target);
        self.garments.vambrace.emit(
            &scaled_pose,
            sight,
            (self.identity.complexion, self.identity.pigment),
            HeldPalms::new(self.held_pair(), molds, tray_open),
            target,
        );
        sight.emit_crown(self.identity.crown, &scaled_pose, self.identity.complexion, self.identity.pigment, target);
    }
}

impl Wardrobe for YouthAttire {
    fn emit(&self, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget) {
        MoldedWardrobe::closed(*self, PalmMolds::cubes()).emit(pose, sight, target);
    }
}
