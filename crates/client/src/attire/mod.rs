use winit::keyboard::KeyCode;
use world::{MeshTarget, PalmMolds};

mod accoutrement;
mod beard;
mod burden;
mod coiffure;
mod compartment;
mod crown;
mod ensemble;
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
mod tray;
mod visage;
mod wardrobe;
#[cfg(test)]
mod tests;

pub use burden::SatchelBurden;
pub use crown::HelmStyle;
pub use ensemble::{CrewIdentity, HeldHands, SuitEnsemble};
pub use file_attire::{AttireProfile, FileAttire};
pub use footwear::MagBootStyle;
pub use garments::{PlssHarness, SuitShell};
pub use hands::{HeldPalms, SuitGauntlet};
pub use headgear::LampBeam;
pub use intercom::IntercomBust;
pub use limb::{FacetedSolid, LimbFrame, TaperedPrism};
pub use palette::{ComplexionSwatch, PigmentSwatch};
pub use pose::{BodyPose, RocketControls};
pub use pouch::BeltSatchel;
pub use proportion::{BodyRegion, CrewPhysique, PartProportion};
pub use relic::{HeldRelic, MartialStyle};
#[cfg(test)]
pub use relic::ToolCondition;
pub use rhythm::StrideRhythm;
pub use sight::{CameraMount, PhotoShutter, SightMode};
pub use wardrobe::{MoldedWardrobe, Wardrobe};

/// Immutable Space-Age EVA crew appearance combining pressure suit, crew identity, dual hands, and helmet lamp (`4` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvaAttire {
    suit: SuitEnsemble,
    identity: CrewIdentity,
    hands: HeldHands,
    lamp: LampBeam,
}

impl EvaAttire {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_lamp(
        suit: SuitEnsemble,
        identity: CrewIdentity,
        hands: HeldHands,
        lamp: LampBeam,
    ) -> Self {
        Self { suit, identity, hands, lamp }
    }

    pub const fn new(suit: SuitEnsemble, identity: CrewIdentity, hands: HeldHands) -> Self {
        Self::with_lamp(suit, identity, hands, LampBeam::Illuminated)
    }

    /// Pressurized EVA suit with integrated helmet floodlight active from second 1.
    pub const fn astronaut() -> Self {
        Self::with_lamp(
            SuitEnsemble::new(
                SuitShell::FlightSuit,
                SuitGauntlet::TactileGloves,
                PlssHarness::PlssBackpack,
                MagBootStyle::MagLockBoots,
            ),
            CrewIdentity::new(
                HelmStyle::ClearBubble,
                PigmentSwatch::LunarWhite,
                ComplexionSwatch::Birch,
                CrewPhysique::OrbitalCadet,
            ),
            HeldHands::empty(),
            LampBeam::Illuminated,
        )
    }

    /// Full EVA mission kit from second 1: pressure suit + illuminated helmet lamp + `RigWrench` + `SuitRadio`.
    pub const fn eva_equipped() -> Self {
        Self::astronaut().with_relics(HeldRelic::RigWrench, HeldRelic::SuitRadio)
    }

    pub fn from_multiplayer_bytes(bytes: [u8; 9]) -> Self {
        let (r_code, l_code) = world::PresenceAttire::unpack_hands(bytes[4], bytes[5]);
        Self::with_lamp(
            SuitEnsemble::new(
                SuitShell::from_code(bytes[0]),
                SuitGauntlet::from_code(bytes[1]),
                PlssHarness::from_code(bytes[2]),
                MagBootStyle::from_code(bytes[3]),
            ),
            CrewIdentity::new(
                HelmStyle::from_code(bytes[4] & 0x0F),
                PigmentSwatch::from_code(bytes[6]),
                ComplexionSwatch::from_code(bytes[7]),
                CrewPhysique::from_code(bytes[8]),
            ),
            HeldHands::new(HeldRelic::from_code(r_code), HeldRelic::from_code(l_code)),
            LampBeam::from_wire_byte(bytes[0]),
        )
    }

    #[cfg(test)]
    pub const fn lamp_illuminated(&self) -> bool { self.lamp.illuminated() }
    pub const fn with_toggled_lamp(mut self) -> Self { self.lamp = self.lamp.toggled(); self }
    pub fn with_next_shell(mut self) -> Self { self.suit = self.suit.next_shell(); self }
    pub fn with_next_gauntlet(mut self) -> Self { self.suit = self.suit.next_gauntlet(); self }
    pub fn with_next_harness(mut self) -> Self { self.suit = self.suit.next_harness(); self }
    pub fn with_next_boots(mut self) -> Self { self.suit = self.suit.next_boots(); self }
    pub fn with_next_helm(mut self) -> Self { self.identity = self.identity.next_helm(); self }
    #[cfg(test)]
    pub fn with_next_relic(mut self) -> Self { self.hands = self.hands.next_relic(); self }
    pub const fn with_relics(mut self, relic: HeldRelic, offhand: HeldRelic) -> Self {
        self.hands = HeldHands::new(relic, offhand);
        self
    }
    pub const fn with_complexion(mut self, complexion: ComplexionSwatch) -> Self {
        self.identity = self.identity.with_complexion(complexion);
        self
    }
    pub const fn with_physique(mut self, physique: CrewPhysique) -> Self {
        self.identity = self.identity.with_physique(physique);
        self
    }
    pub fn with_feminine(mut self, feminine: bool) -> Self {
        self.identity = self.identity.with_feminine(feminine);
        self
    }
    pub fn feminine(&self) -> bool { self.identity.physique().feminine() }

    pub fn with_adjusted_region(mut self, region: BodyRegion, delta_size: i8, delta_girth: i8) -> Self {
        self.identity = self.identity.adjusted_region(region, delta_size, delta_girth);
        self
    }

    pub fn region_proportion(&self, region: BodyRegion) -> PartProportion {
        self.identity.region_proportion(region)
    }
    pub fn shaft_yaw_play(&self) -> f32 { self.identity.physique().shaft_yaw_play() }
    pub fn body_envelope(&self) -> crate::envelope::BodyEnvelope {
        crate::envelope::BodyEnvelope::for_physique(self.identity.physique())
    }
    pub fn region_bytes(&self) -> [u8; 4] { self.identity.physique().region_bytes() }
    pub fn selected_complexion(&self, candidate: ComplexionSwatch) -> bool {
        self.identity.complexion() == candidate
    }
    pub fn right_armed(&self) -> bool { !matches!(self.held_pair().0.martial_style(), MartialStyle::Unarmed) }
    pub fn left_armed(&self) -> bool { !matches!(self.held_pair().1.martial_style(), MartialStyle::Unarmed) }
    pub fn armed(&self) -> bool { self.right_armed() || self.left_armed() }
    pub const fn held_pair(&self) -> (HeldRelic, HeldRelic) { self.hands.pair() }
    pub const fn hand_pair(&self, side: f32) -> (HeldRelic, HeldRelic) {
        let (r, l) = self.hands.pair();
        if side > 0.0 { (r, l) } else { (l, r) }
    }
    pub const fn with_hand_relic(self, side: f32, relic: HeldRelic) -> Self {
        let (r, l) = self.hands.pair();
        if side > 0.0 { self.with_relics(relic, l) } else { self.with_relics(r, relic) }
    }
    pub fn with_next_pigment(mut self) -> Self { self.identity = self.identity.next_pigment(); self }
    pub fn with_next_complexion(mut self) -> Self { self.identity = self.identity.next_complexion(); self }

    pub fn with_wardrobe_key(self, code: KeyCode) -> Option<Self> {
        let updated = match code {
            KeyCode::Digit1 => self.with_next_shell(),
            KeyCode::Digit2 => self.with_next_gauntlet(),
            KeyCode::Digit3 => self.with_next_harness(),
            KeyCode::Digit4 => self.with_next_boots(),
            KeyCode::Digit5 => self.with_next_helm(),
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
        let (r, l) = self.held_pair();
        let (b4, b5) = world::PresenceAttire::pack_hands(
            self.identity.helm().code(),
            r.code(),
            l.code(),
        );
        [
            self.suit.shell().code() | self.lamp.wire_bit(),
            self.suit.gauntlet().code(),
            self.suit.harness().code(),
            self.suit.boots().code(),
            b4,
            b5,
            self.identity.pigment().code(),
            self.identity.complexion().code(),
            self.identity.physique().code(),
        ]
    }

    pub(super) fn scaled_pose(&self, pose: &BodyPose) -> BodyPose {
        pose.with_physique(self.identity.physique())
    }

    pub(super) fn emit_bust_with_headset(&self, pose: &BodyPose, target: &mut dyn MeshTarget) -> BodyPose {
        use self::crown::HeadSculpture;
        let scaled = self.scaled_pose(pose);
        self.suit.shell().emit(&scaled, self.identity.pigment(), target);
        let helm = self.identity.helm().sculpture(self.identity.complexion(), self.identity.pigment(), self.lamp);
        intercom::HeadsetHead::aviation(helm).emit(&scaled, target);
        scaled
    }

    pub(super) fn emit_suit(
        &self,
        pose: &BodyPose,
        sight: SightMode,
        molds: PalmMolds,
        tray_open: bool,
        target: &mut dyn MeshTarget,
    ) {
        let scaled_pose = self.scaled_pose(pose);
        let pigment = self.identity.pigment();
        let complexion = self.identity.complexion();
        self.suit.shell().emit(&scaled_pose, pigment, target);
        self.suit.harness().emit(&scaled_pose, target);
        self.suit.boots().emit(&scaled_pose, pigment, target);
        self.suit.gauntlet().emit(
            &scaled_pose,
            sight,
            (complexion, pigment),
            HeldPalms::new(self.held_pair(), molds, tray_open),
            target,
        );
        sight.emit_helm(self.identity.helm(), &scaled_pose, (complexion, pigment), self.lamp, target);
    }
}

impl Wardrobe for EvaAttire {
    fn emit(&self, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget) {
        MoldedWardrobe::closed(*self, PalmMolds::cubes()).emit(pose, sight, target);
    }
}
