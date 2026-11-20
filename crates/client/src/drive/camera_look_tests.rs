use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{
    BlockMedium, BlockMold, CelestialSystem, CraftBlueprint, CraftPose, PlanetaryWorld, VesselSink,
};
use crate::attire::{HeldRelic, SightMode, YouthAttire};
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarOptics as _, AvatarPresence as _, HandGesture,
    PlanetaryAvatar,
};
use crate::black_box::PresenceProbe;
use crate::controls::{ActionBindings, InputTrigger, KeyMotion, KeyboardControls};
use crate::flight::SurfaceFlight;
use crate::zero_g_flight::ZeroGFlight;

fn build_rocket(world: &mut dyn PlanetaryWorld, with_hull_camera: bool) -> f32 {
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
    assert!(world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, -1, false));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 2.40, 0.25),
        Vec3::NEG_Y,
        av,
        BlockMold::cube().turn_clockwise().in_medium(BlockMedium::FlightConsole),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 0.75),
        Vec3::NEG_Z,
        av,
        BlockMold::cube().in_medium(BlockMedium::CabinSeat),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 1.25),
        Vec3::NEG_Z,
        av,
        BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle),
    ));
    if with_hull_camera {
        assert!(world.place_molded_surface_block(
            Vec3::new(1.25, g0 + 1.75, 1.75),
            Vec3::NEG_Z,
            av,
            BlockMold::cube().in_medium(BlockMedium::RocketCamera),
        ));
    }
    g0
}

fn avatar_pos(avatar: &PlanetaryAvatar) -> Vec3 {
    let mut probe = PresenceProbe::at_origin();
    avatar.report_presence(&mut probe);
    probe.position()
}

fn camera_ray(avatar: &PlanetaryAvatar) -> (Vec3, Vec3) {
    let inv = avatar.view_projection_matrix(1.0).inverse();
    let eye = inv.project_point3(Vec3::ZERO);
    let dir = (inv.project_point3(Vec3::new(0.0, 0.0, 0.5)) - eye).normalize();
    (eye, dir)
}

#[test]
fn test_rocket_camera_allows_mouse_freelook_on_nose_and_belly_channels_without_steering_rocket() {
    let run_flight = |yaw_deg: f32, pitch_deg: f32, belly_channel: bool| -> (Vec3, (Vec3, Vec3), Vec<u8>) {
        let mut world = CelestialSystem::barren_authoritative(32, 1.0);
        build_rocket(world.as_mut(), true);
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
        if belly_channel {
            assert!(pilot.dispatch_trigger(InputTrigger::WheelDown));
        }
        pilot.rotate_degrees(yaw_deg, pitch_deg);

        let mut burn = KeyMotion::idle();
        burn.register_key(KeyCode::Space, true);
        burn.register_key(KeyCode::KeyW, true);
        for _ in 0..180 {
            pilot.step_motion(1.0 / 120.0, &burn, world.as_ref());
        }
        struct PoseRecorder(Vec<u8>);
        impl VesselSink for PoseRecorder {
            fn receive_vessel(&mut self, _blueprint: &CraftBlueprint, pose: CraftPose) {
                pose.inscribe(&mut self.0);
            }
        }
        let mut recorder = PoseRecorder(Vec::new());
        pilot.present_vessel(&mut recorder);
        (avatar_pos(&pilot), camera_ray(&pilot), recorder.0)
    };

    let (pos_nose_fixed, (_, dir_nose_fixed), wire_nose_fixed) = run_flight(0.0, 0.0, false);
    let (pos_nose_look, (_, dir_nose_look), wire_nose_look) = run_flight(95.0, 45.0, false);
    assert!(
        (pos_nose_fixed - pos_nose_look).length() < 1e-5 && wire_nose_fixed == wire_nose_look,
        "Mouse look in NoseCone camera must never steer or rotate the rocket ({pos_nose_fixed:?} vs {pos_nose_look:?})"
    );
    assert!(
        dir_nose_fixed.dot(dir_nose_look) < 0.75,
        "Mouse movement must pan and tilt the NoseCone rocket camera ({dir_nose_fixed:?} vs {dir_nose_look:?})"
    );

    let (pos_belly_fixed, (_, dir_belly_fixed), wire_belly_fixed) = run_flight(0.0, 0.0, true);
    let (pos_belly_look, (_, dir_belly_look), wire_belly_look) = run_flight(-110.0, -40.0, true);
    assert!(
        (pos_belly_fixed - pos_belly_look).length() < 1e-5 && wire_belly_fixed == wire_belly_look,
        "Mouse look in BellyPod camera must never steer or rotate the rocket"
    );
    assert!(
        dir_belly_fixed.dot(dir_belly_look) < 0.80,
        "Mouse movement must pan and tilt the BellyPod landing camera ({dir_belly_fixed:?} vs {dir_belly_look:?})"
    );
}

#[test]
fn test_side_rocket_camera_and_handheld_field_camera_follow_mouse_look_without_steering_rocket() {
    let run_flight = |yaw_deg: f32, pitch_deg: f32| -> (Vec3, (Vec3, Vec3), (Vec3, Vec3)) {
        let mut world = CelestialSystem::barren_authoritative(32, 1.0);
        let g0 = build_rocket(world.as_mut(), false);
        assert!(world.place_molded_surface_block(
            Vec3::new(1.25, g0 + 1.75, -0.25),
            Vec3::Z,
            Vec3::new(1.25, g0, -1.25),
            BlockMold::cube().in_medium(BlockMedium::RocketCamera).next_form().next_form(),
        ));
        let spawn = world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0);
        let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
            spawn,
            Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
            YouthAttire::squire().with_relics(HeldRelic::EmptyPalm, HeldRelic::FieldCamera),
            ActionBindings::seeded(),
        );
        pilot.rotate_degrees(-90.0, 10.0);
        assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
        assert!(pilot.use_right_hand(world.as_mut()));
        pilot.rotate_degrees(yaw_deg, pitch_deg);
        let side_ray = camera_ray(&pilot);

        assert!(pilot.dispatch_trigger(InputTrigger::WheelUp));
        pilot.rotate_degrees(yaw_deg, pitch_deg);
        let inv = pilot.view_projection_for_sight(1.0, SightMode::AngledSight).inverse();
        let shot_eye = inv.project_point3(Vec3::ZERO);
        let shot_dir = (inv.project_point3(Vec3::new(0.0, 0.0, 0.5)) - shot_eye).normalize();

        let mut burn = KeyMotion::idle();
        burn.register_key(KeyCode::Space, true);
        for _ in 0..180 {
            pilot.step_motion(1.0 / 120.0, &burn, world.as_ref());
        }
        (avatar_pos(&pilot), side_ray, (shot_eye, shot_dir))
    };

    let (pos_a, (_, side_dir_a), (shot_eye_a, shot_dir_a)) = run_flight(0.0, 0.0);
    let (pos_b, (_, side_dir_b), (shot_eye_b, shot_dir_b)) = run_flight(90.0, -25.0);
    assert!(
        (pos_a - pos_b).length() < 1e-5,
        "Mouse look across SideFairing and handheld FieldCamera must never steer the rocket ({pos_a:?} vs {pos_b:?})"
    );
    assert!(
        side_dir_a.dot(side_dir_b) < 0.5,
        "Mouse movement must swivel SideFairing rocket camera ({side_dir_a:?} vs {side_dir_b:?})"
    );
    assert!(
        (shot_eye_a - shot_eye_b).length() > 0.8 && shot_dir_a.dot(shot_dir_b) < 0.5,
        "Mouse movement must orbit handheld FieldCamera view around the flying rocket ({shot_dir_a:?} vs {shot_dir_b:?})"
    );
}
