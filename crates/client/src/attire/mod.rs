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
mod proportion;
mod relic;
mod sight;
#[cfg(test)]
mod tests;

pub use crown::CrownStyle;
pub use file_attire::{AttireProfile, FileAttire};
pub use footwear::FootwearStyle;
pub use garments::{GirdleStyle, TunicStyle};
pub use hands::VambraceStyle;
pub use limb::{FacetedSolid, LimbFrame, TaperedPrism};
pub use palette::{ComplexionSwatch, PigmentSwatch};
pub use pose::{BodyPose, StrideRhythm};
pub use proportion::{BodyRegion, PartProportion, YouthPhysique};
pub use relic::{HeldRelic, MartialStyle};
pub use sight::SightMode;

/// Interface for avatar wardrobe appearance emission.
pub trait Wardrobe {
    fn emit(&self, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget);
}

/// Immutable medieval youth character appearance combining body traits, garments, dyes, and dual-hand held relics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YouthAttire {
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
            tunic,
            vambrace,
            girdle,
            footwear,
            crown,
            relic,
            offhand,
            pigment,
            complexion,
            physique,
        }
    }

    /// Secondary constructor assembling a youthful medieval squire in linen doublet and cuffed boots.
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

    pub fn with_next_tunic(self) -> Self { Self { tunic: self.tunic.next(), ..self } }
    pub fn with_next_vambrace(self) -> Self { Self { vambrace: self.vambrace.next(), ..self } }
    pub fn with_next_girdle(self) -> Self { Self { girdle: self.girdle.next(), ..self } }
    pub fn with_next_footwear(self) -> Self { Self { footwear: self.footwear.next(), ..self } }
    pub fn with_next_crown(self) -> Self { Self { crown: self.crown.next(), ..self } }
    pub fn with_next_relic(self) -> Self { Self { relic: self.relic.next(), ..self } }
    pub fn with_next_offhand(self) -> Self { Self { offhand: self.offhand.next(), ..self } }
    pub fn with_relics(self, relic: HeldRelic, offhand: HeldRelic) -> Self { Self { relic, offhand, ..self } }
    pub fn with_complexion(self, complexion: ComplexionSwatch) -> Self { Self { complexion, ..self } }
    pub fn with_physique(self, physique: YouthPhysique) -> Self { Self { physique, ..self } }

    pub fn with_adjusted_region(self, region: BodyRegion, delta_size: i8, delta_girth: i8) -> Self {
        Self { physique: self.physique.adjusted(region, delta_size, delta_girth), ..self }
    }

    pub fn region_proportion(&self, region: BodyRegion) -> PartProportion {
        self.physique.region_proportion(region)
    }

    pub fn region_bytes(&self) -> [u8; 4] {
        self.physique.region_bytes()
    }

    pub fn selected_complexion(&self, candidate: ComplexionSwatch) -> bool {
        self.complexion == candidate
    }

    pub fn right_armed(&self) -> bool { !matches!(self.relic.martial_style(), MartialStyle::Unarmed) }
    pub fn left_armed(&self) -> bool { !matches!(self.offhand.martial_style(), MartialStyle::Unarmed) }
    pub fn armed(&self) -> bool { self.right_armed() || self.left_armed() }

    pub fn with_next_pigment(self) -> Self {
        Self { pigment: self.pigment.next(), ..self }
    }

    pub fn with_next_complexion(self) -> Self {
        Self {
            complexion: self.complexion.next(),
            physique: self.physique.next(),
            ..self
        }
    }

    /// Returns a new immutable YouthAttire when the key corresponds to a wardrobe customization slot without disk I/O.
    pub fn with_wardrobe_key(self, code: KeyCode) -> Option<Self> {
        let updated = match code {
            KeyCode::Digit1 => self.with_next_tunic(),
            KeyCode::Digit2 => self.with_next_vambrace(),
            KeyCode::Digit3 => self.with_next_girdle(),
            KeyCode::Digit4 => self.with_next_footwear(),
            KeyCode::Digit5 => self.with_next_crown(),
            KeyCode::Digit6 => self.with_next_pigment(),
            KeyCode::Digit7 => self.with_next_complexion(),
            KeyCode::KeyH | KeyCode::Digit8 => self.with_next_relic(),
            KeyCode::KeyG | KeyCode::Digit9 => self.with_next_offhand(),
            _ => return None,
        };
        Some(updated)
    }

    /// Returns a new immutable YouthAttire and persists it when the key corresponds to a wardrobe customization slot.
    pub fn cycled_by_key(self, code: KeyCode) -> Option<Self> {
        let updated = self.with_wardrobe_key(code)?;
        FileAttire::profile().save(&updated);
        Some(updated)
    }

    pub fn multiplayer_bytes(&self) -> [u8; 9] {
        [
            self.tunic.code(),
            self.vambrace.code(),
            self.girdle.code(),
            self.footwear.code(),
            self.crown.code(),
            self.relic.code() + self.offhand.code() * 6,
            self.pigment.code(),
            self.complexion.code(),
            self.physique.code(),
        ]
    }

    /// Emits complete multiplayer peer avatar (including head, hair/hood, shoulders, body, and held item).
    #[cfg(test)]
    pub fn emit_peer(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.emit(pose, SightMode::LookingGlassSight, target);
    }
}

impl Wardrobe for YouthAttire {
    /// Emits avatar geometry according to SightMode using polymorphic garment objects.
    fn emit(&self, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget) {
        let scaled_pose = pose.with_physique(self.physique);
        self.tunic.emit(&scaled_pose, self.pigment, target);
        self.girdle.emit(&scaled_pose, target);
        self.footwear.emit(&scaled_pose, self.pigment, target);
        self.vambrace.emit(&scaled_pose, sight, self.complexion, self.pigment, (self.relic, self.offhand), target);
        sight.emit_crown(self.crown, &scaled_pose, self.complexion, self.pigment, target);
    }
}
