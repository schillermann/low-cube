use glam::Vec3;
use world::MeshTarget;
use super::beard::{BeardCut, BeardedHead};
use super::coiffure::{HairCoiffure, TressedHead};
use super::headgear::{BrowAdornment, CrownedHead};
use super::limb::{AxialLimb, FacetedSolid, TaperedPrism};
use super::palette::{ComplexionSwatch, PigmentSwatch};
use super::pose::BodyPose;
use super::visage::BareVisage;

/// Focused 1-method contract for composable anatomical head, hair, beard, collar, and headwear decorators.
pub trait HeadSculpture {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget);
}

/// `HeadSculpture` decorator framing the neck with a tailored stand-collar and contrasting V-lapel.
#[derive(Clone, Copy, Debug)]
pub struct CollaredHead<S> {
    origin: S,
    trim: Vec3,
    lapel: Vec3,
}

impl<S> CollaredHead<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, trim: Vec3, lapel: Vec3) -> Self {
        Self { origin, trim, lapel }
    }
}

impl<S: HeadSculpture> HeadSculpture for CollaredHead<S> {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.origin.emit(pose, target);
        let torso = pose.torso_frame();
        let collar_y = pose.torso_span(0.72);
        TaperedPrism::new(
            8,
            Vec3::new(0.0, collar_y - 0.02, -0.072),
            Vec3::new(0.0, collar_y + 0.045, -0.062),
            pose.scale(0.098),
            pose.torso_depth(0.082),
            pose.scale(0.076),
            pose.torso_depth(0.070),
            self.trim,
        )
        .emit(&torso, true, target);
        for side in [-1.0_f32, 1.0_f32] {
            AxialLimb::new(
                Vec3::new(side * pose.scale(0.018), collar_y - 0.015, pose.torso_depth(0.016)),
                Vec3::new(side * pose.scale(0.058), collar_y + 0.052, pose.torso_depth(-0.022)),
                pose.scale(0.016),
                pose.scale(0.021),
                self.lapel,
            )
            .emit(&torso, true, target);
        }
    }
}

/// Head, hair, beard, helmet, and hat style visible to multiplayer peers and in Looking-Glass / Studio view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrownStyle {
    TousledMop,
    SquireCrop,
    MedievalGugel,
    MerchantCap,
    VisoredHelm,
    SmithGoggles,
    KettleHelm,
    VoyagerCrest,
}

impl CrownStyle {
    pub fn next(self) -> Self {
        Self::from_code(self.code() + 1)
    }

    pub fn from_code(code: u8) -> Self {
        match code % 8 {
            0 => Self::TousledMop,
            1 => Self::SquireCrop,
            2 => Self::MedievalGugel,
            3 => Self::MerchantCap,
            4 => Self::VisoredHelm,
            5 => Self::SmithGoggles,
            6 => Self::KettleHelm,
            _ => Self::VoyagerCrest,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::TousledMop => 0,
            Self::SquireCrop => 1,
            Self::MedievalGugel => 2,
            Self::MerchantCap => 3,
            Self::VisoredHelm => 4,
            Self::SmithGoggles => 5,
            Self::KettleHelm => 6,
            Self::VoyagerCrest => 7,
        }
    }

    pub const fn sculpture(self, complexion: ComplexionSwatch, pigment: PigmentSwatch) -> StyledCrown {
        StyledCrown::new(self, complexion, pigment)
    }

    pub fn emit(
        self,
        pose: &BodyPose,
        complexion: ComplexionSwatch,
        pigment: PigmentSwatch,
        target: &mut dyn MeshTarget,
    ) {
        self.sculpture(complexion, pigment).emit(pose, target);
    }
}

/// Stack-allocated `HeadSculpture` assembling the full Decorator chain for any `CrownStyle`.
#[derive(Clone, Copy, Debug)]
pub struct StyledCrown {
    style: CrownStyle,
    complexion: ComplexionSwatch,
    pigment: PigmentSwatch,
}

impl StyledCrown {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(style: CrownStyle, complexion: ComplexionSwatch, pigment: PigmentSwatch) -> Self {
        Self { style, complexion, pigment }
    }
}

impl HeadSculpture for StyledCrown {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        let skin = self.complexion.skin();
        let cloth = self.pigment.cloth();
        let trim = self.pigment.trim();
        let lapel = (trim * 0.45 + Vec3::new(0.42, 0.56, 0.64)).min(Vec3::ONE);
        let espresso = (Vec3::new(0.23, 0.14, 0.09), Vec3::new(0.32, 0.20, 0.12));
        let auburn = (Vec3::new(0.36, 0.19, 0.10), Vec3::new(0.45, 0.25, 0.13));
        let golden = (Vec3::new(0.84, 0.66, 0.28), Vec3::new(0.92, 0.76, 0.36));

        let fem = pose.feminine();
        let brow_col = if fem && matches!(self.style, CrownStyle::VisoredHelm) {
            Vec3::new(0.56, 0.42, 0.18)
        } else {
            espresso.0
        };
        let collared = CollaredHead::new(BareVisage::new(skin, brow_col, fem), trim, lapel);
        let sw = (cloth, trim);

        if fem {
            match self.style {
                CrownStyle::VisoredHelm => {
                    CrownedHead::new(TressedHead::new(collared, HairCoiffure::FlowingBraids, golden), BrowAdornment::WingedDiadem, sw)
                        .emit(pose, target);
                }
                CrownStyle::MedievalGugel => {
                    CrownedHead::new(TressedHead::new(collared, HairCoiffure::FlowingBraids, espresso), BrowAdornment::RangerHood, sw)
                        .emit(pose, target);
                }
                CrownStyle::MerchantCap => {
                    CrownedHead::new(TressedHead::new(collared, HairCoiffure::FlowingBraids, auburn), BrowAdornment::FloralWreath, sw)
                        .emit(pose, target);
                }
                CrownStyle::SmithGoggles => {
                    TressedHead::new(collared, HairCoiffure::SilkTopknot, espresso).emit(pose, target);
                }
                CrownStyle::KettleHelm => {
                    CrownedHead::new(TressedHead::new(collared, HairCoiffure::FlowingBraids, espresso), BrowAdornment::BattleCirclet, sw)
                        .emit(pose, target);
                }
                CrownStyle::TousledMop => {
                    TressedHead::new(collared, HairCoiffure::FlowingBraids, espresso).emit(pose, target);
                }
                CrownStyle::SquireCrop | CrownStyle::VoyagerCrest => {
                    CrownedHead::new(TressedHead::new(collared, HairCoiffure::FlowingBraids, auburn), BrowAdornment::RoyalTiara, sw)
                        .emit(pose, target);
                }
            }
            return;
        }

        match self.style {
            CrownStyle::TousledMop => {
                TressedHead::new(collared, HairCoiffure::PartedWave, espresso).emit(pose, target);
            }
            CrownStyle::VoyagerCrest => {
                let bearded = BeardedHead::new(collared, BeardCut::FullJawline, (auburn.0, espresso.0));
                TressedHead::new(bearded, HairCoiffure::PartedWave, espresso).emit(pose, target);
            }
            CrownStyle::SquireCrop => {
                let bearded = BeardedHead::new(collared, BeardCut::PointedGoatee, auburn);
                let tressed = TressedHead::new(bearded, HairCoiffure::CroppedTemples, auburn);
                CrownedHead::new(tressed, BrowAdornment::SquireBeret, sw).emit(pose, target);
            }
            CrownStyle::MedievalGugel => {
                let bearded = BeardedHead::new(collared, BeardCut::RangerStubble, espresso);
                let tressed = TressedHead::new(bearded, HairCoiffure::CroppedTemples, espresso);
                CrownedHead::new(tressed, BrowAdornment::RangerHood, sw).emit(pose, target);
            }
            CrownStyle::MerchantCap => {
                let bearded = BeardedHead::new(collared, BeardCut::FullJawline, (auburn.0, espresso.0));
                let tressed = TressedHead::new(bearded, HairCoiffure::CroppedTemples, auburn);
                CrownedHead::new(tressed, BrowAdornment::VelvetCap, sw).emit(pose, target);
            }
            CrownStyle::VisoredHelm => {
                let tressed = TressedHead::new(collared, HairCoiffure::CroppedTemples, espresso);
                CrownedHead::new(tressed, BrowAdornment::RaisedSallet, sw).emit(pose, target);
            }
            CrownStyle::SmithGoggles => {
                let bearded = BeardedHead::new(collared, BeardCut::BraidedFork, espresso);
                let tressed = TressedHead::new(bearded, HairCoiffure::PartedWave, espresso);
                CrownedHead::new(tressed, BrowAdornment::BrowGoggles, sw).emit(pose, target);
            }
            CrownStyle::KettleHelm => {
                let bearded = BeardedHead::new(collared, BeardCut::GarrisonMustache, espresso);
                let tressed = TressedHead::new(bearded, HairCoiffure::CroppedTemples, espresso);
                CrownedHead::new(tressed, BrowAdornment::KettleBrim, sw).emit(pose, target);
            }
        }
    }
}
