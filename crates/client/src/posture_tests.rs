use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{GravityField, MeshTarget, PlanetaryWorld};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::horizon::CameraHorizon;
use crate::controls::{KeyboardControls, KeyMotion};
use crate::gait::{LocomotionGait, SneakGait, WalkGait};
use crate::orientation::MouseLook;
use crate::posture::AvatarPosture;

struct FlatGroundWorld;

impl GravityField for FlatGroundWorld {
    fn pull_direction_at(&self, _position: Vec3) -> Vec3 { Vec3::NEG_Y }
    fn surface_up_at(&self, _position: Vec3) -> Vec3 { Vec3::Y }
}

impl PlanetaryWorld for FlatGroundWorld {
    fn emit_surface(&self, _target: &mut dyn MeshTarget) {}
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 { Vec3::new(position.x, player_height, position.z) }
}

#[test]
fn test_posture_initial_standing() {
    assert!(!AvatarPosture::standing().crouched() && AvatarPosture::crouching().crouched());
}

#[test]
fn test_posture_transitions_to_crouch_and_back() {
    let initial = AvatarPosture::standing();
    let sneak_gait = SneakGait::crouching();
    let walk_gait = WalkGait::upright();

    let mut posture = initial;
    for _ in 0..30 {
        posture = sneak_gait.shape_posture(posture, 1.0 / 60.0);
    }
    assert!(posture.crouched());

    for _ in 0..30 {
        posture = walk_gait.shape_posture(posture, 1.0 / 60.0);
    }
    assert!(!posture.crouched());
}

#[test]
fn test_posture_computes_distinct_view_matrices() {
    let look = MouseLook::idle();
    let horizon = CameraHorizon::aligned(glam::Quat::IDENTITY, &look);
    let position = Vec3::new(0.0, 1.0, 0.0);
    let standing_view = AvatarPosture::standing().view_matrix(&horizon, position);
    let crouching_view = AvatarPosture::crouching().view_matrix(&horizon, position);
    assert_ne!(standing_view, crouching_view);
}

#[test]
fn test_avatar_lowers_viewpoint_when_sneaking() {
    let world = FlatGroundWorld;
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 1.0, 0.0));
    let mut controls = KeyMotion::idle();
    let standing_matrix = avatar.view_projection_matrix(16.0 / 9.0);

    controls.register_key(KeyCode::ShiftLeft, true);
    for _ in 0..20 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let crouching_matrix = avatar.view_projection_matrix(16.0 / 9.0);
    assert_ne!(standing_matrix, crouching_matrix);

    controls.register_key(KeyCode::ShiftLeft, false);
    for _ in 0..30 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let restored_matrix = avatar.view_projection_matrix(16.0 / 9.0);
    let diff = (standing_matrix - restored_matrix).abs();
    for col in 0..4 {
        for row in 0..4 {
            assert!(diff.col(col)[row] < 1e-3);
        }
    }
}

#[test]
fn test_avatar_expands_fov_when_sprinting() {
    let world = FlatGroundWorld;
    let mut avatar = PlanetaryAvatar::at(Vec3::new(0.0, 1.0, 0.0));
    let mut controls = KeyMotion::idle();

    let normal_matrix = avatar.view_projection_matrix(16.0 / 9.0);

    // Double-tap W to initiate sprint
    controls.register_key(KeyCode::KeyW, true);
    controls.register_key(KeyCode::KeyW, false);
    controls.register_key(KeyCode::KeyW, true);

    for _ in 0..30 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let sprint_matrix = avatar.view_projection_matrix(16.0 / 9.0);

    // Projection matrix changed due to expanded sprint FOV
    assert_ne!(normal_matrix, sprint_matrix);
    assert!((normal_matrix.col(0)[0] - sprint_matrix.col(0)[0]).abs() > 0.05);

    // Releasing W restores normal walking FOV
    controls.register_key(KeyCode::KeyW, false);
    for _ in 0..40 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let restored_matrix = avatar.view_projection_matrix(16.0 / 9.0);
    assert!((normal_matrix.col(0)[0] - restored_matrix.col(0)[0]).abs() < 1e-3);
}

#[test]
fn test_player_avatar_leap_posture_crouches_extends_and_lands() {
    let (dt, rhythm, mut posture) = (1.0 / 60.0, crate::attire::StrideRhythm::idle(), AvatarPosture::standing());
    let stand_pose = posture.body_pose(Vec3::ZERO, glam::Quat::IDENTITY, glam::Quat::IDENTITY, rhythm);
    for _ in 0..4 { posture = posture.advance_leap(Vec3::ZERO, Vec3::new(0.0, 3.0 * dt, 0.0), Vec3::Y, dt); }
    assert!(posture.body_pose(Vec3::ZERO, glam::Quat::IDENTITY, glam::Quat::IDENTITY, rhythm).crouch_compression() > 0.25);
    for i in 0..24 { posture = posture.advance_leap(Vec3::ZERO, Vec3::new(0.0, (1.8 - (i as f32) * 0.15) * dt, 0.0), Vec3::Y, dt); }
    let air_pose = posture.body_pose(Vec3::ZERO, glam::Quat::IDENTITY, glam::Quat::IDENTITY, rhythm);
    assert!(air_pose.leg_joints(1.0).1.z < stand_pose.leg_joints(1.0).1.z - 0.08 && air_pose.eye_origin() == stand_pose.eye_origin());
    for _ in 0..6 { posture = posture.advance_leap(Vec3::ZERO, Vec3::ZERO, Vec3::Y, dt); }
    assert!(posture.body_pose(Vec3::ZERO, glam::Quat::IDENTITY, glam::Quat::IDENTITY, rhythm).crouch_compression() > 0.25);
    for _ in 0..50 { posture = posture.advance_leap(Vec3::ZERO, Vec3::ZERO, Vec3::Y, dt); }
    assert!(posture.body_pose(Vec3::ZERO, glam::Quat::IDENTITY, glam::Quat::IDENTITY, rhythm).crouch_compression() < 0.01);
}

#[test]
fn test_doorframe_passage_respects_character_height_width_and_depth() {
    use crate::attire::{BodyRegion, YouthAttire};
    use world::{CuboidShape, ReliefElevation, TerracedRelief};
    struct FlatRelief;
    impl ReliefElevation for FlatRelief { fn elevation_at(&self, _: i32, _: i32) -> i32 { 0 } fn surface_height_at(&self, _: f32, _: f32) -> f32 { 0.0 } }
    struct DoorframeWorld(TerracedRelief);
    impl GravityField for DoorframeWorld { fn pull_direction_at(&self, _: Vec3) -> Vec3 { Vec3::NEG_Y } fn surface_up_at(&self, _: Vec3) -> Vec3 { Vec3::Y } }
    impl PlanetaryWorld for DoorframeWorld {
        fn emit_surface(&self, _: &mut dyn MeshTarget) {}
        fn constrain_to_surface(&self, p: Vec3, d: f32) -> Vec3 { Vec3::new(p.x, self.0.support_height_at(p.x, p.z, p.y - d, d) + d, p.z) }
        fn confine_beneath_crust(&self, p: Vec3) -> Vec3 { let (u, v, z) = self.0.confine(p.x, p.z, p.y); Vec3::new(u, z, v) }
    }
    let build_world = |wide_two: bool, lintel_base: f32| -> DoorframeWorld {
        let mut t = TerracedRelief::in_span(Box::new(FlatRelief), 16);
        let right_post = if wide_two { 2 } else { 1 };
        for h in 0..5 {
            let y = (h as f32) * 0.5;
            t.stack_molded_cell(-1, 0, y, Box::new(CuboidShape::new(0.5)));
            t.stack_molded_cell(right_post, 0, y, Box::new(CuboidShape::new(0.5)));
        }
        t.stack_molded_cell(0, 0, lintel_base, Box::new(CuboidShape::new(0.5)));
        if wide_two { t.stack_molded_cell(1, 0, lintel_base, Box::new(CuboidShape::new(0.5))); }
        DoorframeWorld(t)
    };
    let walk_through = |world: &DoorframeWorld, attire: YouthAttire, x: f32, sneak: bool, sideways: bool| -> f32 {
        let mut av = PlanetaryAvatar::with_attire(Vec3::new(x, 1.0, 0.90), Box::new(crate::flight::SurfaceFlight::terrestrial(1.0)), attire);
        if sideways { av.rotate_view(std::f32::consts::FRAC_PI_2 / 0.003, 0.0); }
        let mut ctrl = KeyMotion::idle();
        if sneak { ctrl.register_key(KeyCode::ShiftLeft, true); }
        for _ in 0..20 { av.step_motion(1.0 / 60.0, &ctrl, world); }
        ctrl.register_key(if sideways { KeyCode::KeyA } else { KeyCode::KeyW }, true);
        for _ in 0..45 { av.step_motion(1.0 / 60.0, &ctrl, world); }
        av.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO).z
    };
    let (norm, short, tall) = (
        YouthAttire::squire(),
        YouthAttire::squire().with_adjusted_region(BodyRegion::Crown, -4, 0).with_adjusted_region(BodyRegion::Torso, -4, 0).with_adjusted_region(BodyRegion::Legs, -4, 0),
        YouthAttire::squire().with_adjusted_region(BodyRegion::Crown, 4, 0).with_adjusted_region(BodyRegion::Torso, 4, 0).with_adjusted_region(BodyRegion::Legs, 4, 0),
    );
    let (slender, deep) = (
        YouthAttire::squire().with_adjusted_region(BodyRegion::Torso, -4, -4).with_adjusted_region(BodyRegion::Arms, -4, -4),
        YouthAttire::squire().with_adjusted_region(BodyRegion::Torso, 4, 4).with_adjusted_region(BodyRegion::Crown, 4, 4),
    );
    let (door_2x3, door_2x4, door_1x4) = (build_world(true, 1.5), build_world(true, 2.0), build_world(false, 2.0));
    assert!(walk_through(&door_2x3, norm, 0.50, false, false) > 0.45, "Default height (1.80m) must be blocked upright by 3-block (1.5m) lintel");
    assert!(walk_through(&door_2x3, short, 0.50, false, false) < -0.20, "Short character (1.30m) must walk upright under 3-block (1.5m) lintel");
    assert!(walk_through(&door_2x3, norm, 0.50, true, false) < -0.10, "Default character must pass under 3-block (1.5m) lintel when crouching");
    assert!(walk_through(&door_2x4, norm, 0.50, false, false) < -0.20, "Default character (1.80m) must walk upright under 4-block (2.0m) lintel");
    assert!(walk_through(&door_2x4, tall, 0.50, false, false) > 0.45, "Tall character (2.38m) must be blocked upright by 4-block (2.0m) lintel");
    assert!(walk_through(&door_1x4, norm, 0.25, false, false) > 0.45, "Default width (0.65m > 0.50m) must be blocked walking forward into 1-block-wide doorframe");
    assert!(walk_through(&door_1x4, slender, 0.25, false, false) < -0.20, "Slender width (< 0.47m) must walk forward through 1-block-wide doorframe");
    assert!(walk_through(&door_1x4, norm, 0.25, false, true) < -0.20, "Default depth (0.39m < 0.50m) must sidestep sideways through 1-block-wide doorframe");
    assert!(walk_through(&door_1x4, deep, 0.25, false, true) > 0.45, "Deep torso (0.54m > 0.50m) must be blocked even sidestepping into 1-block-wide doorframe");

    let mut tunnel = TerracedRelief::in_span(Box::new(FlatRelief), 16);
    for (u, v) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        for step in 0..6 { tunnel.excavate_layer(u, v, -((step as f32) * 0.5) - 0.25); }
    }
    for (u, v) in [(0, -1), (1, -1), (0, -2), (1, -2)] {
        for layer in [-2.75, -2.25, -1.75] { tunnel.excavate_layer(u, v, layer); }
    }
    let tunnel_world = DoorframeWorld(tunnel);
    let valkyrie = crate::menu::AvatarArchetype::Knight.female_attire();
    let mut valk = PlanetaryAvatar::with_attire(Vec3::new(0.50, -2.0, 0.25), Box::new(crate::flight::SurfaceFlight::terrestrial(1.0)), valkyrie);
    let mut ctrl = KeyMotion::idle();
    ctrl.register_key(KeyCode::ShiftLeft, true);
    ctrl.register_key(KeyCode::KeyW, true);
    for _ in 0..45 { valk.step_motion(1.0 / 60.0, &ctrl, &tunnel_world); }
    ctrl.register_key(KeyCode::KeyW, false);
    ctrl.register_key(KeyCode::ShiftLeft, false);
    ctrl.register_key(KeyCode::Space, true);
    for frame in 0..60 {
        valk.step_motion(1.0 / 60.0, &ctrl, &tunnel_world);
        let eye = valk.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
        assert!(eye.z < -0.15 && eye.y <= -1.50 - 0.12, "Frame {frame}: Valkyrie under 3-block (-3.0..-1.5m) overhang must never jump with head into ceiling blocks above -1.5m, got {eye:?}");
    }
    let mut tunnel_4 = TerracedRelief::in_span(Box::new(FlatRelief), 16);
    for (u, v) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        for step in 0..6 { tunnel_4.excavate_layer(u, v, -((step as f32) * 0.5) - 0.25); }
    }
    for (u, v) in [(0, -1), (1, -1), (0, -2), (1, -2)] { for layer in [-2.75, -2.25, -1.75, -1.25] { tunnel_4.excavate_layer(u, v, layer); } }
    let tunnel_4_world = DoorframeWorld(tunnel_4);
    let mut valk_4 = PlanetaryAvatar::with_attire(Vec3::new(0.50, -2.0, 0.25), Box::new(crate::flight::SurfaceFlight::terrestrial(1.0)), valkyrie);
    let mut ctrl_4 = KeyMotion::idle();
    ctrl_4.register_key(KeyCode::KeyW, true);
    for _ in 0..45 { valk_4.step_motion(1.0 / 60.0, &ctrl_4, &tunnel_4_world); }
    ctrl_4.register_key(KeyCode::KeyW, false);
    ctrl_4.register_key(KeyCode::Space, true);
    for frame in 0..60 {
        valk_4.step_motion(1.0 / 60.0, &ctrl_4, &tunnel_4_world);
        let eye = valk_4.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
        assert!(eye.z < -0.15 && eye.y <= -1.00 - 0.12, "Frame {frame}: Upright Valkyrie in 4-block (-3.0..-1.0m) tunnel must bonk against -1.0m ceiling without clipping or roof-snapping, got {eye:?}");
    }
}

#[test]
fn test_crane_lowers_built_car_and_clicking_seat_drives_steers_and_brakes() {
    use crate::attire::{HeldRelic, YouthAttire};
    use world::{BiomedCubeWorld, BlockMedium, BlockMold};
    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let rig_mold = BlockMold::cube().in_medium(BlockMedium::CantileverRig);
    let eng_mold = BlockMold::cube().turn_clockwise().in_medium(BlockMedium::SteeringWheel);
    let seat_mold = BlockMold::cube().in_medium(BlockMedium::CabinSeat);
    let wheel_mold = BlockMold::cube().in_medium(BlockMedium::CarWheel);
    let av_pos = Vec3::new(0.25, g0, 1.75);
    assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), Vec3::new(0.0, -0.8, -0.6).normalize(), av_pos, rig_mold));
    assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), Vec3::new(0.0, -0.8, -0.6).normalize(), av_pos, eng_mold));
    let hoist_eye = Vec3::new(1.25, g0 + 1.75, 1.25);
    assert!(world.place_molded_surface_block(hoist_eye, Vec3::NEG_Z, av_pos, seat_mold));
    assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, av_pos, wheel_mold));
    assert!((world.constrain_to_surface(Vec3::new(1.25, 22.0, 0.75), 0.0).y - (g0 + 2.0)).abs() < 1e-4, "Seat must be hoisted on crane before lowering");

    let mut driver = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.25, 22.0, 1.25), 1.0)).with_test_engines();
    driver.apply_attire(YouthAttire::squire().with_relics(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm));
    driver.rotate_degrees(0.0, 35.0);
    assert!(driver.use_right_hand(&mut world), "Clicking the crane mast must lower the car onto the ground");
    let g_seat = world.constrain_to_surface(Vec3::new(1.25, 22.0, 0.75), 0.0).y;
    assert!(g_seat < g0 + 1.0, "Lowered car seat must rest on ground (got {g_seat} vs hoisted {})", g0 + 2.0);

    let mut driver_at_car = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(1.25, 22.0, 1.75), 1.0));
    driver_at_car.rotate_degrees(0.0, 58.0);
    let cam_pos = |a: &PlanetaryAvatar| a.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);
    let stand_eye = cam_pos(&driver_at_car);
    assert!(driver_at_car.use_right_hand(&mut world), "Clicking the car seat must seat the player in the car");
    let seated_eye = cam_pos(&driver_at_car);
    assert!(
        (seated_eye.x - 1.25).abs() < 0.25 && (seated_eye.z - 0.75).abs() < 0.25 && seated_eye.y < stand_eye.y,
        "Player must be seated at the car seat position (got {seated_eye:?} vs stand {stand_eye:?})"
    );
    assert!(driver_at_car.use_right_hand(&mut world), "Clicking the steering wheel while seated must engage driving mode");

    let mut gas = KeyMotion::idle();
    gas.register_key(KeyCode::KeyW, true);
    for _ in 0..15 { driver_at_car.step_motion(1.0 / 60.0, &gas, &world); }
    assert!(cam_pos(&driver_at_car).x > seated_eye.x + 0.4, "CarEngine steering block heading (+X) must dictate initial driving direction");
    gas.register_key(KeyCode::KeyD, true);
    for _ in 0..20 { driver_at_car.step_motion(1.0 / 60.0, &gas, &world); }
    let driven_eye = cam_pos(&driver_at_car);
    assert!(driven_eye.distance(seated_eye) > 1.2 && (driven_eye.z - seated_eye.z).abs() > 0.15, "Accelerating and steering must move and turn the car");

    let mut brake = KeyMotion::idle();
    brake.register_key(KeyCode::KeyS, true);
    for _ in 0..30 { driver_at_car.step_motion(1.0 / 60.0, &brake, &world); }
    let stopped_eye = cam_pos(&driver_at_car);
    driver_at_car.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
    assert!(cam_pos(&driver_at_car).distance(stopped_eye) < 1e-4, "Braking must bring the car to a full stop");
    assert!(driver_at_car.use_right_hand(&mut world), "Clicking again while seated must dismount and park the car");

    struct VertLedger(Vec<Vec3>);
    impl MeshTarget for VertLedger {
        fn push_quad(&mut self, q: world::Quad) { q.transfer(|v0, _, _, _, _, _| self.0.push(v0)); }
    }
    driver_at_car.apply_attire(YouthAttire::squire().with_relics(HeldRelic::CabinSeat, HeldRelic::EmptyPalm));
    let mut turns = Vec::with_capacity(5);
    for step in 0..5 {
        let mut ledger = VertLedger(Vec::new());
        driver_at_car.emit_appearance(&mut ledger);
        turns.push(ledger.0);
        if step < 4 {
            assert!(driver_at_car.dispatch_trigger( crate::controls::InputTrigger::WheelClick ));
        }
    }
    assert!(turns[0] != turns[1] && turns[1] != turns[2] && turns[2] != turns[3] && turns[0] != turns[2] && turns[1] != turns[3] && turns[0] == turns[4]);
}
