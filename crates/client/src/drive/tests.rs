use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{BiomedCubeWorld, BlockMedium, BlockMold, MeshTarget, PlanetaryWorld, Quad, SurfaceFrame};
use crate::attire::{HeldRelic, MartialStyle, StrideRhythm, YouthAttire};
use crate::avatar::{Avatar, HandGesture, PlanetaryAvatar};
use acoustics::{AcousticWave, EngineRumble, PalmScrape, SpadeStrike};
use crate::controls::{InputTrigger, KeyMotion, KeyboardControls};
use crate::posture::AvatarPosture;

struct VertCollector(Vec<Vec3>);
impl MeshTarget for VertCollector {
    fn push_quad(&mut self, quad: Quad) {
        quad.transfer(|v0, v1, v2, v3, _, _| self.0.extend_from_slice(&[v0, v1, v2, v3]));
    }
}

#[test]
fn test_wheel_types_spin_reverse_after_brake_and_seated_head_and_hands() {
    let (palm_wav, spade_wav, engine_wav) = (
        PalmScrape::earthy().riff_wave(),
        SpadeStrike::forged().riff_wave(),
        EngineRumble::combustion().riff_wave(),
    );
    assert!(palm_wav != spade_wav && spade_wav != engine_wav && palm_wav != engine_wav);

    let (frame, m0) = (
        SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0),
        BlockMold::cube().in_medium(BlockMedium::CarWheel),
    );
    let (m1, m2, m3) = (
        m0.next_form(),
        m0.next_form().next_form(),
        m0.next_form().next_form().next_form(),
    );
    let radii = [
        m0.shape_for(Vec3::NEG_Z, &frame).wheel_radius(),
        m1.shape_for(Vec3::NEG_Z, &frame).wheel_radius(),
        m2.shape_for(Vec3::NEG_Z, &frame).wheel_radius(),
        m3.shape_for(Vec3::NEG_Z, &frame).wheel_radius(),
    ];
    assert!(radii[0] != radii[1] && radii[1] != radii[2] && radii[2] != radii[3] && radii[0] != radii[2]);

    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let (rig_mold, eng_mold, seat_mold) = (
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
        BlockMold::cube().turn_clockwise().in_medium(BlockMedium::SteeringWheel),
        BlockMold::cube().in_medium(BlockMedium::CabinSeat),
    );
    let (bench_mold, stool_mold, throne_mold) = (
        seat_mold.next_form(),
        seat_mold.next_form().next_form(),
        seat_mold.next_form().next_form().next_form(),
    );
    let mut seat_meshes = [
        VertCollector(Vec::new()),
        VertCollector(Vec::new()),
        VertCollector(Vec::new()),
        VertCollector(Vec::new()),
    ];
    for (i, sm) in [seat_mold, bench_mold, stool_mold, throne_mold].into_iter().enumerate() {
        sm.shape_for(Vec3::NEG_Z, &frame).emit_shape(world::GridCell::new(0, 0), 0.0, &frame, Vec3::ONE, &mut seat_meshes[i]);
    }
    assert!(
        seat_meshes[0].0 != seat_meshes[1].0
            && seat_meshes[2].0 == seat_meshes[0].0
            && seat_meshes[3].0 == seat_meshes[1].0
            && seat_meshes[0].0.iter().any(|v| v.z <= 0.05)
            && seat_meshes[0].0.iter().filter(|v| v.y > 0.15).all(|v| v.z > 0.15)
    );
    let (mut pedal_verts, mut wheel_verts, mut axle_verts) = (
        VertCollector(Vec::new()),
        VertCollector(Vec::new()),
        VertCollector(Vec::new()),
    );
    BlockMold::cube().in_medium(BlockMedium::PedalCluster).shape_for(Vec3::NEG_Z, &frame).emit_shape(world::GridCell::new(0, 0), 0.0, &frame, Vec3::ONE, &mut pedal_verts);
    m0.shape_for(Vec3::X, &frame).emit_shape(world::GridCell::new(0, 0), 0.0, &frame, Vec3::ONE, &mut wheel_verts);
    BlockMold::cube().in_medium(BlockMedium::CrossAxle).shape_for(Vec3::X, &frame).emit_shape(world::GridCell::new(0, 0), 0.0, &frame, Vec3::ONE, &mut axle_verts);
    assert!(
        pedal_verts.0.iter().any(|v| (v.y - 0.50).abs() < 1e-4)
            && wheel_verts.0.iter().any(|v| (v.x - 0.50).abs() < 1e-4)
            && wheel_verts.0.iter().any(|v| v.y < -0.04)
            && !axle_verts.0.is_empty()
    );

    let (wheel_mold, av_pos, cam_pos) = (
        m2,
        Vec3::new(0.25, g0, 1.75),
        |a: &PlanetaryAvatar| a.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO),
    );
    assert!(world.place_molded_surface_block(Vec3::new(2.25, g0 + 1.5, 1.25), Vec3::new(0.0, -0.8, -0.6).normalize(), av_pos, bench_mold));
    let mut world_sitter = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(2.25, 22.0, 1.25), 1.0));
    world_sitter.apply_attire(YouthAttire::squire().with_relics(HeldRelic::FieldCamera, HeldRelic::EmptyPalm));
    world_sitter.rotate_degrees(0.0, 58.0);
    assert!(!world_sitter.use_right_hand(&mut world) && world_sitter.release_hand(&mut world));
    let sit_eye = cam_pos(&world_sitter);
    assert!(world_sitter.dispatch_trigger(InputTrigger::WheelClick) && (cam_pos(&world_sitter) - sit_eye).length() < 1e-4);
    let (mut step_d, mut step_a) = (KeyMotion::idle(), KeyMotion::idle());
    step_d.register_key(KeyCode::KeyD, true);
    step_a.register_key(KeyCode::KeyA, true);
    world_sitter.step_motion(1.0 / 60.0, &step_d, &world);
    let stand_right = cam_pos(&world_sitter);
    assert!(stand_right.x > 2.50 && stand_right.y < g0 + 2.0);
    world_sitter = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(2.25, 22.0, 1.25), 1.0));
    world_sitter.rotate_degrees(0.0, 58.0);
    assert!(!world_sitter.use_right_hand(&mut world) && world_sitter.release_hand(&mut world));
    world_sitter.step_motion(1.0 / 60.0, &step_a, &world);
    let stand_left = cam_pos(&world_sitter);
    assert!(stand_left.x < 2.00 && stand_left.y < g0 + 2.0);

    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, av_pos, rig_mold));
    assert!(
        world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, av_pos, BlockMold::cube().in_medium(BlockMedium::PedalCluster)),
        "Any block medium must mount first onto an empty crane without steering-wheel restriction"
    );
    let mut seatless = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.25), 1.0));
    seatless.rotate_degrees(-90.0, 12.0);
    let stand_y = cam_pos(&seatless).y;
    let _ = seatless.use_right_hand(&mut world);
    assert!((cam_pos(&seatless).y - stand_y).abs() < 1e-4, "Without a seat on the crane object, player must not sit");
    assert!(world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, -1, false));

    for (eye, dir, mold) in [
        (Vec3::new(1.25, g0 + 2.40, 0.25), Vec3::NEG_Y, eng_mold),
        (Vec3::new(1.25, g0 + 1.25, 0.75), Vec3::NEG_Z, m1),
        (Vec3::new(1.25, g0 + 1.75, 1.25), Vec3::NEG_Z, seat_mold),
        (Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, m1),
    ] {
        assert!(world.place_molded_surface_block(eye, dir, av_pos, mold));
    }
    let mut miner = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.75, 22.0, 1.25), 1.0));
    miner.apply_attire(YouthAttire::squire().with_relics(HeldRelic::CarWheel, HeldRelic::EmptyPalm));
    assert!(!miner.use_right_hand(&mut world));
    for _ in 0..320 {
        miner.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
        let _ = miner.step_digging(&mut world);
    }
    assert!(!miner.release_hand(&mut world) && world.harvest_surface(Vec3::new(0.75, g0 + 1.80, 1.25), Vec3::NEG_Z).is_none());
    assert!(
        world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.75, 0.75), Vec3::X, av_pos, wheel_mold)
            && world.place_molded_surface_block(
                Vec3::new(1.25, g0 + 2.35, 0.75),
                Vec3::NEG_Y,
                av_pos,
                BlockMold::cube().in_medium(BlockMedium::PlanedPlanks),
            )
    );
    assert!(
        (world.constrain_to_surface(Vec3::new(1.25, 22.0, 1.25), 0.0).y - (g0 + 2.0)).abs() < 1e-4
            && (world.constrain_to_surface(Vec3::new(1.25, 22.0, 0.75), 0.0).y - (g0 + 2.0)).abs() < 1e-4
    );
    let mut driver = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(1.75, 22.0, 0.75), 1.0));
    driver.rotate_degrees(-90.0, 12.0);
    assert!(HandGesture::RightTap.perform(&mut driver, &mut world));
    driver.apply_attire(YouthAttire::squire().with_relics(HeldRelic::FieldCamera, HeldRelic::EmptyPalm));
    assert!(driver.dispatch_trigger(InputTrigger::WheelClick));
    driver.apply_attire(YouthAttire::squire());
    assert!(driver.use_right_hand(&mut world));
    let (mut gas, mut hold_s) = (KeyMotion::idle(), KeyMotion::idle());
    gas.register_key(KeyCode::KeyW, true);
    hold_s.register_key(KeyCode::KeyS, true);
    let view_fwd = |a: &PlanetaryAvatar| (a.view_projection_matrix(1.0).inverse().project_point3(Vec3::new(0.0, 0.0, 0.5)) - cam_pos(a)).normalize();
    let initial_dir = view_fwd(&driver);
    driver.rotate_degrees(240.0, 0.0);
    assert!(view_fwd(&driver).dot(initial_dir) > 0.10);
    let (start_x, mut verts_t0, mut verts_t1) = (cam_pos(&driver).x, VertCollector(Vec::new()), VertCollector(Vec::new()));
    driver.emit_appearance(&mut verts_t0);
    for _ in 0..20 {
        driver.step_motion(1.0 / 60.0, &gas, &world);
    }
    assert!(cam_pos(&driver).x > start_x + 0.5);
    driver.emit_appearance(&mut verts_t1);
    assert_ne!(verts_t0.0, verts_t1.0);
    for _ in 0..18 {
        driver.step_motion(1.0 / 60.0, &hold_s, &world);
    }
    let braked_x = cam_pos(&driver).x;
    for _ in 0..55 {
        driver.step_motion(1.0 / 60.0, &hold_s, &world);
    }
    assert!(cam_pos(&driver).x < braked_x - 0.35);

    assert!(driver.use_left_hand(&mut world));
    driver.rotate_degrees(240.0, 0.0);
    assert!(view_fwd(&driver).dot(initial_dir) > 0.10);
    driver.rotate_degrees(-480.0, 0.0);
    assert!(view_fwd(&driver).dot(initial_dir) > 0.10);
    let parked_eye = cam_pos(&driver);
    assert!(parked_eye.y < g0 + 2.5 && parked_eye.y > g0 + 1.25);
    let mut driven_verts = VertCollector(Vec::new());
    for v in &verts_t1.0 {
        if v.y < g0 + 0.5 {
            driven_verts.0.push(*v);
        }
    }
    assert!(driven_verts.0.iter().map(|v| v.y).fold(f32::MAX, f32::min) >= g0 - 1e-4);
    let eng_top = world.constrain_to_surface(Vec3::new(parked_eye.x, 22.0, 0.75), 0.0).y;
    assert!(eng_top > g0 + 0.65);
    assert!(
        world.harvest_surface(Vec3::new(parked_eye.x, eng_top + 0.5, 0.75), Vec3::NEG_Y).is_none()
            && !world.place_molded_surface_block(Vec3::new(parked_eye.x, eng_top + 0.5, 0.75), Vec3::NEG_Y, av_pos, BlockMold::cube())
    );
    let to_mast = (Vec3::new(0.25, g0 + 0.25, 0.25) - parked_eye).normalize();
    let cur_fwd = view_fwd(&driver);
    let yaw_deg = cur_fwd.x.atan2(-cur_fwd.z).to_degrees() - to_mast.x.atan2(-to_mast.z).to_degrees();
    let pitch_deg = cur_fwd.y.asin().to_degrees() - to_mast.y.asin().to_degrees();
    driver.rotate_degrees(-yaw_deg, pitch_deg);
    assert!(driver.use_right_hand(&mut world) && cam_pos(&driver).y > parked_eye.y + 0.8);

    let pose_c = AvatarPosture::crouching().body_pose(Vec3::ZERO, Quat::IDENTITY, Quat::IDENTITY, StrideRhythm::idle()).with_surface_rotation(Quat::IDENTITY).with_seated_steer(0.0);
    let pose_l = AvatarPosture::crouching().body_pose(Vec3::ZERO, Quat::IDENTITY, Quat::from_rotation_y(0.9), StrideRhythm::idle()).with_surface_rotation(Quat::IDENTITY).with_seated_steer(0.0);
    assert!(pose_c.torso_frame().point(0.2, 0.5, 0.3) == pose_l.torso_frame().point(0.2, 0.5, 0.3) && pose_c.leg_joints(1.0) == pose_l.leg_joints(1.0) && pose_c.crown_frame().point(0.0, 0.1, 0.1) != pose_l.crown_frame().point(0.0, 0.1, 0.1));
    let (r_left, r_right) = (pose_c.with_seated_steer(-1.0).arm(1.0, MartialStyle::Unarmed).external(false).2.point(0.0, 0.0, 0.0), pose_c.with_seated_steer(1.0).arm(1.0, MartialStyle::Unarmed).external(false).2.point(0.0, 0.0, 0.0));
    let (l_left, l_right) = (pose_c.with_seated_steer(-1.0).arm(-1.0, MartialStyle::Unarmed).external(false).2.point(0.0, 0.0, 0.0), pose_c.with_seated_steer(1.0).arm(-1.0, MartialStyle::Unarmed).external(false).2.point(0.0, 0.0, 0.0));
    assert!(r_left.y > r_right.y + 0.08 && l_right.y > l_left.y + 0.08);
}

#[test]
fn test_holding_mouse_button_dismantles_seat_while_short_click_perches() {
    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(2.25, 22.0, 0.25), 0.0).y;
    let seat_mold = BlockMold::cube().in_medium(BlockMedium::CabinSeat);
    let aim = Vec3::new(0.0, -0.8, -0.6).normalize();
    let av_pos = Vec3::new(2.25, g0, 1.75);
    assert!(world.place_molded_surface_block(Vec3::new(2.25, g0 + 1.5, 1.25), aim, av_pos, seat_mold));
    assert!((world.constrain_to_surface(Vec3::new(2.25, 22.0, 0.25), 0.0).y - (g0 + 0.5)).abs() < 1e-4);

    let stand = world.constrain_to_surface(Vec3::new(2.25, 22.0, 1.25), 1.0);
    let cam_pos = |a: &PlanetaryAvatar| a.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO);

    let mut bare_miner = PlanetaryAvatar::at(stand);
    bare_miner.rotate_degrees(0.0, 58.0);
    let stand_eye = cam_pos(&bare_miner);
    assert!(HandGesture::RightHold(280).perform(&mut bare_miner, &mut world));
    assert!((cam_pos(&bare_miner) - stand_eye).length() < 0.05, "Avatar must remain standing while dismantling seat");
    assert!((world.constrain_to_surface(Vec3::new(2.25, 22.0, 0.25), 0.0).y - g0).abs() < 1e-4, "Held mouse button must dismantle the seat");

    assert!(world.place_molded_surface_block(Vec3::new(2.25, g0 + 1.5, 1.25), aim, av_pos, seat_mold));
    bare_miner.apply_attire(YouthAttire::squire().with_relics(HeldRelic::IronSpade, HeldRelic::EmptyPalm));
    assert!(HandGesture::RightTap.perform(&mut bare_miner, &mut world));
    assert!(cam_pos(&bare_miner).z < 0.6, "Short click must perch on the seat");
    bare_miner.rotate_degrees(0.0, 88.0);
    assert!(HandGesture::RightHold(90).perform(&mut bare_miner, &mut world));
    bare_miner.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
    assert!((world.constrain_to_surface(Vec3::new(2.25, 22.0, 0.25), 0.0).y - g0).abs() < 1e-4, "Holding mouse button while seated must also dismantle the seat");
}
