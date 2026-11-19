use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{BlockMedium, BlockMold, CelestialSystem, LunarWorld};
use crate::attire::YouthAttire;
use crate::avatar::{Avatar as _, AvatarMotor as _, AvatarOptics as _, HandGesture, PlanetaryAvatar};
use crate::black_box::PresenceProbe;
use crate::controls::{ActionBindings, KeyMotion, KeyboardControls};
use crate::flight::SurfaceFlight;
use crate::zero_g_flight::ZeroGFlight;

fn avatar_pos(avatar: &dyn crate::avatar::Avatar) -> Vec3 {
    let mut probe = PresenceProbe::at_origin();
    avatar.report_presence(&mut probe);
    probe.position()
}

#[test]
fn test_nose_first_moon_impact_keeps_rocket_and_pilot_outside_moon_and_settles_upright() {
    let moon = CelestialSystem::moon_center(32);
    let cam_pos = |a: &PlanetaryAvatar| a.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let (av, crane_dir) = (Vec3::new(0.25, g0, 1.75), Vec3::new(0.0, -0.8, -0.6).normalize());
    for med in [BlockMedium::CantileverRig, BlockMedium::RocketNozzle] {
        assert!(world.place_molded_surface_block(
            Vec3::new(0.25, g0 + 1.5, 1.25),
            crane_dir,
            av,
            BlockMold::cube().in_medium(med),
        ));
    }
    assert!(world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, -1, false));
    for (eye, dir, mold) in [
        (Vec3::new(1.25, g0 + 2.40, 0.25), Vec3::NEG_Y, BlockMold::cube().in_medium(BlockMedium::FlightConsole)),
        (Vec3::new(1.25, g0 + 1.75, 0.75), Vec3::NEG_Z, BlockMold::cube().in_medium(BlockMedium::CabinSeat)),
        (Vec3::new(1.25, g0 + 1.75, 1.25), Vec3::NEG_Z, BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle)),
        (Vec3::new(1.25, g0 + 1.75, 1.75), Vec3::NEG_Z, BlockMold::cube().in_medium(BlockMedium::RocketCamera)),
    ] {
        assert!(world.place_molded_surface_block(eye, dir, av, mold));
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
    let (mut pitch, mut burn, idle) = (KeyMotion::idle(), KeyMotion::idle(), KeyMotion::idle());
    pitch.register_key(KeyCode::Space, true);
    pitch.register_key(KeyCode::KeyW, true);
    burn.register_key(KeyCode::Space, true);
    for tick in 0..1300 {
        let ctrl = if tick < 30 { &pitch } else if tick < 1090 { &burn } else { &idle };
        pilot.step_motion(1.0 / 120.0, ctrl, world.as_ref());
        let (rel_av, rel_cam) = (avatar_pos(&pilot) - moon, cam_pos(&pilot) - moon);
        assert!(
            rel_av.abs().max_element() >= LunarWorld::HALF_SIZE - 0.5
                && rel_cam.abs().max_element() >= LunarWorld::HALF_SIZE - 0.5,
            "Rocket and camera must never enter Moon interior: av={rel_av:?} cam={rel_cam:?}"
        );
    }
    assert!(
        (avatar_pos(&pilot) - moon).abs().max_element() <= LunarWorld::HALF_SIZE + 3.0,
        "Rocket must settle cleanly on the Moon surface"
    );
}

#[test]
fn test_rocket_camera_cycles_three_aerospace_modes_and_rejects_compact_field_camera_on_rocket() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let (av, crane_dir) = (Vec3::new(0.25, g0, 1.75), Vec3::new(0.0, -0.8, -0.6).normalize());
    for med in [BlockMedium::CantileverRig, BlockMedium::RocketNozzle] {
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, av, BlockMold::cube().in_medium(med)));
    }
    assert!(world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, -1, false));
    for (eye, dir, mold) in [
        (Vec3::new(1.25, g0 + 2.40, 0.25), Vec3::NEG_Y, BlockMold::cube().turn_clockwise().in_medium(BlockMedium::FlightConsole)),
        (Vec3::new(1.25, g0 + 1.75, 0.75), Vec3::NEG_Z, BlockMold::cube().in_medium(BlockMedium::CabinSeat)),
        (Vec3::new(1.25, g0 + 1.75, 1.25), Vec3::NEG_Z, BlockMold::cube().in_medium(BlockMedium::FieldCamera)),
    ] {
        assert!(world.place_molded_surface_block(eye, dir, av, mold));
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
    let inv = pilot.view_projection_matrix(1.0).inverse();
    let eye = inv.project_point3(Vec3::ZERO);
    let dir = (inv.project_point3(Vec3::new(0.0, 0.0, 0.5)) - eye).normalize();
    assert!(dir.y.abs() < 0.4, "Compact FieldCamera must not act as rocket nose camera, got {dir:?}");

    let mold0 = BlockMold::cube().in_medium(BlockMedium::RocketCamera);
    let mold1 = mold0.next_form();
    let mold2 = mold1.next_form();
    assert_eq!(mold2.next_form(), mold0);
    assert!(!mold0.wall_mounted_pod() && !mold1.wall_mounted_pod() && mold2.wall_mounted_pod());
}
