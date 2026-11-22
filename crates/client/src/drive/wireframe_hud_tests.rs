use glam::{Mat4, Vec2, Vec3};
use world::{
    AttitudeGuidance, BlockMedium, BlockMold, CelestialSystem, CellRole, JetTone, MeshTarget,
    PilotOrder, Quad, SurfaceFrame, VehicleAssembly, VehicleAxes, VehiclePart, WedgeHeading,
};
use crate::attire::YouthAttire;
use crate::avatar::{Avatar as _, AvatarMotor as _, HandGesture, PlanetaryAvatar};
use crate::controls::{ActionBindings, ControlAction, KeyboardControls, LocomotionTarget};
use crate::flight::SurfaceFlight;
use crate::menu::{OpticOverlay, SilentOverlay, WireframeShipOverlay};
use crate::zero_g_flight::ZeroGFlight;

fn quad_rcs_rocket() -> VehicleAssembly {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let att = |heading: WedgeHeading| {
        BlockMold::shape_from_stamp([1, heading.code(), BlockMedium::RocketNozzle.code()])
    };
    let parts = vec![
        VehiclePart::new(
            (0, 0),
            0.5,
            BlockMold::cube().in_medium(BlockMedium::CabinSeat).shape_for(Vec3::NEG_Z, &frame),
        ),
        VehiclePart::new(
            (0, -1),
            0.5,
            BlockMold::cube().in_medium(BlockMedium::FlightConsole).shape_for(Vec3::Z, &frame),
        ),
        VehiclePart::new(
            (0, 0),
            0.0,
            BlockMold::cube().in_medium(BlockMedium::RocketNozzle).shape_for(Vec3::NEG_Z, &frame),
        ),
        VehiclePart::new((-1, 0), 0.5, att(WedgeHeading::East)),
        VehiclePart::new((1, 0), 0.5, att(WedgeHeading::West)),
        VehiclePart::new((0, -2), 0.5, att(WedgeHeading::North)),
        VehiclePart::new((0, 1), 0.5, att(WedgeHeading::South)),
        VehiclePart::new((0, 0), 1.0, BlockMold::cube().in_medium(BlockMedium::QuarriedStone).shape_for(Vec3::Z, &frame)),
        VehiclePart::new((0, -1), 1.0, BlockMold::cube().in_medium(BlockMedium::QuarriedStone).shape_for(Vec3::Z, &frame)),
        VehiclePart::new((0, 0), 1.5, BlockMold::cube().in_medium(BlockMedium::QuarriedStone).shape_for(Vec3::Z, &frame)),
        VehiclePart::new((0, -1), 1.5, BlockMold::cube().in_medium(BlockMedium::QuarriedStone).shape_for(Vec3::Z, &frame)),
    ];
    VehicleAssembly::new(
        Vec3::new(0.25, 10.0, 0.25),
        VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y),
        Vec3::NEG_Z,
        parts,
    )
}

struct WireHudProbe {
    card_quads: usize,
    steer_quads: usize,
    steer_sum: Vec2,
    exhaust_cores: usize,
    min_pt: Vec3,
    max_pt: Vec3,
}

impl WireHudProbe {
    const fn empty() -> Self {
        Self {
            card_quads: 0,
            steer_quads: 0,
            steer_sum: Vec2::ZERO,
            exhaust_cores: 0,
            min_pt: Vec3::splat(f32::INFINITY),
            max_pt: Vec3::splat(f32::NEG_INFINITY),
        }
    }
}

impl MeshTarget for WireHudProbe {
    fn push_quad(&mut self, quad: Quad) {
        let glow = quad.radiate(|g| g);
        quad.transfer(|v0, v1, v2, v3, _, col| {
            let c = Vec2::new((v0.x + v2.x) * 0.5, (v0.y + v2.y) * 0.5);
            if c.x < -0.34 && c.y < -0.28 {
                self.card_quads += 1;
                if glow > 0.97 && (col - Vec3::new(0.44, 1.00, 0.52)).length() < 0.02 {
                    self.steer_sum += c;
                    self.steer_quads += 1;
                }
                if glow > 0.98 && col.x > 0.80 && col.y > 0.90 {
                    self.exhaust_cores += 1;
                }
            }
            for v in [v0, v1, v2, v3] {
                self.min_pt = self.min_pt.min(v);
                self.max_pt = self.max_pt.max(v);
            }
        });
    }
}

struct LiftAndRollControls;

impl KeyboardControls for LiftAndRollControls {
    fn drive_locomotion(&self, _fwd: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        target.jump();
        target.roll(1.0);
    }
}

#[test]
fn test_hull_wireframe_contrasts_exhaust_jets_with_opposing_steer_vector() {
    let craft = quad_rcs_rocket();

    let idle = craft.sample_wireframe(&PilotOrder::idle());
    let (mut fuselage_prisms, mut idle_pods) = (0usize, 0usize);
    idle.visit_cells(|role, _, _| match role {
        CellRole::Fuselage => fuselage_prisms += 1,
        CellRole::AttitudePod => idle_pods += 1,
        _ => {}
    });
    let mut idle_jets = 0usize;
    idle.visit_jets(|_, _, _, _| idle_jets += 1);
    assert_eq!(fuselage_prisms, 1, "Multi-block hull must collapse into one uncluttered fuselage prism");
    assert_eq!(idle_pods, 4);
    assert_eq!(idle_jets, 0);
    idle.transfer_steer(|_, steer, lit| assert!(!lit && steer == Vec3::ZERO));

    let lift = craft.sample_wireframe(&PilotOrder::from_channels(
        Vec3::Y,
        Vec3::ZERO,
        Vec3::ZERO,
        AttitudeGuidance::Engaged,
    ));
    let mut bell_dirs = Vec::new();
    lift.visit_jets(|tone, _, dir, thr| {
        if tone == JetTone::BellAmber && thr > 0.95 {
            bell_dirs.push(dir);
        }
    });
    assert_eq!(bell_dirs.len(), 1);
    assert!(bell_dirs[0].y < -0.95, "Main bell exhaust must shoot downward (-Y)");
    lift.transfer_steer(|_, steer, lit| {
        assert!(lit && steer.y > 0.50, "STEER vector must point upward (+Y) opposite bell exhaust");
    });

    let strafe_right = craft.sample_wireframe(&PilotOrder::from_channels(
        Vec3::X,
        Vec3::ZERO,
        Vec3::ZERO,
        AttitudeGuidance::Engaged,
    ));
    let mut rcs_x = 0.0_f32;
    strafe_right.visit_jets(|tone, _, dir, _| {
        if tone == JetTone::VernierCyan {
            rcs_x += dir.x;
        }
    });
    assert!(rcs_x < -1.5, "Exhaust jets must shoot leftward (-X) when strafing right");
    strafe_right.transfer_steer(|_, steer, _| {
        assert!(steer.x > 0.50, "STEER vector must point rightward (+X) opposite leftward exhaust");
    });

    for rot in [Vec3::NEG_Y, Vec3::Z] {
        let spin = craft.sample_wireframe(&PilotOrder::from_channels(
            Vec3::ZERO,
            rot,
            Vec3::ZERO,
            AttitudeGuidance::Manual,
        ));
        let (mut active_pods, mut active_bells) = (0usize, 0usize);
        spin.visit_cells(|role, _, _| match role {
            CellRole::ActivePod => active_pods += 1,
            CellRole::ActiveMainBell => active_bells += 1,
            _ => {}
        });
        assert!(active_pods >= 2 && active_bells == 0);
        spin.transfer_steer(|_, steer, lit| {
            assert!(lit && steer.x > 0.12, "Clockwise yaw/roll must project rightward STEER cue, got {steer:?}");
        });
    }
}

#[test]
fn test_wireframe_ship_overlay_syncs_with_camera_orientation_inside_hud_card() {
    let craft = quad_rcs_rocket();
    let ux = 0.5625;
    let wf = craft.sample_wireframe(&PilotOrder::from_channels(
        Vec3::X,
        Vec3::ZERO,
        Vec3::ZERO,
        AttitudeGuidance::Engaged,
    ));
    let proj = Mat4::perspective_rh_gl(1.1, 16.0 / 9.0, 0.1, 500.0);
    let fore_vp = proj * Mat4::look_to_rh(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y);
    let aft_vp = proj * Mat4::look_to_rh(Vec3::ZERO, Vec3::Z, Vec3::Y);

    let mut fore_probe = WireHudProbe::empty();
    let mut aft_probe = WireHudProbe::empty();
    WireframeShipOverlay::new(SilentOverlay, wf, fore_vp, ux).emit(&mut fore_probe);
    WireframeShipOverlay::new(SilentOverlay, wf, aft_vp, ux).emit(&mut aft_probe);

    assert!(fore_probe.exhaust_cores >= 2 && aft_probe.exhaust_cores >= 2);
    assert!(fore_probe.steer_quads >= 3 && aft_probe.steer_quads >= 3);
    let fore_sx = fore_probe.steer_sum.x / (fore_probe.steer_quads as f32);
    let aft_sx = aft_probe.steer_sum.x / (aft_probe.steer_quads as f32);
    assert!(
        fore_sx > -0.60 && aft_sx < -0.70,
        "Opposite side cameras must mirror screen-space STEER X ({fore_sx:.3} vs {aft_sx:.3})"
    );

    let wf_fore = craft.sample_wireframe(&PilotOrder::from_channels(
        Vec3::Z,
        Vec3::ZERO,
        Vec3::ZERO,
        AttitudeGuidance::Engaged,
    ));
    let nose_vp = proj * Mat4::look_to_rh(Vec3::ZERO, Vec3::Y, Vec3::Z);
    let belly_vp = proj * Mat4::look_to_rh(Vec3::ZERO, Vec3::NEG_Y, Vec3::NEG_Z);
    let mut nose_probe = WireHudProbe::empty();
    let mut belly_probe = WireHudProbe::empty();
    WireframeShipOverlay::new(SilentOverlay, wf_fore, nose_vp, ux).emit(&mut nose_probe);
    WireframeShipOverlay::new(SilentOverlay, wf_fore, belly_vp, ux).emit(&mut belly_probe);
    let nose_sy = nose_probe.steer_sum.y / (nose_probe.steer_quads as f32);
    let belly_sy = belly_probe.steer_sum.y / (belly_probe.steer_quads as f32);
    assert!(
        belly_sy > nose_sy + 0.08,
        "Nose vs Belly pitch flip must invert screen-space STEER Y ({nose_sy:.3} vs {belly_sy:.3})"
    );
    assert!(
        fore_probe.min_pt.x >= -0.955
            && fore_probe.max_pt.x <= -0.340
            && fore_probe.min_pt.y >= -0.935
            && fore_probe.max_pt.y <= -0.290,
        "3D wireframe HUD must remain strictly inside bottom-left card bounds: {:?}..{:?}",
        fore_probe.min_pt,
        fore_probe.max_pt
    );
}

#[test]
fn test_rocket_cameras_render_3d_wireframe_hud_across_nose_belly_and_side_channels() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let av = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    for med in [BlockMedium::CantileverRig, BlockMedium::RocketNozzle] {
        assert!(world.place_molded_surface_block(
            Vec3::new(0.25, g0 + 1.5, 1.25),
            crane_dir,
            av,
            BlockMold::cube().in_medium(med),
        ));
    }
    for (pos, dir, mold) in [
        (Vec3::new(1.25, g0 + 2.90, 0.25), Vec3::NEG_Y, BlockMold::cube().turn_clockwise().in_medium(BlockMedium::FlightConsole)),
        (Vec3::new(1.25, g0 + 1.75, 0.75), Vec3::NEG_Z, BlockMold::cube().in_medium(BlockMedium::CabinSeat)),
        (Vec3::new(1.25, g0 + 1.75, 1.25), Vec3::NEG_Z, BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle)),
        (Vec3::new(1.25, g0 + 2.90, 0.75), Vec3::NEG_Y, BlockMold::cube().in_medium(BlockMedium::RocketCamera)),
        (Vec3::new(1.25, g0 + 2.90, 1.25), Vec3::NEG_Y, BlockMold::cube().next_form().next_form().in_medium(BlockMedium::RocketCamera)),
    ] {
        assert!(world.place_molded_surface_block(pos, dir, av, mold));
    }

    let spawn = world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0);
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
        spawn,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        YouthAttire::squire(),
        ActionBindings::seeded(),
    );
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
    assert!(pilot.use_right_hand(world.as_mut()));
    pilot.step_motion(1.0 / 60.0, &LiftAndRollControls, world.as_ref());

    let next_cam = ActionBindings::seeded().trigger_for(ControlAction::MoldRight);
    for label in ["nose", "belly", "side"] {
        let mut probe = WireHudProbe::empty();
        pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut probe);
        assert!(
            probe.card_quads > 30 && probe.exhaust_cores >= 1 && probe.steer_quads >= 3,
            "{label} camera must render 3D wireframe, EXHAUST, and STEER >"
        );
        assert!(pilot.dispatch_trigger(next_cam));
    }

    let mut cockpit = WireHudProbe::empty();
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut cockpit);
    assert_eq!(cockpit.card_quads, 0, "Cockpit view must remain clear of hull-camera wireframe card");
}
