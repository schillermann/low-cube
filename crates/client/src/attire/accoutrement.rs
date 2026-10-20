use glam::Vec3;
use world::MeshTarget;
use super::garments::{GirdleStyle, TunicStyle};
use super::limb::{AxialLimb, FacetedSolid, LimbFrame, TaperedPrism};
use super::palette::PigmentSwatch;
use super::pose::BodyPose;

/// Detailed low-poly Polygon-style body armor, capes, quivers, backpacks, robes, and waist gear.
pub struct PolygonAccoutrement;

impl PolygonAccoutrement {
    pub fn emit_tunic_details(
        style: TunicStyle,
        pose: &BodyPose,
        frame: &LimbFrame,
        pigment: PigmentSwatch,
        target: &mut dyn MeshTarget,
    ) {
        let crouch = pose.crouch_compression();
        let hip_y = pose.hip_elevation();
        let waist_y = pose.torso_span(0.26 - crouch * 0.12);
        let chest_y = pose.torso_span(0.54 - crouch * 0.30);
        let sternum_y = pose.torso_span(0.72 - crouch * 0.42);
        let cloth = pigment.cloth();
        let trim = pigment.trim();
        let steel = Vec3::new(0.66, 0.70, 0.76);
        let dark_steel = Vec3::new(0.36, 0.39, 0.44);
        let brass = Vec3::new(0.86, 0.70, 0.26);
        let leather = Vec3::new(0.42, 0.26, 0.15);

        match style {
            TunicStyle::LinenDoublet => {
                // Noble gold chain of office across chest and tailored peplum tabs
                for side in [-1.0f32, 1.0f32] {
                    AxialLimb::new(
                        Vec3::new(side * pose.scale(0.18), sternum_y - 0.08, -0.010),
                        Vec3::new(0.0, chest_y - 0.06, 0.056),
                        0.014,
                        0.016,
                        brass,
                    )
                    .emit(frame, true, target);
                }
            }
            TunicStyle::QuiltedJerkin => {
                // Woodsman leather suspenders with brass buckles and heavy work apron
                for side in [-1.0f32, 1.0f32] {
                    let sx = side * pose.scale(0.115);
                    TaperedPrism::new(4, Vec3::new(sx, waist_y, 0.048), Vec3::new(sx * 1.15, sternum_y - 0.06, -0.015), 0.022, 0.012, 0.024, 0.012, leather)
                        .emit(frame, true, target);
                    TaperedPrism::new(4, Vec3::new(sx, chest_y - 0.02, 0.054), Vec3::new(sx, chest_y + 0.02, 0.054), 0.026, 0.014, 0.026, 0.014, brass)
                        .emit(frame, true, target);
                }
            }
            TunicStyle::PeasantShirt => {
                // Ranger diagonal chest baldric strap and long split trail coat tails
                AxialLimb::new(
                    Vec3::new(-pose.scale(0.19), sternum_y - 0.06, -0.018),
                    Vec3::new(pose.scale(0.15), waist_y + 0.02, 0.048),
                    0.022,
                    0.022,
                    leather,
                )
                .emit(frame, true, target);
                for side in [-1.0f32, 1.0f32] {
                    let tx = side * pose.scale(0.11);
                    TaperedPrism::new(6, Vec3::new(tx, hip_y - 0.24, -0.055), Vec3::new(tx, hip_y + 0.02, -0.040), pose.scale(0.085), pose.torso_depth(0.085), pose.scale(0.095), pose.torso_depth(0.095), trim)
                        .emit(frame, true, target);
                }
            }
            TunicStyle::PlateCuirass => {
                // Armored Knight sculpted steel plackart, multi-tiered pauldrons, and flowing back cape
                TaperedPrism::new(6, Vec3::new(0.0, waist_y + 0.02, -0.020), Vec3::new(0.0, chest_y + 0.04, -0.028), pose.scale(0.165), pose.torso_depth(0.112), pose.scale(0.228), pose.torso_depth(0.132), steel)
                    .emit(frame, true, target);
                for side in [-1.0f32, 1.0f32] {
                    let px = side * pose.scale(0.245);
                    TaperedPrism::new(6, Vec3::new(px, sternum_y - 0.14, -0.066), Vec3::new(px * 0.92, sternum_y + 0.01, -0.066), pose.scale(0.095), pose.torso_depth(0.098), pose.scale(0.078), pose.torso_depth(0.084), steel)
                        .emit(frame, true, target);
                    TaperedPrism::new(6, Vec3::new(px * 1.04, sternum_y - 0.08, -0.066), Vec3::new(px * 0.96, sternum_y - 0.02, -0.066), pose.scale(0.102), pose.torso_depth(0.102), pose.scale(0.088), pose.torso_depth(0.090), brass)
                        .emit(frame, true, target);
                }
                TaperedPrism::new(6, Vec3::new(0.0, 0.28, -0.22), Vec3::new(0.0, sternum_y - 0.04, -0.155), pose.scale(0.26), 0.028, pose.scale(0.19), 0.022, cloth)
                    .emit(frame, true, target);
            }
            TunicStyle::ForgeApron => {
                // Blacksmith heavy heat-shielded leather apron from chest to knees + riveted shoulder guard
                TaperedPrism::new(6, Vec3::new(0.0, hip_y - 0.24, 0.045), Vec3::new(0.0, chest_y + 0.05, 0.042), pose.scale(0.185), 0.025, pose.scale(0.155), 0.022, leather)
                    .emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(pose.scale(0.235), sternum_y - 0.14, -0.066), Vec3::new(pose.scale(0.215), sternum_y + 0.02, -0.066), pose.scale(0.095), pose.torso_depth(0.095), pose.scale(0.082), pose.torso_depth(0.085), dark_steel)
                    .emit(frame, true, target);
            }
            TunicStyle::HeraldicTabard => {
                // Sentinel chainmail skirt, heraldic front/back tabard panels, and steel gorget
                TaperedPrism::new(8, Vec3::new(0.0, hip_y - 0.16, -0.034), Vec3::new(0.0, hip_y + 0.04, -0.032), pose.scale(0.212), pose.torso_depth(0.132), pose.scale(0.198), pose.torso_depth(0.124), dark_steel)
                    .emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(0.0, hip_y - 0.20, 0.052), Vec3::new(0.0, waist_y + 0.02, 0.046), pose.scale(0.115), 0.016, pose.scale(0.105), 0.014, cloth)
                    .emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, sternum_y - 0.08, -0.068), Vec3::new(0.0, sternum_y + 0.01, -0.068), pose.scale(0.185), pose.torso_depth(0.108), pose.scale(0.135), pose.torso_depth(0.092), steel)
                    .emit(frame, true, target);
            }
        }
    }

    pub fn emit_girdle_details(
        style: GirdleStyle,
        pose: &BodyPose,
        frame: &LimbFrame,
        target: &mut dyn MeshTarget,
    ) {
        let crouch = pose.crouch_compression();
        let waist_y = pose.torso_span(0.26 - crouch * 0.12);
        let chest_y = pose.torso_span(0.54 - crouch * 0.30);
        let leather = Vec3::new(0.38, 0.23, 0.13);
        let dark_hide = Vec3::new(0.25, 0.15, 0.09);
        let brass = Vec3::new(0.84, 0.68, 0.24);
        let steel = Vec3::new(0.62, 0.66, 0.72);
        let wood = Vec3::new(0.52, 0.35, 0.18);
        let feather = Vec3::new(0.92, 0.90, 0.82);

        match style {
            GirdleStyle::KnottedCord => {
                let px = pose.scale(0.158);
                TaperedPrism::hexagonal(Vec3::new(-px, waist_y - 0.09, 0.015), Vec3::new(-px, waist_y, 0.015), 0.042, 0.026, Vec3::new(0.64, 0.18, 0.22))
                    .emit(frame, true, target);
            }
            GirdleStyle::PouchedGirdle => {
                // Ranger 3D Back Quiver with 3 feathered arrows
                TaperedPrism::hexagonal(Vec3::new(pose.scale(0.06), waist_y + 0.04, -0.175), Vec3::new(pose.scale(0.11), chest_y + 0.10, -0.185), 0.048, 0.058, leather)
                    .emit(frame, true, target);
                for i in -1..=1 {
                    let ax = pose.scale(0.11) + (i as f32) * 0.022;
                    TaperedPrism::hexagonal(Vec3::new(ax, chest_y + 0.08, -0.185), Vec3::new(ax + 0.015, chest_y + 0.25, -0.195), 0.007, 0.016, feather)
                        .emit(frame, true, target);
                }
            }
            GirdleStyle::UnbeltedWaist => {
                // Woodsman 3D Framed Timber Backpack with bedroll on the back
                TaperedPrism::new(4, Vec3::new(0.0, waist_y + 0.02, -0.195), Vec3::new(0.0, chest_y + 0.08, -0.195), pose.scale(0.155), 0.075, pose.scale(0.145), 0.070, leather)
                    .emit(frame, true, target);
                TaperedPrism::hexagonal(Vec3::new(0.0, chest_y + 0.08, -0.195), Vec3::new(0.0, chest_y + 0.15, -0.195), pose.scale(0.165), pose.scale(0.165), wood)
                    .emit(frame, true, target);
            }
            GirdleStyle::KnightlyBelt => {
                // Armored hip tassets flanking the waist
                for side in [-1.0f32, 1.0f32] {
                    let tx = side * pose.scale(0.175);
                    TaperedPrism::new(4, Vec3::new(tx, waist_y - 0.16, -0.030), Vec3::new(tx, waist_y - 0.01, -0.030), 0.045, 0.085, 0.038, 0.080, steel)
                        .emit(frame, true, target);
                }
            }
            GirdleStyle::ForgeBelt => {
                // Blacksmith hanging forged iron tongs and heavy tool holster at the hips
                let sx = pose.scale(0.175);
                TaperedPrism::new(4, Vec3::new(sx, waist_y - 0.20, -0.008), Vec3::new(sx, waist_y + 0.01, -0.008), 0.018, 0.032, 0.024, 0.042, steel)
                    .emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(-sx, waist_y - 0.14, 0.012), Vec3::new(-sx, waist_y + 0.01, 0.012), 0.044, 0.052, 0.038, 0.046, dark_hide)
                    .emit(frame, true, target);
            }
            GirdleStyle::GarrisonBelt => {
                // Guard hanging brass lantern at left hip
                let lx = -pose.scale(0.175);
                TaperedPrism::hexagonal(Vec3::new(lx, waist_y - 0.13, 0.018), Vec3::new(lx, waist_y - 0.03, 0.018), 0.032, 0.036, brass)
                    .emit(frame, true, target);
                TaperedPrism::hexagonal(Vec3::new(lx, waist_y - 0.10, 0.018), Vec3::new(lx, waist_y - 0.05, 0.018), 0.026, 0.026, Vec3::new(0.98, 0.78, 0.28))
                    .emit(frame, true, target);
            }
        }
    }
}
