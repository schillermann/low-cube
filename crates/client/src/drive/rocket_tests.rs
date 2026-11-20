use glam::Vec3;
use sanctuary::{HostedPlanet, JoinTarget, RemoteLink, RemotePlanet, SanctuaryHost};
use winit::keyboard::KeyCode;
use world::{BlockMedium, BlockMold, CelestialSystem, LunarWorld, PlanetaryWorld, WorldEarthworks as _};
use crate::attire::YouthAttire;
use crate::avatar::{Avatar as _, AvatarMotor as _, AvatarOptics as _, HandGesture, PlanetaryAvatar};
use crate::black_box::PresenceProbe;
use crate::controls::{ActionBindings, KeyMotion, KeyboardControls};
use crate::flight::SurfaceFlight;
use crate::zero_g_flight::ZeroGFlight;

fn build_crane_rocket(world: &mut dyn PlanetaryWorld) -> f32 {
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let av_pos = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    assert!(world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.5, 1.25),
        crane_dir,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.5, 1.25),
        crane_dir,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::RocketNozzle),
    ));
    assert!(world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, -1, false));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 2.40, 0.25),
        Vec3::NEG_Y,
        av_pos,
        BlockMold::cube().turn_clockwise().in_medium(BlockMedium::FlightConsole),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 0.75),
        Vec3::NEG_Z,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::CabinSeat),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 1.25),
        Vec3::NEG_Z,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::CabinSeat),
    ));
    g0
}

fn avatar_pos(avatar: &dyn crate::avatar::Avatar) -> Vec3 {
    let mut probe = PresenceProbe::at_origin();
    avatar.report_presence(&mut probe);
    probe.position()
}

#[test]
fn test_crane_rocket_launches_without_wheels_coasts_zero_g_and_allows_eva_spacewalk() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let g0 = build_crane_rocket(world.as_mut());

    let spawn = world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0);
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
        spawn,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        YouthAttire::squire(),
        ActionBindings::seeded(),
    );
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
    assert!(pilot.use_right_hand(world.as_mut()), "FlightConsole must release rocket without wheels");

    let mut burn = KeyMotion::idle();
    burn.register_key(KeyCode::Space, true);
    let dt = 1.0 / 120.0;
    for _ in 0..600 {
        pilot.step_motion(dt, &burn, world.as_ref());
    }
    let space_pos = avatar_pos(&pilot);
    assert!(
        space_pos.y > g0 + 120.0,
        "Rocket must ascend into deep space (>120m above ground), got y={}",
        space_pos.y
    );
    assert!(
        world.apply_pull(space_pos, 1.0).length() < 1e-4,
        "Spacecraft at y={} must be in 0.0g vacuum",
        space_pos.y
    );

    let idle = KeyMotion::idle();
    let p_a = avatar_pos(&pilot);
    for _ in 0..30 {
        pilot.step_motion(dt, &idle, world.as_ref());
    }
    let p_b = avatar_pos(&pilot);
    for _ in 0..30 {
        pilot.step_motion(dt, &idle, world.as_ref());
    }
    let p_c = avatar_pos(&pilot);
    let v1 = (p_b - p_a).length() / 0.25;
    let v2 = (p_c - p_b).length() / 0.25;
    assert!((v1 - v2).abs() < 0.05 && v1 > 5.0, "Inertial drift in 0.0g must preserve velocity ({v1} vs {v2})");

    assert!(pilot.use_right_hand(world.as_mut()), "Pilot must release FlightConsole in orbit");
    let mut step_out = KeyMotion::idle();
    step_out.register_key(KeyCode::KeyD, true);
    pilot.step_motion(dt, &step_out, world.as_ref());
    for _ in 0..60 {
        pilot.step_motion(dt, &idle, world.as_ref());
    }
    let eva_pos = avatar_pos(&pilot);
    assert!(
        eva_pos.y > g0 + 110.0,
        "EVA astronaut must float weightlessly in space without falling to Earth, got y={}",
        eva_pos.y
    );

    let reboarded = world
        .board_vehicle(eva_pos, (space_pos - eva_pos).normalize(), true)
        .expect("EVA astronaut must re-board floating spacecraft in 0.0g");
    let moon_center = CelestialSystem::moon_center(32);
    let mut moon_lander = reboarded;
    let landing_approach = moon_center + Vec3::new(0.25, LunarWorld::HALF_SIZE + 0.01, 0.25);
    moon_lander.relocate(landing_approach, Vec3::Y);
    assert!(world.park_vehicle(moon_lander), "Spacecraft must park onto lunar surface grid");
    let parked_top = world.constrain_to_surface(
        moon_center + Vec3::new(0.25, LunarWorld::HALF_SIZE + 4.0, 0.25),
        0.0,
    );
    assert!(
        parked_top.y > moon_center.y + LunarWorld::HALF_SIZE + 0.2,
        "Parked rocket blocks must rest physically on the Moon surface, got y={}",
        parked_top.y
    );
}

#[test]
fn test_lunar_pedestrian_leap_exceeds_earth_jump_and_coop_reaches_moon() {
    let host = SanctuaryHost::bind(
        "127.0.0.1:0",
        CelestialSystem::barren_authoritative(32, 1.0),
        32,
    )
    .expect("bind host");
    let addr = host.local_addr().expect("addr");
    let mut host_world = HostedPlanet::new(host);
    let mut guest_world = RemotePlanet::new(
        CelestialSystem::hollow(32, 1.0),
        RemoteLink::bind(JoinTarget::Direct(addr)).expect("guest link"),
        std::collections::VecDeque::new(),
    );
    for _ in 0..8 {
        let _ = guest_world.step_surface(0.03);
        let _ = host_world.step_surface(0.03);
    }

    let earth_start = host_world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 1.0);
    let mut earth_jumper = PlanetaryAvatar::with_attire_and_bindings(
        earth_start,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        YouthAttire::squire(),
        ActionBindings::seeded(),
    );
    let mut jump = KeyMotion::idle();
    jump.register_key(KeyCode::Space, true);
    let idle = KeyMotion::idle();
    let dt = 1.0 / 120.0;
    let mut earth_peak = 0.0f32;
    for i in 0..90 {
        earth_jumper.step_motion(dt, if i == 0 { &jump } else { &idle }, &host_world);
        earth_peak = earth_peak.max(avatar_pos(&earth_jumper).y - earth_start.y);
    }

    let moon_center = CelestialSystem::moon_center(32);
    let moon_start = host_world.constrain_to_surface(
        moon_center + Vec3::new(0.25, LunarWorld::HALF_SIZE + 4.0, 0.25),
        1.0,
    );
    let mut moon_jumper = PlanetaryAvatar::with_attire_and_bindings(
        moon_start,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        YouthAttire::squire(),
        ActionBindings::seeded(),
    );
    let mut moon_peak = 0.0f32;
    for i in 0..180 {
        moon_jumper.step_motion(dt, if i == 0 { &jump } else { &idle }, &host_world);
        moon_peak = moon_peak.max(avatar_pos(&moon_jumper).y - moon_start.y);
    }
    assert!(
        moon_peak > earth_peak * 2.5,
        "Lunar 0.22g leap ({moon_peak:.2}m) must reach >2.5x Earth jump ({earth_peak:.2}m)"
    );

    let dig_eye = moon_start + Vec3::new(0.0, 0.6, 0.0);
    let target_spot = moon_start + Vec3::new(0.0, 2.0, -0.5);
    let before = guest_world.constrain_to_surface(target_spot, 0.0).y;
    assert!(host_world.harvest_surface(dig_eye, Vec3::new(0.0, -0.9, -0.3).normalize()).is_some());
    for _ in 0..8 {
        let _ = host_world.step_surface(0.03);
        let _ = guest_world.step_surface(0.03);
    }
    let after = guest_world.constrain_to_surface(target_spot, 0.0).y;
    assert!(after < before - 0.1, "Lunar excavation must replicate from Host to Guest ({before} -> {after})");
}

#[test]
fn test_mouse_look_rotates_gaze_freely_without_steering_rocket() {
    let run_launch = |look_yaw_deg: f32, look_pitch_deg: f32| -> (Vec3, fauna::SightCone) {
        let mut world = CelestialSystem::barren_authoritative(32, 1.0);
        let _ = build_crane_rocket(world.as_mut());
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
        pilot.rotate_degrees(look_yaw_deg, look_pitch_deg);

        let mut burn = KeyMotion::idle();
        burn.register_key(KeyCode::Space, true);
        for _ in 0..240 {
            pilot.step_motion(1.0 / 120.0, &burn, world.as_ref());
        }
        (avatar_pos(&pilot), pilot.sight_cone())
    };

    let (pos_straight, cone_straight) = run_launch(0.0, 0.0);
    let (pos_freelook, cone_freelook) = run_launch(135.0, 30.0);

    assert!(
        (pos_straight - pos_freelook).length() < 1e-5,
        "Mouse look must never steer or tilt rocket trajectory ({pos_straight:?} vs {pos_freelook:?})"
    );
    let probe_spot = pos_straight + Vec3::new(-18.0, 0.0, 0.0);
    assert_ne!(
        cone_straight.reaches(probe_spot, 40.0),
        cone_freelook.reaches(probe_spot, 40.0),
        "Pilot mouse input must still rotate cockpit free-look camera"
    );
}

#[test]
fn test_clicking_rocket_controls_prefers_nose_camera_over_lower_hull_camera() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let g0 = build_crane_rocket(world.as_mut());
    let av = Vec3::new(0.25, g0, 1.75);
    assert!(world.place_molded_surface_block(Vec3::new(1.25, g0 + 1.75, 1.75), Vec3::NEG_Z, av, BlockMold::cube().in_medium(BlockMedium::RocketCamera)));
    assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, av, BlockMold::cube().in_medium(BlockMedium::PlanedPlanks)));
    for top_y in [2.90, 3.40] {
        assert!(world.place_molded_surface_block(Vec3::new(0.75, g0 + top_y, 0.75), Vec3::NEG_Y, av, BlockMold::cube().in_medium(BlockMedium::PlanedPlanks)));
    }
    assert!(world.place_molded_surface_block(Vec3::new(0.75, g0 + 3.90, 0.75), Vec3::NEG_Y, av, BlockMold::cube().in_medium(BlockMedium::RocketCamera)));
    let spawn = world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0);
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(spawn, Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))), YouthAttire::squire(), ActionBindings::seeded());
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
    assert!(pilot.use_right_hand(world.as_mut()));
    let inv = pilot.view_projection_matrix(1.0).inverse();
    let eye = inv.project_point3(Vec3::ZERO);
    let dir = (inv.project_point3(Vec3::new(0.0, 0.0, 0.5)) - eye).normalize();
    assert!(eye.y > g0 + 2.4 && dir.y > 0.99, "Nose camera at rocket tip must be selected over lower camera, got ({eye:?}, {dir:?})");
}
