mod probe;

use glam::{Mat4, Vec3};
use world::{
    AttitudeGuidance, BlockMedium, BlockMold, CelestialSystem, CellRole, JetTone, PilotOrder,
};
use crate::attire::YouthAttire;
use crate::avatar::{Avatar as _, AvatarMotor as _, HandGesture, PlanetaryAvatar};
use crate::controls::{ActionBindings, ControlAction};
use crate::flight::SurfaceFlight;
use crate::menu::{OpticOverlay, SilentOverlay, WireframeShipOverlay};
use crate::zero_g_flight::ZeroGFlight;
use self::probe::{CraftFixture, LiftAndRollControls, QuadRcsCraft, WireHudProbe};

#[test]
fn test_idle_hull_wireframe_collapses_into_single_unlit_fuselage_prism() {
    let idle = QuadRcsCraft::new().assembly().sample_wireframe(&PilotOrder::idle());
    let (mut outer_edges, mut idle_pods, mut idle_jets) = (0usize, 0usize, 0usize);
    idle.visit_edges(|_, _| outer_edges += 1);
    idle.visit_cells(|role, _, _| {
        if role == CellRole::AttitudePod {
            idle_pods += 1;
        }
    });
    idle.visit_jets(|_, _, _, _| idle_jets += 1);
    assert_eq!((outer_edges, idle_pods, idle_jets), (12, 4, 0));
    idle.transfer_steer(|_, steer, lit| assert!(!lit && steer == Vec3::ZERO));
}

#[test]
fn test_main_bell_and_vernier_burns_oppose_projected_steer_vector() {
    let craft = QuadRcsCraft::new().assembly();
    let lift = craft.sample_wireframe(&PilotOrder::from_channels(
        Vec3::Y,
        Vec3::ZERO,
        Vec3::ZERO,
        AttitudeGuidance::Engaged,
    ));
    let mut bell_y = 0.0_f32;
    lift.visit_jets(|tone, _, dir, thr| {
        if tone == JetTone::BellAmber && thr > 0.95 {
            bell_y = dir.y;
        }
    });
    assert!(bell_y < -0.95);
    lift.transfer_steer(|_, steer, lit| assert!(lit && steer.y > 0.50));

    let strafe = craft.sample_wireframe(&PilotOrder::from_channels(
        Vec3::X,
        Vec3::ZERO,
        Vec3::ZERO,
        AttitudeGuidance::Engaged,
    ));
    let mut rcs_x = 0.0_f32;
    strafe.visit_jets(|tone, _, dir, _| {
        if tone == JetTone::VernierCyan {
            rcs_x += dir.x;
        }
    });
    assert!(rcs_x < -1.5);
    strafe.transfer_steer(|_, steer, _| assert!(steer.x > 0.50));
}

#[test]
fn test_manual_attitude_burns_highlight_only_vernier_pods() {
    let craft = QuadRcsCraft::new().assembly();
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
        spin.transfer_steer(|_, steer, lit| assert!(lit && steer.x > 0.12));
    }
}

#[test]
fn test_wireframe_ship_overlay_mirrors_and_inverts_with_camera_axes() {
    let craft = QuadRcsCraft::new().assembly();
    let ux = 0.5625;
    let proj = Mat4::perspective_rh_gl(1.1, 16.0 / 9.0, 0.1, 500.0);
    let wf_side = craft.sample_wireframe(&PilotOrder::from_channels(
        Vec3::X,
        Vec3::ZERO,
        Vec3::ZERO,
        AttitudeGuidance::Engaged,
    ));
    let (mut fore_probe, mut aft_probe) = (WireHudProbe::empty(), WireHudProbe::empty());
    WireframeShipOverlay::new(SilentOverlay, wf_side, proj * Mat4::look_to_rh(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y), ux)
        .emit(&mut fore_probe);
    WireframeShipOverlay::new(SilentOverlay, wf_side, proj * Mat4::look_to_rh(Vec3::ZERO, Vec3::Z, Vec3::Y), ux)
        .emit(&mut aft_probe);
    assert!(fore_probe.active_telemetry(2) && aft_probe.active_telemetry(2));
    assert!(fore_probe.mirrors_steer_x_against(&aft_probe) && fore_probe.confined_within_card());

    let wf_fore = craft.sample_wireframe(&PilotOrder::from_channels(
        Vec3::Z,
        Vec3::ZERO,
        Vec3::ZERO,
        AttitudeGuidance::Engaged,
    ));
    let (mut nose_probe, mut belly_probe) = (WireHudProbe::empty(), WireHudProbe::empty());
    WireframeShipOverlay::new(SilentOverlay, wf_fore, proj * Mat4::look_to_rh(Vec3::ZERO, Vec3::Y, Vec3::Z), ux)
        .emit(&mut nose_probe);
    WireframeShipOverlay::new(SilentOverlay, wf_fore, proj * Mat4::look_to_rh(Vec3::ZERO, Vec3::NEG_Y, Vec3::NEG_Z), ux)
        .emit(&mut belly_probe);
    assert!(nose_probe.inverts_steer_y_against(&belly_probe));
}

#[test]
fn test_rocket_cameras_render_wireframe_hud_and_clear_cockpit() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let av = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    for med in [BlockMedium::CantileverRig, BlockMedium::RocketNozzle] {
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, av, BlockMold::cube().in_medium(med)));
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
    pilot.step_motion(1.0 / 60.0, &LiftAndRollControls::new(), world.as_ref());

    let next_cam = ActionBindings::seeded().trigger_for(ControlAction::MoldRight);
    for _ in 0..3 {
        let mut probe = WireHudProbe::empty();
        pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut probe);
        assert!(probe.active_telemetry(1));
        assert!(pilot.dispatch_trigger(next_cam));
    }

    let mut cockpit = WireHudProbe::empty();
    pilot.emit_reticle_for_aspect(16.0 / 9.0, &mut cockpit);
    assert!(cockpit.blank_in_cockpit());
}
