use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{
    BlockMedium, BlockMold, CelestialSystem, CraftBlueprint, CraftPose, MeshTarget, PlanetaryWorld,
    Quad, VesselSink,
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

fn reticle_probe(avatar: &PlanetaryAvatar, aspect: f32) -> (glam::Vec2, Vec<Vec3>) {
    struct HudProbe(Vec<Vec3>);
    impl MeshTarget for HudProbe {
        fn push_quad(&mut self, quad: Quad) {
            self.0.push(quad.transfer(|v0, _, v2, _, _, _| (v0 + v2) * 0.5));
        }
    }
    let mut probe = HudProbe(Vec::new());
    avatar.emit_reticle_for_aspect(aspect, &mut probe);
    let c = *probe.0.first().expect("Reticle must emit crosshair quads");
    (glam::Vec2::new(c.x, c.y), probe.0)
}

#[test]
fn test_rocket_camera_moves_within_small_radius_on_nose_and_belly_channels_without_steering_rocket() {
    let aspect = 16.0 / 9.0;
    let run_flight = |yaw_deg: f32, pitch_deg: f32, belly_channel: bool| -> (Vec3, (Vec3, Vec3), (glam::Vec2, Vec<Vec3>), glam::Mat4, Vec<u8>) {
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
        (
            avatar_pos(&pilot),
            camera_ray(&pilot),
            reticle_probe(&pilot, aspect),
            pilot.view_projection_matrix(aspect),
            recorder.0,
        )
    };

    let (pos_nose_fixed, (eye_nose, dir_nose_fixed), (ret_nose_fixed, hud_nose), _, wire_nose_fixed) = run_flight(0.0, 0.0, false);
    let (pos_nose_small, (_, dir_nose_small), (ret_nose_small, _), vp_nose_small, _) = run_flight(9.0, 6.0, false);
    let (pos_nose_wide, (_, dir_nose_wide), (ret_nose_wide, _), vp_nose_wide, wire_nose_wide) = run_flight(95.0, 45.0, false);
    assert!(
        (pos_nose_fixed - pos_nose_wide).length() < 1e-5
            && (pos_nose_fixed - pos_nose_small).length() < 1e-5
            && wire_nose_fixed == wire_nose_wide,
        "Mouse look in NoseCone camera must never steer or rotate the rocket ({pos_nose_fixed:?} vs {pos_nose_wide:?})"
    );
    assert!(
        dir_nose_fixed.dot(dir_nose_small) < 0.990
            && dir_nose_fixed.dot(dir_nose_wide) < 0.880
            && dir_nose_fixed.dot(dir_nose_wide) >= 0.864,
        "Mouse movement must move NoseCone camera only within a small circular radius ({dir_nose_fixed:?} -> {dir_nose_wide:?})"
    );
    let nose_target = eye_nose + dir_nose_fixed * 80.0;
    let proj_small = vp_nose_small.project_point3(nose_target).truncate();
    let proj_wide = vp_nose_wide.project_point3(nose_target).truncate();
    assert!(
        ret_nose_fixed.length() < 1e-5
            && ret_nose_small.length() > 0.08
            && (ret_nose_small - proj_small).length() < 5e-4
            && (ret_nose_wide - proj_wide).length() < 5e-4,
        "Crosshair must stay aimed at the 3D NoseCone target ({ret_nose_wide:?} vs {proj_wide:?})"
    );

    let (pos_belly_fixed, (eye_belly, dir_belly_fixed), (_, hud_belly), _, wire_belly_fixed) = run_flight(0.0, 0.0, true);
    let (pos_belly_wide, (_, dir_belly_wide), (ret_belly_wide, _), vp_belly_wide, wire_belly_wide) = run_flight(-110.0, -40.0, true);
    assert!(
        hud_nose.len() > 70 && hud_belly.len() > 70 && hud_nose != hud_belly,
        "Preview HUD must highlight active window frame and render distinct CAM 1 - NOSE vs CAM 2 - BELLY captions"
    );
    assert!(
        (pos_belly_fixed - pos_belly_wide).length() < 1e-5 && wire_belly_fixed == wire_belly_wide,
        "Mouse look in BellyPod camera must never steer or rotate the rocket"
    );
    assert!(
        dir_belly_fixed.dot(dir_belly_wide) < 0.880 && dir_belly_fixed.dot(dir_belly_wide) >= 0.864,
        "Mouse movement must move BellyPod landing camera only within a small circular radius ({dir_belly_fixed:?} -> {dir_belly_wide:?})"
    );
    let belly_target = eye_belly + dir_belly_fixed * 40.0;
    let proj_belly = vp_belly_wide.project_point3(belly_target).truncate();
    assert!(
        ret_belly_wide.length() > 0.25 && (ret_belly_wide - proj_belly).length() < 5e-4,
        "Crosshair must stay aimed at the 3D BellyPod touchdown target ({ret_belly_wide:?} vs {proj_belly:?})"
    );
}

#[test]
fn test_side_rocket_camera_stays_in_small_radius_while_cockpit_and_handheld_orbit_freely() {
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
        struct BustVerts(Vec<Vec3>);
        impl MeshTarget for BustVerts {
            fn push_quad(&mut self, q: Quad) { q.transfer(|v0, _, _, _, _, _| self.0.push(v0)); }
        }
        let mut previews = Vec::new();
        let (mut bust_a, mut bust_b) = (BustVerts(Vec::new()), BustVerts(Vec::new()));
        assert!(pilot.emit_intercom_bust(1.62, &mut bust_a).is_none() && previews.is_empty());
        assert_eq!(reticle_probe(&pilot, 16.0 / 9.0).1.len(), 10, "Pedestrian view must omit channel strip");
        pilot.rotate_degrees(-90.0, 10.0);
        assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
        assert!(pilot.use_right_hand(world.as_mut()));
        assert_eq!(pilot.collect_camera_previews(1.62, &mut previews), 1);
        assert_eq!(previews.len(), 1, "Only hull cameras are rendered into the coarse preview atlas");
        assert!(pilot.emit_intercom_bust(1.62, &mut bust_a).is_some() && bust_a.0.len() > 80);
        pilot.rotate_degrees(yaw_deg, pitch_deg);
        let _ = pilot.emit_intercom_bust(1.62, &mut bust_b);
        if yaw_deg.abs() > 1.0 {
            assert_ne!(bust_a.0, bust_b.0, "3D Intercom avatar head must turn with live mouse movement");
        }
        let side_ray = camera_ray(&pilot);
        let side_hud = reticle_probe(&pilot, 16.0 / 9.0).1;
        assert!(side_hud.len() > 70, "SideFairing must display intercom frame and active status badge");

        assert!(pilot.dispatch_trigger(InputTrigger::WheelUp));
        previews.clear();
        assert_eq!(pilot.collect_camera_previews(1.62, &mut previews), 0);
        let cockpit_hud = reticle_probe(&pilot, 16.0 / 9.0).1;
        assert!(cockpit_hud.len() >= 25 && cockpit_hud != side_hud);
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
        side_dir_a.dot(side_dir_b) < 0.880 && side_dir_a.dot(side_dir_b) >= 0.864,
        "Mouse movement must swivel SideFairing rocket camera only within a small radius ({side_dir_a:?} -> {side_dir_b:?})"
    );
    assert!(
        (shot_eye_a - shot_eye_b).length() > 0.8 && shot_dir_a.dot(shot_dir_b) < 0.5,
        "Mouse movement in cockpit channel must still orbit handheld FieldCamera view around the flying rocket"
    );
}

#[test]
fn test_camera_attached_under_crane_rocket_stays_above_world_cube_surface() {
    for cam_mold in [
        BlockMold::cube().in_medium(BlockMedium::RocketCamera),
        BlockMold::cube().next_form().in_medium(BlockMedium::RocketCamera),
    ] {
        let mut world = CelestialSystem::barren_authoritative(32, 1.0);
        let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
        let av = Vec3::new(0.25, g0, 1.75);
        let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
        for med in [BlockMedium::CantileverRig, BlockMedium::RocketNozzle] {
            assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, av, BlockMold::cube().in_medium(med)));
        }
        assert!(world.place_molded_surface_block(Vec3::new(1.25, g0 + 2.90, 0.25), Vec3::NEG_Y, av, BlockMold::cube().turn_clockwise().in_medium(BlockMedium::FlightConsole)));
        assert!(world.place_molded_surface_block(Vec3::new(1.25, g0 + 1.75, 0.75), Vec3::NEG_Z, av, BlockMold::cube().in_medium(BlockMedium::CabinSeat)));
        assert!(world.place_molded_surface_block(Vec3::new(1.25, g0 + 1.75, 1.25), Vec3::NEG_Z, av, BlockMold::cube().in_medium(BlockMedium::PlanedPlanks)));
        assert!(world.plan_molded_block(Vec3::new(1.25, g0 + 0.90, 1.25), Vec3::Y, Vec3::new(1.25, g0, 2.25), cam_mold).is_some());
        assert!(world.place_molded_surface_block(Vec3::new(1.25, g0 + 0.90, 1.25), Vec3::Y, Vec3::new(1.25, g0, 2.25), cam_mold));
        assert!(world.place_molded_surface_block(Vec3::new(1.25, g0 + 0.90, 0.75), Vec3::Y, Vec3::new(1.25, g0, 2.25), cam_mold));
        assert!(world.place_molded_surface_block(Vec3::new(1.25, g0 + 0.40, 1.25), Vec3::NEG_Y, Vec3::new(1.25, g0, 2.25), BlockMold::cube().in_medium(BlockMedium::QuarriedStone)));

        let spawn = world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0);
        let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
            spawn,
            Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
            YouthAttire::squire(),
            ActionBindings::seeded(),
        );
        pilot.rotate_degrees(-90.0, 10.0);
        assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
        pilot.rotate_degrees(0.0, -20.0);
        assert!(pilot.use_right_hand(world.as_mut()));

        let (pad_eye, pad_dir) = camera_ray(&pilot);
        let ground_under_cam = world.constrain_to_surface(Vec3::new(pad_eye.x, 22.0, pad_eye.z), 0.0).y;
        assert!(
            pad_eye.y >= ground_under_cam + 0.15 && pad_dir.y < -0.99,
            "Under-rocket camera must look down from ABOVE World-Cube surface (eye={pad_eye:?}, ground={ground_under_cam:.2}, dir={pad_dir:?})"
        );
        let idle = KeyMotion::idle();
        for _ in 0..30 {
            pilot.step_motion(1.0 / 120.0, &idle, world.as_ref());
            let (eye, dir) = camera_ray(&pilot);
            let ground_y = world.constrain_to_surface(Vec3::new(eye.x, 22.0, eye.z), 0.0).y;
            assert!(eye.y >= ground_y + 0.15 && dir.y < -0.99, "Ticked pad camera dipped below ground: {eye:?} < {ground_y}");
        }
    }
}
