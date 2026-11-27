use glam::Vec3;
use world::MeshTarget;
use super::accoutrement::{ChestDcuTorso, PlssPackTorso};
use super::limb::{FacetedSolid, TaperedPrism};
use super::palette::PigmentSwatch;
use super::pose::BodyPose;

/// Composable 3D EVA upper-body shell, chest DCU, waist ring, and dorsal PLSS sculpture contract.
pub trait TorsoSculpture {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget);
}

#[derive(Clone, Copy)]
struct PressurizedTorso {
    style: SuitShell,
    pigment: PigmentSwatch,
}

impl PressurizedTorso {
    const fn new(style: SuitShell, pigment: PigmentSwatch) -> Self {
        Self { style, pigment }
    }
}

impl TorsoSculpture for PressurizedTorso {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        let frame = pose.torso_frame();
        let hip_y = pose.hip_elevation();
        let waist_y = pose.torso_span(0.26);
        let chest_y = pose.torso_span(0.54);
        let sternum_y = pose.torso_span(0.72);

        let cloth = self.pigment.cloth();
        let trim = self.pigment.trim();
        let carbon = Vec3::new(0.24, 0.28, 0.34);
        let titanium = Vec3::new(0.64, 0.70, 0.78);
        let (hip_col, chest_col, bearing_col) = match self.style {
            SuitShell::FlightSuit => (cloth, cloth, trim),
            SuitShell::SurveySuit => (carbon, cloth, trim),
            SuitShell::ThermalShell => (cloth * 0.94, cloth, titanium),
            SuitShell::ArmoredRig => (carbon, titanium, titanium),
            SuitShell::TechCoverall => (carbon, cloth, carbon),
            SuitShell::CommandSuit => (trim, cloth, titanium),
        };

        let fem = pose.feminine();
        let (hip_y0, hip_rx, hip_rz, waist_rx, waist_rz, chest_rx, sternum_rx) = if fem {
            (hip_y - 0.14, pose.scale(0.212), pose.torso_depth(0.142), pose.scale(0.118), pose.torso_depth(0.088), pose.scale(0.186), pose.scale(0.180))
        } else {
            (hip_y, pose.scale(0.204), pose.torso_depth(0.126), pose.scale(0.152), pose.torso_depth(0.104), pose.scale(0.218), pose.scale(0.238))
        };

        TaperedPrism::new(8, Vec3::new(0.0, hip_y0, -0.030), Vec3::new(0.0, waist_y, -0.048), hip_rx, hip_rz, waist_rx, waist_rz, hip_col)
            .emit(&frame, false, target);
        TaperedPrism::new(8, Vec3::new(0.0, waist_y, -0.048), Vec3::new(0.0, chest_y, -0.064), waist_rx * 0.98, waist_rz * 0.98, chest_rx, pose.torso_depth(0.118), chest_col)
            .emit(&frame, false, target);
        TaperedPrism::new(8, Vec3::new(0.0, chest_y, -0.064), Vec3::new(0.0, sternum_y, -0.076), chest_rx, pose.torso_depth(0.118), sternum_rx, pose.torso_depth(0.106), chest_col)
            .emit(&frame, true, target);

        if fem {
            let under_col = chest_col * 0.76;
            let upper_col = (chest_col * 1.06).min(Vec3::ONE);
            let y0 = chest_y - pose.torso_size(0.110);
            let y1 = chest_y - pose.torso_size(0.064);
            let y2 = chest_y + pose.torso_size(0.018);
            let y_top = chest_y + pose.torso_size(0.076);
            for side in [-1.0f32, 1.0f32] {
                let bx = side * pose.scale(0.056);
                TaperedPrism::new(8, Vec3::new(bx * 0.93, y0, 0.024), Vec3::new(bx, y1, 0.062), pose.scale(0.090), 0.048, pose.scale(0.100), 0.068, under_col)
                    .emit(&frame, false, target);
                TaperedPrism::new(8, Vec3::new(bx, y1, 0.062), Vec3::new(bx, y2, 0.070), pose.scale(0.100), 0.068, pose.scale(0.102), 0.074, chest_col)
                    .emit(&frame, false, target);
                TaperedPrism::new(8, Vec3::new(bx, y2, 0.070), Vec3::new(bx * 0.90, y_top, 0.030), pose.scale(0.102), 0.074, pose.scale(0.090), 0.050, upper_col)
                    .emit(&frame, true, target);
            }
        }

        for side in [-1.0f32, 1.0f32] {
            let sx = side * pose.scale(if fem { 0.178 } else { 0.226 });
            TaperedPrism::new(
                8,
                Vec3::new(sx, sternum_y - pose.torso_size(0.12), -0.066),
                Vec3::new(sx * 0.95, sternum_y - pose.torso_size(0.008), -0.066),
                pose.scale(if fem { 0.058 } else { 0.078 }),
                pose.torso_depth(0.080),
                pose.scale(if fem { 0.050 } else { 0.068 }),
                pose.torso_depth(0.070),
                bearing_col,
            )
            .emit(&frame, true, target);
        }
    }
}

/// Space-Age pressurized EVA upper-body shell and flight coverall style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuitShell {
    FlightSuit,
    SurveySuit,
    ThermalShell,
    ArmoredRig,
    TechCoverall,
    CommandSuit,
}

impl SuitShell {
    pub fn next(self) -> Self { Self::from_code(self.code() + 1) }

    pub fn from_code(code: u8) -> Self {
        match (code & 0x7F) % 6 {
            0 => Self::FlightSuit,
            1 => Self::SurveySuit,
            2 => Self::ThermalShell,
            3 => Self::ArmoredRig,
            4 => Self::TechCoverall,
            _ => Self::CommandSuit,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::FlightSuit => 0,
            Self::SurveySuit => 1,
            Self::ThermalShell => 2,
            Self::ArmoredRig => 3,
            Self::TechCoverall => 4,
            Self::CommandSuit => 5,
        }
    }

    /// Emits the pressurized EVA upper torso decorated with `ChestDcuTorso`.
    pub fn emit(self, pose: &BodyPose, pigment: PigmentSwatch, target: &mut dyn MeshTarget) {
        ChestDcuTorso::new(PressurizedTorso::new(self, pigment), self, pigment).emit(pose, target);
    }
}

#[derive(Clone, Copy)]
struct WaistDisconnectRing {
    style: PlssHarness,
}

impl WaistDisconnectRing {
    const fn new(style: PlssHarness) -> Self { Self { style } }
}

impl TorsoSculpture for WaistDisconnectRing {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        let frame = pose.torso_frame();
        let waist_y = pose.torso_span(0.26);
        let carbon = Vec3::new(0.22, 0.26, 0.32);
        let alloy = Vec3::new(0.72, 0.78, 0.86);
        let pouch = Vec3::new(0.34, 0.40, 0.48);
        let rx = pose.scale(if pose.feminine() { 0.122 } else { 0.158 });
        let rz = pose.torso_depth(if pose.feminine() { 0.090 } else { 0.106 });
        let latch_z = -0.048 + rz - 0.004;

        TaperedPrism::new(8, Vec3::new(0.0, waist_y - 0.026, -0.048), Vec3::new(0.0, waist_y + 0.026, -0.048), rx, rz, rx, rz, carbon)
            .emit(&frame, false, target);
        TaperedPrism::new(4, Vec3::new(0.0, waist_y - 0.028, latch_z), Vec3::new(0.0, waist_y + 0.028, latch_z), 0.030, 0.016, 0.030, 0.016, alloy)
            .emit(&frame, true, target);

        if matches!(self.style, PlssHarness::SamplePouchRig | PlssHarness::BeaconBelt) {
            let px = pose.scale(0.160);
            TaperedPrism::hexagonal(Vec3::new(px, waist_y - 0.08, -0.008), Vec3::new(px, waist_y + 0.01, -0.008), 0.038, 0.028, pouch)
                .emit(&frame, true, target);
        }
    }
}

/// Space-Age dorsal PLSS life-support pack, umbilical belt, and EVA utility harness style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlssHarness {
    UmbilicalBelt,
    SamplePouchRig,
    PlssBackpack,
    JetpackHarness,
    ToolRigBelt,
    BeaconBelt,
}

impl PlssHarness {
    pub fn next(self) -> Self { Self::from_code(self.code() + 1) }

    pub fn from_code(code: u8) -> Self {
        match code % 6 {
            0 => Self::UmbilicalBelt,
            1 => Self::SamplePouchRig,
            2 => Self::PlssBackpack,
            3 => Self::JetpackHarness,
            4 => Self::ToolRigBelt,
            _ => Self::BeaconBelt,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::UmbilicalBelt => 0,
            Self::SamplePouchRig => 1,
            Self::PlssBackpack => 2,
            Self::JetpackHarness => 3,
            Self::ToolRigBelt => 4,
            Self::BeaconBelt => 5,
        }
    }

    /// Emits the EVA waist disconnect ring decorated with `PlssPackTorso`.
    pub fn emit(self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        PlssPackTorso::new(WaistDisconnectRing::new(self), self).emit(pose, target);
    }
}
