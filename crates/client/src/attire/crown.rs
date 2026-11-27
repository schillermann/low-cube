use glam::Vec3;
use world::MeshTarget;
use super::beard::{BeardCut, BeardedHead};
use super::coiffure::{HairCoiffure, TressedHead};
use super::headgear::{HelmVisor, HelmetLampHead, LampBeam, PressureHelmHead, RadioMastHead};
use super::limb::{AxialLimb, FacetedSolid, TaperedPrism};
use super::palette::{ComplexionSwatch, PigmentSwatch};
use super::pose::BodyPose;
use super::visage::BareVisage;

/// Focused 1-method contract for composable crew head, neck-ring, visor, comms, and helmet-lamp decorators.
pub trait HeadSculpture {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget);
}

/// `HeadSculpture` decorator framing the neck with an anodized titanium EVA pressure-disconnect ring (`3` fields).
#[derive(Clone, Copy, Debug)]
pub struct NeckRingHead<S> {
    origin: S,
    ring_alloy: Vec3,
    latch_trim: Vec3,
}

impl<S> NeckRingHead<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, ring_alloy: Vec3, latch_trim: Vec3) -> Self {
        Self { origin, ring_alloy, latch_trim }
    }
}

impl<S: HeadSculpture> HeadSculpture for NeckRingHead<S> {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.origin.emit(pose, target);
        let torso = pose.torso_frame();
        let collar_y = pose.torso_span(0.72);
        TaperedPrism::new(
            8,
            Vec3::new(0.0, collar_y - 0.020, -0.070),
            Vec3::new(0.0, collar_y + 0.042, -0.060),
            pose.scale(0.106),
            pose.torso_depth(0.090),
            pose.scale(0.096),
            pose.torso_depth(0.084),
            self.ring_alloy,
        )
        .emit(&torso, true, target);
        for side in [-1.0_f32, 1.0_f32] {
            AxialLimb::new(
                Vec3::new(side * pose.scale(0.084), collar_y - 0.008, pose.torso_depth(-0.048)),
                Vec3::new(side * pose.scale(0.092), collar_y + 0.038, pose.torso_depth(-0.048)),
                pose.scale(0.016),
                pose.scale(0.018),
                self.latch_trim,
            )
            .emit(&torso, true, target);
        }
    }
}

/// Space-Age EVA pressure helmet configuration visible to multiplayer peers and in Studio / Looking-Glass view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelmStyle {
    ClearBubble,
    SolarVisor,
    SurveyDome,
    TacticalShell,
    CommsCanopy,
    OpticGoggles,
    ArmoredCowl,
    CommandCrest,
}

impl HelmStyle {
    pub fn next(self) -> Self {
        Self::from_code(self.code() + 1)
    }

    pub fn from_code(code: u8) -> Self {
        match code % 8 {
            0 => Self::ClearBubble,
            1 => Self::SolarVisor,
            2 => Self::SurveyDome,
            3 => Self::TacticalShell,
            4 => Self::CommsCanopy,
            5 => Self::OpticGoggles,
            6 => Self::ArmoredCowl,
            _ => Self::CommandCrest,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::ClearBubble => 0,
            Self::SolarVisor => 1,
            Self::SurveyDome => 2,
            Self::TacticalShell => 3,
            Self::CommsCanopy => 4,
            Self::OpticGoggles => 5,
            Self::ArmoredCowl => 6,
            Self::CommandCrest => 7,
        }
    }

    pub const fn sculpture(
        self,
        complexion: ComplexionSwatch,
        pigment: PigmentSwatch,
        lamp: LampBeam,
    ) -> StyledHelm {
        StyledHelm::new(self, (complexion, pigment), lamp)
    }

    pub fn emit(
        self,
        pose: &BodyPose,
        (complexion, pigment): (ComplexionSwatch, PigmentSwatch),
        lamp: LampBeam,
        target: &mut dyn MeshTarget,
    ) {
        self.sculpture(complexion, pigment, lamp).emit(pose, target);
    }
}

/// Stack-allocated `HeadSculpture` composing `HelmetLampHead<RadioMastHead<PressureHelmHead<NeckRingHead<...>>>>` (`3` fields).
#[derive(Clone, Copy, Debug)]
pub struct StyledHelm {
    style: HelmStyle,
    swatches: (ComplexionSwatch, PigmentSwatch),
    lamp: LampBeam,
}

impl StyledHelm {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        style: HelmStyle,
        swatches: (ComplexionSwatch, PigmentSwatch),
        lamp: LampBeam,
    ) -> Self {
        Self { style, swatches, lamp }
    }

    fn visor_kind(self) -> HelmVisor {
        match self.style {
            HelmStyle::ClearBubble | HelmStyle::CommsCanopy => HelmVisor::ClearBubble,
            HelmStyle::SolarVisor | HelmStyle::CommandCrest => HelmVisor::SolarGold,
            HelmStyle::SurveyDome => HelmVisor::SurveyCyan,
            HelmStyle::TacticalShell => HelmVisor::TacticalSlit,
            HelmStyle::OpticGoggles => HelmVisor::OpticArray,
            HelmStyle::ArmoredCowl => HelmVisor::ArmoredCowl,
        }
    }
}

impl HeadSculpture for StyledHelm {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        let (complexion, pigment) = self.swatches;
        let skin = complexion.skin();
        let shell = pigment.cloth();
        let trim = pigment.trim();
        let ring_alloy = Vec3::new(0.58, 0.63, 0.70);
        let espresso = (Vec3::new(0.23, 0.14, 0.09), Vec3::new(0.32, 0.20, 0.12));
        let auburn = (Vec3::new(0.36, 0.19, 0.10), Vec3::new(0.45, 0.25, 0.13));
        let fem = pose.feminine();
        let neck_sealed = NeckRingHead::new(BareVisage::new(skin, espresso.0, fem), ring_alloy, trim);
        let visor = self.visor_kind();
        let mast_accent = Vec3::new(0.86, 0.90, 0.96);

        if fem {
            let coiffure = if matches!(self.style, HelmStyle::OpticGoggles | HelmStyle::TacticalShell) {
                HairCoiffure::SilkTopknot
            } else {
                HairCoiffure::FlowingBraids
            };
            let hair = if matches!(self.style, HelmStyle::CommandCrest | HelmStyle::SurveyDome) {
                auburn
            } else {
                espresso
            };
            let tressed = TressedHead::new(neck_sealed, coiffure, hair);
            let helmed = PressureHelmHead::new(tressed, visor, (shell, trim));
            HelmetLampHead::new(RadioMastHead::new(helmed, mast_accent), self.lamp).emit(pose, target);
            return;
        }

        match self.style {
            HelmStyle::ClearBubble | HelmStyle::SolarVisor | HelmStyle::TacticalShell => {
                let tressed = TressedHead::new(neck_sealed, HairCoiffure::CroppedTemples, espresso);
                let helmed = PressureHelmHead::new(tressed, visor, (shell, trim));
                HelmetLampHead::new(RadioMastHead::new(helmed, mast_accent), self.lamp).emit(pose, target);
            }
            HelmStyle::SurveyDome | HelmStyle::CommsCanopy => {
                let bearded = BeardedHead::new(neck_sealed, BeardCut::CrewStubble, espresso);
                let tressed = TressedHead::new(bearded, HairCoiffure::CroppedTemples, espresso);
                let helmed = PressureHelmHead::new(tressed, visor, (shell, trim));
                HelmetLampHead::new(RadioMastHead::new(helmed, mast_accent), self.lamp).emit(pose, target);
            }
            HelmStyle::OpticGoggles | HelmStyle::ArmoredCowl | HelmStyle::CommandCrest => {
                let bearded = BeardedHead::new(neck_sealed, BeardCut::FullJawline, (auburn.0, espresso.0));
                let tressed = TressedHead::new(bearded, HairCoiffure::PartedWave, auburn);
                let helmed = PressureHelmHead::new(tressed, visor, (shell, trim));
                HelmetLampHead::new(RadioMastHead::new(helmed, mast_accent), self.lamp).emit(pose, target);
            }
        }
    }
}
