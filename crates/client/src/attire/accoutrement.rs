use glam::Vec3;
use world::MeshTarget;
use super::garments::{PlssHarness, SuitShell, TorsoSculpture};
use super::limb::{AxialLimb, FacetedSolid, TaperedPrism};
use super::palette::PigmentSwatch;
use super::pose::BodyPose;

/// `TorsoSculpture` decorator mounting a Chest Display & Control Unit (DCU) and shell fittings (`3` fields).
#[derive(Clone, Copy)]
pub struct ChestDcuTorso<S> {
    origin: S,
    style: SuitShell,
    pigment: PigmentSwatch,
}

impl<S> ChestDcuTorso<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, style: SuitShell, pigment: PigmentSwatch) -> Self {
        Self { origin, style, pigment }
    }
}

impl<S: TorsoSculpture> TorsoSculpture for ChestDcuTorso<S> {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.origin.emit(pose, target);
        let frame = pose.torso_frame();
        let waist_y = pose.torso_span(0.26);
        let chest_y = pose.torso_span(0.54);
        let sternum_y = pose.torso_span(0.72);
        let cloth = self.pigment.cloth();
        let trim = self.pigment.trim();
        let titanium = Vec3::new(0.68, 0.74, 0.82);
        let carbon = Vec3::new(0.22, 0.26, 0.32);
        let cyan_led = Vec3::new(0.26, 0.90, 0.98);
        let amber_led = Vec3::new(0.96, 0.74, 0.22);
        let fem = pose.feminine();
        let dcu_z = if fem { 0.088 } else { 0.030 };

        TaperedPrism::new(
            4,
            Vec3::new(0.0, chest_y - 0.048, dcu_z),
            Vec3::new(0.0, chest_y + 0.042, dcu_z),
            pose.scale(0.076),
            0.014,
            pose.scale(0.072),
            0.014,
            carbon,
        )
        .emit(&frame, true, target);
        TaperedPrism::new(
            4,
            Vec3::new(0.0, chest_y + 0.008, dcu_z + 0.012),
            Vec3::new(0.0, chest_y + 0.032, dcu_z + 0.012),
            pose.scale(0.048),
            0.006,
            pose.scale(0.048),
            0.006,
            cyan_led,
        )
        .emit(&frame, true, target);

        match self.style {
            SuitShell::FlightSuit => {
                for side in [-1.0f32, 1.0f32] {
                    AxialLimb::new(
                        Vec3::new(side * pose.scale(0.16), sternum_y - 0.06, -0.012),
                        Vec3::new(side * pose.scale(0.06), chest_y - 0.02, dcu_z),
                        0.014,
                        0.014,
                        trim,
                    )
                    .emit(&frame, true, target);
                }
            }
            SuitShell::SurveySuit => {
                for side in [-1.0f32, 1.0f32] {
                    let sx = side * pose.scale(if fem { 0.140 } else { 0.118 });
                    TaperedPrism::new(4, Vec3::new(sx, waist_y, 0.046), Vec3::new(sx * 1.08, sternum_y - 0.05, -0.015), 0.020, 0.012, 0.022, 0.012, carbon)
                        .emit(&frame, true, target);
                    TaperedPrism::new(4, Vec3::new(sx, chest_y - 0.02, 0.052), Vec3::new(sx, chest_y + 0.02, 0.052), 0.024, 0.014, 0.024, 0.014, amber_led)
                        .emit(&frame, true, target);
                }
            }
            SuitShell::ThermalShell => {
                let strap_z = if fem { 0.080 } else { 0.050 };
                AxialLimb::new(
                    Vec3::new(-pose.scale(0.18), sternum_y - 0.06, -0.018),
                    Vec3::new(pose.scale(0.15), waist_y + 0.02, strap_z),
                    0.018,
                    0.018,
                    amber_led,
                )
                .emit(&frame, true, target);
            }
            SuitShell::ArmoredRig => {
                for side in [-1.0f32, 1.0f32] {
                    let px = side * pose.scale(if fem { 0.192 } else { 0.242 });
                    let pr = if fem { 0.068 } else { 0.092 };
                    TaperedPrism::new(6, Vec3::new(px, sternum_y - 0.12, -0.066), Vec3::new(px * 0.92, sternum_y + 0.01, -0.066), pose.scale(pr), pose.torso_depth(pr), pose.scale(pr * 0.84), pose.torso_depth(pr * 0.85), titanium)
                        .emit(&frame, true, target);
                }
            }
            SuitShell::TechCoverall => {
                TaperedPrism::new(6, Vec3::new(pose.scale(if fem { 0.190 } else { 0.230 }), sternum_y - 0.13, -0.066), Vec3::new(pose.scale(if fem { 0.176 } else { 0.212 }), sternum_y + 0.01, -0.066), pose.scale(0.074), pose.torso_depth(0.080), pose.scale(0.064), pose.torso_depth(0.072), amber_led)
                    .emit(&frame, true, target);
            }
            SuitShell::CommandSuit => {
                TaperedPrism::new(8, Vec3::new(0.0, sternum_y - 0.08, -0.068), Vec3::new(0.0, sternum_y + 0.01, -0.068), pose.scale(0.185), pose.torso_depth(0.108), pose.scale(0.135), pose.torso_depth(0.092), cloth)
                    .emit(&frame, true, target);
            }
        }
    }
}

/// `TorsoSculpture` decorator mounting a dorsal PLSS life-support pack and EVA belt pods (`2` fields).
#[derive(Clone, Copy)]
pub struct PlssPackTorso<S> {
    origin: S,
    style: PlssHarness,
}

impl<S> PlssPackTorso<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, style: PlssHarness) -> Self {
        Self { origin, style }
    }
}

impl<S: TorsoSculpture> TorsoSculpture for PlssPackTorso<S> {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.origin.emit(pose, target);
        let frame = pose.torso_frame();
        let waist_y = pose.torso_span(0.26);
        let chest_y = pose.torso_span(0.54);
        let plss_white = Vec3::new(0.84, 0.88, 0.93);
        let carbon = Vec3::new(0.22, 0.26, 0.32);
        let titanium = Vec3::new(0.64, 0.70, 0.78);
        let cyan = Vec3::new(0.24, 0.88, 0.98);
        let amber = Vec3::new(0.96, 0.72, 0.20);

        TaperedPrism::new(
            4,
            Vec3::new(0.0, waist_y + 0.04, -0.182),
            Vec3::new(0.0, chest_y + 0.10, -0.182),
            pose.scale(0.142),
            0.062,
            pose.scale(0.136),
            0.058,
            plss_white,
        )
        .emit(&frame, true, target);

        match self.style {
            PlssHarness::UmbilicalBelt => {
                let px = pose.scale(0.158);
                TaperedPrism::hexagonal(Vec3::new(-px, waist_y - 0.09, 0.015), Vec3::new(-px, waist_y, 0.015), 0.036, 0.026, cyan)
                    .emit(&frame, true, target);
            }
            PlssHarness::SamplePouchRig => {
                for side in [-1.0_f32, 1.0_f32] {
                    let ox = side * pose.scale(0.078);
                    TaperedPrism::hexagonal(Vec3::new(ox, waist_y + 0.06, -0.228), Vec3::new(ox, chest_y + 0.08, -0.228), 0.036, 0.036, amber)
                        .emit(&frame, true, target);
                }
            }
            PlssHarness::PlssBackpack => {
                TaperedPrism::hexagonal(Vec3::new(0.0, chest_y + 0.10, -0.185), Vec3::new(0.0, chest_y + 0.16, -0.185), pose.scale(0.145), pose.scale(0.145), carbon)
                    .emit(&frame, true, target);
            }
            PlssHarness::JetpackHarness => {
                for side in [-1.0f32, 1.0f32] {
                    let tx = side * pose.scale(0.156);
                    TaperedPrism::hexagonal(Vec3::new(tx, waist_y + 0.02, -0.196), Vec3::new(tx, chest_y + 0.08, -0.196), 0.032, 0.026, titanium)
                        .emit(&frame, true, target);
                }
            }
            PlssHarness::ToolRigBelt => {
                let sx = pose.scale(0.175);
                TaperedPrism::new(4, Vec3::new(sx, waist_y - 0.18, -0.008), Vec3::new(sx, waist_y + 0.01, -0.008), 0.018, 0.032, 0.024, 0.042, titanium)
                    .emit(&frame, true, target);
                TaperedPrism::new(6, Vec3::new(-sx, waist_y - 0.14, 0.012), Vec3::new(-sx, waist_y + 0.01, 0.012), 0.040, 0.048, 0.036, 0.042, carbon)
                    .emit(&frame, true, target);
            }
            PlssHarness::BeaconBelt => {
                let lx = -pose.scale(0.175);
                TaperedPrism::hexagonal(Vec3::new(lx, waist_y - 0.12, 0.018), Vec3::new(lx, waist_y - 0.03, 0.018), 0.030, 0.034, titanium)
                    .emit(&frame, true, target);
                TaperedPrism::hexagonal(Vec3::new(lx, waist_y - 0.09, 0.018), Vec3::new(lx, waist_y - 0.05, 0.018), 0.024, 0.024, cyan)
                    .emit(&frame, true, target);
            }
        }
    }
}
