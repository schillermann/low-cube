use glam::Vec3;
use sanctuary::{HostedPlanet, JoinTarget, RemoteLink, RemotePlanet, SanctuaryHost};
use winit::keyboard::KeyCode;
use world::{BlockMedium, BlockMold, CelestialSystem, GravityField as _, MeshTarget, PlanetaryWorld, Quad};
use crate::attire::{EvaAttire, PhotoShutter};
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarOptics as _, AvatarPresence as _, HandGesture,
    PhotoPort as _, PlanetaryAvatar,
};
use crate::black_box::PresenceProbe;
use crate::controls::{ActionBindings, InputTrigger, KeyMotion, KeyboardControls};
use crate::flight::SurfaceFlight;
use crate::zero_g_flight::ZeroGFlight;
use super::space_step::MAX_COSMIC_SPEED;

fn pos_of(avatar: &PlanetaryAvatar) -> Vec3 {
    let mut probe = PresenceProbe::at_origin();
    avatar.report_presence(&mut probe);
    probe.position()
}

fn build_expanse_ship(world: &mut dyn PlanetaryWorld) -> f32 {
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let av_pos = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    for mold in [
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
        BlockMold::cube().in_medium(BlockMedium::RocketNozzle),
    ] {
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, av_pos, mold));
    }
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
        BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 1.75),
        Vec3::NEG_Z,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::RocketCamera),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.75, 0.75),
        Vec3::X,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::PlanedPlanks),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(0.75, g0 + 0.90, 0.75),
        Vec3::Y,
        av_pos,
        BlockMold::cube().in_medium(BlockMedium::RocketNozzle),
    ));
    g0
}

#[test]
fn test_rocket_speed_converges_to_cosmic_cap_and_never_grows_unbounded() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let _ = build_expanse_ship(world.as_mut());
    let spawn = world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0);
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
        spawn,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        EvaAttire::astronaut(),
        ActionBindings::seeded(),
    );
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
    assert!(pilot.use_right_hand(world.as_mut()));

    let mut burn = KeyMotion::idle();
    burn.register_key(KeyCode::Space, true);
    let dt = 1.0 / 120.0;
    for _ in 0..1800 {
        pilot.step_motion(dt, &burn, world.as_ref());
    }
    let p_15s = pos_of(&pilot);
    for _ in 0..120 {
        pilot.step_motion(dt, &burn, world.as_ref());
    }
    let speed_16s = (pos_of(&pilot) - p_15s).length();
    for _ in 0..3600 {
        pilot.step_motion(dt, &burn, world.as_ref());
    }
    let p_46s = pos_of(&pilot);
    for _ in 0..120 {
        pilot.step_motion(dt, &burn, world.as_ref());
    }
    let speed_47s = (pos_of(&pilot) - p_46s).length();
    assert!(
        speed_16s <= MAX_COSMIC_SPEED + 0.05 && speed_47s <= MAX_COSMIC_SPEED + 0.05,
        "Rocket speed must never exceed MAX_COSMIC_SPEED ({MAX_COSMIC_SPEED}), got {speed_16s:.2} and {speed_47s:.2}"
    );
    assert!((speed_47s - speed_16s).abs() < 0.25, "Sustained thrust must plateau at terminal cosmic speed");
}

#[test]
fn test_outboard_cameras_orrery_and_flip_and_burn_maneuver() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let _ = build_expanse_ship(world.as_mut());
    let spawn = world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0);
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
        spawn,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        EvaAttire::astronaut(),
        ActionBindings::seeded(),
    );
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
    assert!(pilot.use_right_hand(world.as_mut()));

    struct GlowCounter(usize);
    impl MeshTarget for GlowCounter {
        fn push_quad(&mut self, quad: Quad) {
            if quad.radiate(|g| g > 0.84) {
                self.0 += 1;
            }
        }
    }
    let mut holo = GlowCounter(0);
    pilot.emit_appearance(&mut holo);
    assert!(holo.0 > 60, "FlightConsole must project glowing 3D holographic Orrery wireframe, got {}", holo.0);

    let mut burn = KeyMotion::idle();
    burn.register_key(KeyCode::Space, true);
    let idle = KeyMotion::idle();
    let dt = 1.0 / 120.0;
    for _ in 0..480 {
        pilot.step_motion(dt, &burn, world.as_ref());
    }
    let ship_pos = pos_of(&pilot);
    assert!(pilot.sight_cone().reaches(ship_pos + Vec3::Y * 25.0, 40.0), "Clicking FlightConsole must immediately select Zenith nose camera");
    assert!(pilot.dispatch_trigger(InputTrigger::WheelClick), "WheelClick on hull camera must snap Polaroid");
    assert!(matches!(pilot.take_photo_shutter(), PhotoShutter::Triggered(_)));

    assert!(pilot.dispatch_trigger(InputTrigger::WheelDown), "WheelDown must select Nadir landing camera");
    assert!(pilot.sight_cone().reaches(ship_pos - Vec3::Y * 25.0, 40.0), "Nadir camera must look down below nozzles");

    assert!(pilot.dispatch_trigger(InputTrigger::Key(KeyCode::KeyR)), "KeyR must engage 180-degree Flip & Burn");
    for _ in 0..180 {
        pilot.step_motion(dt, &idle, world.as_ref());
    }
    let before_brake = pos_of(&pilot);
    pilot.step_motion(dt, &idle, world.as_ref());
    let coast_step = (pos_of(&pilot) - before_brake).y;
    for _ in 0..180 {
        pilot.step_motion(dt, &burn, world.as_ref());
    }
    let after_brake = pos_of(&pilot);
    pilot.step_motion(dt, &idle, world.as_ref());
    let braked_step = (pos_of(&pilot) - after_brake).y;
    assert!(braked_step < coast_step * 0.35, "Retrograde main burn after Flip must decelerate ascent ({coast_step} -> {braked_step})");
}

#[test]
fn test_coop_thrust_gravity_grounds_crew_under_burn_and_floats_when_cut() {
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
        let _ = guest_world.step_surface(0.02);
        let _ = host_world.step_surface(0.02);
    }

    let _ = build_expanse_ship(&mut host_world);
    for _ in 0..8 {
        let _ = host_world.step_surface(0.02);
        let _ = guest_world.step_surface(0.02);
    }

    let host_spawn = host_world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0);
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
        host_spawn,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        EvaAttire::astronaut(),
        ActionBindings::seeded(),
    );
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, &mut host_world));
    assert!(pilot.use_right_hand(&mut host_world));

    let dt = 1.0 / 120.0;
    let mut burn = KeyMotion::idle();
    burn.register_key(KeyCode::Space, true);
    let idle = KeyMotion::idle();
    for _ in 0..480 {
        pilot.step_motion(dt, &burn, &host_world);
    }
    pilot.sync_world_presence(&mut host_world);
    for _ in 0..6 {
        let _ = host_world.step_surface(0.01);
        let _ = guest_world.step_surface(0.01);
    }

    let deck_spot = pos_of(&pilot) + Vec3::new(-0.5, 0.9, 0.0);
    let mut crew = PlanetaryAvatar::with_attire_and_bindings(
        deck_spot,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        EvaAttire::astronaut(),
        ActionBindings::seeded(),
    );

    for _ in 0..90 {
        pilot.step_motion(dt, &burn, &host_world);
        pilot.sync_world_presence(&mut host_world);
        crew.sync_world_presence(&mut guest_world);
        let _ = host_world.step_surface(dt);
        let _ = guest_world.step_surface(dt);
        crew.step_motion(dt, &idle, &guest_world);
    }
    let settled_offset = pos_of(&crew).y - pos_of(&pilot).y;
    assert!(
        settled_offset.abs() < 1.2
            && pos_of(&crew).y > 115.0
            && guest_world.apply_pull(pos_of(&crew), 1.0).length() < 1e-4,
        "Crewmate must ride high-speed rocket in 0.0g space under Thrust Gravity, offset={settled_offset}, y={}",
        pos_of(&crew).y
    );

    for i in 0..60 {
        pilot.step_motion(dt, &idle, &host_world);
        pilot.sync_world_presence(&mut host_world);
        crew.sync_world_presence(&mut guest_world);
        let _ = host_world.step_surface(dt);
        let _ = guest_world.step_surface(dt);
        crew.step_motion(dt, if i < 6 { &burn } else { &idle }, &guest_world);
    }
    let float_offset = pos_of(&crew).y - pos_of(&pilot).y;
    assert!(
        float_offset > settled_offset + 0.25,
        "Cutting main thrust ('On the Float') must let crewmate float weightlessly off the deck ({settled_offset} -> {float_offset})"
    );
}

#[test]
fn test_camera_less_rocket_keeps_cockpit_gaze_without_fallback() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let av = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    for mold in [
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
        BlockMold::cube().in_medium(BlockMedium::RocketNozzle),
    ] {
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, av, mold));
    }
    assert!(world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, -1, false));
    assert!(world.place_molded_surface_block(Vec3::new(1.25, g0 + 2.40, 0.25), Vec3::NEG_Y, av, BlockMold::cube().turn_clockwise().in_medium(BlockMedium::FlightConsole)));
    assert!(world.place_molded_surface_block(Vec3::new(1.25, g0 + 1.75, 0.75), Vec3::NEG_Z, av, BlockMold::cube().in_medium(BlockMedium::CabinSeat)));
    assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, av, BlockMold::cube().in_medium(BlockMedium::PlanedPlanks)));
    for top_y in [2.90, 3.40, 3.90] {
        assert!(world.place_molded_surface_block(Vec3::new(0.75, g0 + top_y, 0.75), Vec3::NEG_Y, av, BlockMold::cube().in_medium(BlockMedium::PlanedPlanks)));
    }
    let spawn = world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0);
    let cam_ray = |a: &PlanetaryAvatar| {
        let inv = a.view_projection_matrix(1.0).inverse();
        let eye = inv.project_point3(Vec3::ZERO);
        (eye, (inv.project_point3(Vec3::new(0.0, 0.0, 0.5)) - eye).normalize())
    };
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(spawn, Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))), EvaAttire::astronaut(), ActionBindings::seeded());
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
    assert!(pilot.use_right_hand(world.as_mut()));
    let (cockpit_eye, cockpit_dir) = cam_ray(&pilot);
    assert!(cockpit_eye.y < g0 + 2.0 && cockpit_dir.y.abs() < 0.4, "Camera-less rocket must stay in cockpit view without fallback, got ({cockpit_eye:?}, {cockpit_dir:?})");
    let _ = pilot.dispatch_trigger(InputTrigger::WheelDown);
    assert_eq!(cam_ray(&pilot), (cockpit_eye, cockpit_dir), "Scrolling without installed camera must not switch to any synthetic hull view");
}
