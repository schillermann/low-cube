use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{BiomedCubeWorld, MeshTarget, PlanetaryWorld, Quad};
use crate::allocation_audit::count_allocations;
use crate::attire::{BodyPose, HeldRelic, MartialStyle, SightMode, StrideRhythm, Wardrobe, YouthAttire, YouthPhysique};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{KeyboardControls, KeyMotion};
use crate::menu::StudioAnimation;

struct QuadCollector { quads: Vec<Quad> }
impl QuadCollector {
    fn empty() -> Self { Self { quads: Vec::new() } }
    fn count(&self) -> usize { self.quads.len() }

    fn visible_in_frustum(&self, vp: glam::Mat4) -> usize {
        self.quads.iter().filter(|q| {
            q.transfer(|v0, _, _, _, _, _| {
                let clip = vp * v0.extend(1.0);
                clip.w > 0.03 && (clip.x / clip.w).abs() < 1.0 && (clip.y / clip.w).abs() < 1.0 && (clip.z / clip.w).abs() <= 1.0
            })
        }).count()
    }
}

impl MeshTarget for QuadCollector {
    fn push_quad(&mut self, quad: Quad) { self.quads.push(quad); }
}

#[test]
fn test_spawn_contains_only_player_avatar_without_persons_or_items() {
    let world = BiomedCubeWorld::beveled(32, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let attire = YouthAttire::squire();

    let avatar = PlanetaryAvatar::at(spawn);
    let mut spawn_collector = QuadCollector::empty();
    avatar.emit_appearance(&mut spawn_collector);

    let first_person_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let mut expected_collector = QuadCollector::empty();
    attire.emit(&first_person_pose, SightMode::FirstPersonSight, &mut expected_collector);

    assert_eq!(
        spawn_collector.count(),
        expected_collector.count(),
        "Spawn must not contain extra showcase persons or ground items"
    );

    let vp = avatar.view_projection_matrix(16.0 / 9.0);
    assert_eq!(
        spawn_collector.visible_in_frustum(vp),
        0,
        "Looking forward at spawn with empty hands must show an unobstructed view without persons or items"
    );
}

#[test]
fn test_standing_walking_and_running_figures_animate_distinct_gaits_with_zero_allocations() {
    let pos = Vec3::new(0.0, 10.0, 0.0);
    let attire = YouthAttire::squire();

    let (mut stand_quads, mut sneak_quads, mut walk_quads, mut run_quads, mut jump_quads) = (
        QuadCollector::empty(), QuadCollector::empty(), QuadCollector::empty(), QuadCollector::empty(), QuadCollector::empty(),
    );
    StudioAnimation::StandingIdle.emit_figure(pos, 0.0, 0.25, &attire, &mut stand_quads);
    StudioAnimation::CrouchingSneak.emit_figure(pos, 0.0, 0.25, &attire, &mut sneak_quads);
    StudioAnimation::WalkingStride.emit_figure(pos, 0.0, 0.25, &attire, &mut walk_quads);
    StudioAnimation::RunningSprint.emit_figure(pos, 0.0, 0.25, &attire, &mut run_quads);
    StudioAnimation::JumpingLeap.emit_figure(pos, 0.0, 0.55, &attire, &mut jump_quads);

    let extract_vertices = |collector: &QuadCollector| -> Vec<Vec3> {
        collector.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect()
    };

    let (v_stand, v_sneak, v_walk, v_run, v_jump) = (
        extract_vertices(&stand_quads), extract_vertices(&sneak_quads), extract_vertices(&walk_quads), extract_vertices(&run_quads), extract_vertices(&jump_quads),
    );
    assert_ne!(v_stand, v_sneak, "Standing and sneaking animations must produce distinct limb postures");
    assert_ne!(v_sneak, v_walk, "Sneaking and walking animations must produce distinct limb postures");
    assert_ne!(v_walk, v_run, "Walking and running animations must produce distinct limb postures");
    assert_ne!(v_run, v_jump, "Running and jumping animations must produce distinct limb postures");

    let stand_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let crouch_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 0.36, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let leap_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 1.16, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let ((stand_h, stand_k, _, _), (crouch_h, crouch_k, _, _), (leap_h, leap_k, leap_a, _), (_, leap_left_k, _, _)) =
        (stand_pose.leg_joints(1.0), crouch_pose.leg_joints(1.0), leap_pose.leg_joints(1.0), leap_pose.leg_joints(-1.0));
    assert!(crouch_k.z > stand_k.z + 0.18 && crouch_h.y < stand_h.y - 0.30 && leap_k.z < leap_h.z - 0.06 && leap_a.z < leap_k.z - 0.06 && (leap_k.z - leap_left_k.z).abs() < 1e-5);
    assert!(crouch_pose.crown_frame().point(0.0, 0.0, 0.0).z < stand_pose.crown_frame().point(0.0, 0.0, 0.0).z - 0.15);
    assert!((leap_pose.crown_frame().point(0.0, 0.0, 0.0).y - stand_pose.crown_frame().point(0.0, 0.0, 0.0).y).abs() < 1e-5);
    let ((_, stand_elbow, stand_wrist, _), (_, crouch_elbow, crouch_wrist, _)) =
        (stand_pose.external_arm(1.0, false, MartialStyle::Unarmed), crouch_pose.external_arm(1.0, false, MartialStyle::Unarmed));
    assert!(crouch_elbow.z < stand_elbow.z - 0.08 && crouch_wrist.point(0.0, 0.0, 0.0).z < crouch_elbow.z - 0.12 && crouch_wrist.point(0.0, 0.0, 0.0).z < stand_wrist.point(0.0, 0.0, 0.0).z - 0.20);
    for step_idx in 0..16 {
        let run_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::new((step_idx as f32) * std::f32::consts::TAU / 16.0, 1.0), YouthPhysique::SlenderSquire);
        let (shoulder_w, elbow_w, _, _) = run_pose.external_arm(1.0, false, MartialStyle::Unarmed);
        assert!(shoulder_w.y - elbow_w.y >= 0.20 && elbow_w.x > stand_elbow.x + 0.04);
    }

    let world = BiomedCubeWorld::centripetal(32, 2.5, 4.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let (mut avatar, controls) = (PlanetaryAvatar::at(spawn), KeyMotion::idle());
    avatar.step_motion(1.0 / 60.0, &controls, &world);
    let allocs = count_allocations(|| { for _ in 0..120 { avatar.step_motion(1.0 / 60.0, &controls, &world); } });
    assert_eq!(allocs, 0, "PlanetaryAvatar step_motion must allocate zero heap memory");
}

#[test]
fn test_dual_hand_weapons_and_strike_trajectories() {
    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn);
    let controls = KeyMotion::idle();

    avatar.apply_attire(YouthAttire::squire().with_relics(HeldRelic::WoodmanAxe, HeldRelic::KnightlySword));
    let mut dual_quads = QuadCollector::empty();
    avatar.emit_appearance(&mut dual_quads);

    avatar.use_right_hand(&mut world);
    for _ in 0..8 { avatar.step_motion(1.0 / 60.0, &controls, &world); }
    let mut right_striking_quads = QuadCollector::empty();
    avatar.emit_appearance(&mut right_striking_quads);
    let vertices = |c: &QuadCollector| c.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect::<Vec<_>>();
    assert_ne!(vertices(&dual_quads), vertices(&right_striking_quads), "Right mouse strike must animate the right-hand weapon posture");

    avatar.use_left_hand(&mut world);
    for _ in 0..8 { avatar.step_motion(1.0 / 60.0, &controls, &world); }
    let mut left_striking_quads = QuadCollector::empty();
    avatar.emit_appearance(&mut left_striking_quads);
    assert_ne!(vertices(&right_striking_quads), vertices(&left_striking_quads), "Left mouse strike must animate the left-hand weapon posture");

    let rest_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let swing_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::with_cleave(0.0, 0.0, 0.62), YouthPhysique::SlenderSquire);
    let ((_, rest_elbow, rest_wrist, _), (_, axe_elbow, axe_wrist, _)) = (rest_pose.external_arm(1.0, true, MartialStyle::LumberjackCleave), swing_pose.external_arm(1.0, true, MartialStyle::LumberjackCleave));
    let ((_, _, fp_axe_wrist, _), (_, sword_elbow, sword_wrist, _)) = (swing_pose.first_person_arm(1.0, MartialStyle::LumberjackCleave), swing_pose.external_arm(1.0, true, MartialStyle::SwordplaySlash));
    assert!(rest_elbow.z - axe_elbow.z > 0.12 && rest_elbow.z - sword_elbow.z > 0.12);
    let (ext_head, fp_head, fp_left_head) = (axe_wrist.point(-0.025, 0.24, 0.054), fp_axe_wrist.point(-0.025, 0.24, 0.054), swing_pose.first_person_arm(-1.0, MartialStyle::LumberjackCleave).2.point(0.025, 0.24, 0.054));
    let (ext_wrist, fp_wrist, sword_tip) = (axe_wrist.point(0.0, 0.0, 0.0), fp_axe_wrist.point(0.0, 0.0, 0.0), sword_wrist.point(-0.025, 0.68, 0.054));
    assert!(axe_wrist.point(0.0, 0.0, 0.064).y - ext_wrist.y < rest_wrist.point(0.0, 0.0, 0.064).y - rest_wrist.point(0.0, 0.0, 0.0).y);
    assert!(axe_wrist.point(0.0, 0.036, 0.0).x - ext_wrist.x > rest_wrist.point(0.0, 0.036, 0.0).x - rest_wrist.point(0.0, 0.0, 0.0).x + 0.01);
    let full_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::with_cleave(0.0, 0.0, 0.68), YouthPhysique::SlenderSquire);
    let (eye, (_, _, fp_full_axe, _), (_, _, fp_full_sword, _)) = (full_pose.eye_origin(), full_pose.first_person_arm(1.0, MartialStyle::LumberjackCleave), full_pose.first_person_arm(1.0, MartialStyle::SwordplaySlash));
    let (axe_blade_hit, sword_tip_hit) = (fp_full_axe.point(-0.025, 0.218, 0.140) - eye, fp_full_sword.point(-0.025, 0.680, 0.054) - eye);
    assert!(axe_blade_hit.x.abs() < 0.005 && axe_blade_hit.y.abs() < 0.005 && sword_tip_hit.x.abs() < 0.005 && sword_tip_hit.y.abs() < 0.005);
    assert!(ext_head.x > ext_wrist.x + 0.04 && fp_head.x > fp_wrist.x + 0.04 && ext_head.z < ext_wrist.z - 0.14 && sword_tip.z < sword_wrist.point(0.0, 0.0, 0.0).z - 0.50);
    assert!(ext_head.x >= 0.0 && fp_head.x >= 0.0 && fp_left_head.x <= 0.0 && (fp_head.x + fp_left_head.x).abs() < 1e-4);
}

#[test]
fn test_wardrobe_excludes_removed_hand_cycle_keys() {
    let squire = YouthAttire::squire();
    for key in [KeyCode::KeyH, KeyCode::KeyG, KeyCode::Digit8, KeyCode::Digit9] {
        assert!(squire.with_wardrobe_key(key).is_none());
    }
}

#[test]
fn test_satchel_open_shows_weapons_behind_and_outside_satchel_without_overlap() {
    let world = BiomedCubeWorld::beveled(32, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let (mut unarmed, mut armed) = (PlanetaryAvatar::at(spawn), PlanetaryAvatar::at(spawn));
    unarmed.rotate_view(0.0, 380.0);
    armed.rotate_view(0.0, 380.0);
    armed.apply_attire(YouthAttire::squire().with_relics(HeldRelic::WoodmanAxe, HeldRelic::KnightlySword));

    let mut closed_unarmed = QuadCollector::empty();
    unarmed.emit_appearance(&mut closed_unarmed);
    assert!(unarmed.dispatch_wardrobe_key(KeyCode::Tab) && armed.dispatch_wardrobe_key(KeyCode::Tab));
    let (mut open_unarmed, mut open_armed) = (QuadCollector::empty(), QuadCollector::empty());
    unarmed.emit_appearance(&mut open_unarmed);
    armed.emit_appearance(&mut open_armed);
    assert!(open_armed.count() > open_unarmed.count(), "Held tools and weapons must remain visible while satchel menu is open");

    let vp = armed.view_projection_matrix(16.0 / 9.0);
    let satchel_len = open_unarmed.count() - closed_unarmed.count();
    let satchel_quads = &open_unarmed.quads[closed_unarmed.count()..];
    let satchel_max_x = satchel_quads.iter().flat_map(|q| q.transfer(|v0, v1, v2, v3, _, _| [v0, v1, v2, v3]))
        .map(|v| { let c = vp * v.extend(1.0); (c.x / c.w).abs() }).fold(0.0_f32, f32::max);
    let weapon_quads = &open_armed.quads[closed_unarmed.count()..(open_armed.count() - satchel_len)];
    let weapon_min_x = weapon_quads.iter().flat_map(|q| q.transfer(|v0, v1, v2, v3, _, _| [v0, v1, v2, v3]))
        .map(|v| { let c = vp * v.extend(1.0); (c.x / c.w).abs() }).fold(f32::INFINITY, f32::min);
    assert!(satchel_max_x + 0.05 < weapon_min_x, "Satchel ({satchel_max_x}) and held weapons ({weapon_min_x}) must not overlap on screen");

    let mut controls = KeyMotion::idle();
    controls.register_key(KeyCode::KeyW, true);
    controls.register_key(KeyCode::Space, true);
    armed.rotate_view(120.0, -80.0);
    for _ in 0..10 { armed.step_motion(1.0 / 60.0, &controls, &world); }
    assert!(armed.view_projection_matrix(16.0 / 9.0) == vp && armed.dispatch_wardrobe_key(KeyCode::Escape) && !armed.dispatch_wardrobe_key(KeyCode::Escape));

    let mut dual_axes = PlanetaryAvatar::at(spawn);
    dual_axes.apply_attire(YouthAttire::squire().with_relics(HeldRelic::WoodmanAxe, HeldRelic::WoodmanAxe));
    let (mut initial_axes_mesh, mut unpacked_axes_mesh) = (QuadCollector::empty(), QuadCollector::empty());
    dual_axes.emit_appearance(&mut initial_axes_mesh);
    assert!(dual_axes.dispatch_wardrobe_key(KeyCode::KeyQ) && dual_axes.dispatch_wardrobe_key(KeyCode::KeyE));
    assert!(dual_axes.dispatch_wardrobe_key(KeyCode::KeyQ) && dual_axes.dispatch_wardrobe_key(KeyCode::KeyQ));
    assert!(dual_axes.dispatch_wardrobe_key(KeyCode::KeyE) && dual_axes.dispatch_wardrobe_key(KeyCode::KeyE));
    dual_axes.emit_appearance(&mut unpacked_axes_mesh);
    assert_eq!(unpacked_axes_mesh.count(), initial_axes_mesh.count(), "Stowing 2 axes and unpacking both must preserve both axes");
}
