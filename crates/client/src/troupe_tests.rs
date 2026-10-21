use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{BiomedCubeWorld, MeshTarget, PlanetaryWorld, Quad};
use crate::allocation_audit::count_allocations;
use crate::attire::{BodyPose, MartialStyle, SightMode, StrideRhythm, Wardrobe, YouthAttire, YouthPhysique};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::KeyMotion;
use crate::troupe::{FigureTroupe, GaitFigure, SpawnTroupe, TroupeAvatar};

struct QuadCollector {
    quads: Vec<Quad>,
}

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
fn test_spawn_troupe_emits_three_characters_and_ground_axe_visible_from_spawn() {
    let world = BiomedCubeWorld::beveled(32, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let attire = YouthAttire::squire();

    let solo_avatar = PlanetaryAvatar::at(spawn);
    let mut solo_collector = QuadCollector::empty();
    solo_avatar.emit_appearance(&mut solo_collector);

    let peer_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let mut peer_collector = QuadCollector::empty();
    attire.emit(&peer_pose, SightMode::LookingGlassSight, &mut peer_collector);

    let troupe_avatar = TroupeAvatar::near_spawn(Box::new(PlanetaryAvatar::at(spawn)), spawn, &world, attire);
    let mut troupe_collector = QuadCollector::empty();
    troupe_avatar.emit_appearance(&mut troupe_collector);

    // TroupeAvatar must emit the player's first-person mesh + 5 third-person characters + grounded axe
    assert!(
        troupe_collector.count() > solo_collector.count() + 5 * peer_collector.count(),
        "TroupeAvatar must emit first-person body, 5 complete third-person characters, and the ground axe near spawn"
    );

    let vp = troupe_avatar.view_projection_matrix(16.0 / 9.0);
    assert!(
        troupe_collector.visible_in_frustum(vp) > solo_collector.visible_in_frustum(vp) + 4 * peer_collector.count(),
        "All five characters and the axe near spawn must stand inside the player's initial field of view"
    );
}

#[test]
fn test_standing_walking_and_running_figures_animate_distinct_gaits_with_zero_allocations() {
    let pos = Vec3::new(0.0, 10.0, 0.0);
    let attire = YouthAttire::squire();

    let stand = GaitFigure::standing_at(pos, Quat::IDENTITY, Quat::IDENTITY).step(0.25);
    let sneak = GaitFigure::sneaking_at(pos, Quat::IDENTITY, Quat::IDENTITY).step(0.25);
    let walk = GaitFigure::walking_at(pos, Quat::IDENTITY, Quat::IDENTITY).step(0.25);
    let run = GaitFigure::running_at(pos, Quat::IDENTITY, Quat::IDENTITY).step(0.25);
    let jump = GaitFigure::jumping_at(pos, Quat::IDENTITY, Quat::IDENTITY).step(0.25);

    let (mut stand_quads, mut sneak_quads, mut walk_quads, mut run_quads, mut jump_quads) = (QuadCollector::empty(), QuadCollector::empty(), QuadCollector::empty(), QuadCollector::empty(), QuadCollector::empty());

    stand.emit(&attire, &mut stand_quads);
    sneak.emit(&attire, &mut sneak_quads);
    walk.emit(&attire, &mut walk_quads);
    run.emit(&attire, &mut run_quads);
    jump.emit(&attire, &mut jump_quads);

    let extract_vertices = |collector: &QuadCollector| -> Vec<Vec3> {
        collector.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect()
    };

    let (v_stand, v_sneak, v_walk, v_run, v_jump) = (extract_vertices(&stand_quads), extract_vertices(&sneak_quads), extract_vertices(&walk_quads), extract_vertices(&run_quads), extract_vertices(&jump_quads));
    assert_ne!(v_stand, v_sneak, "Standing and sneaking animations must produce distinct limb postures");
    assert_ne!(v_sneak, v_walk, "Sneaking and walking animations must produce distinct limb postures");
    assert_ne!(v_walk, v_run, "Walking and running animations must produce distinct limb postures");
    assert_ne!(v_run, v_jump, "Running and jumping animations must produce distinct limb postures");

    let stand_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let crouch_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 0.36, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let leap_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 1.16, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let ((stand_h, stand_k, _, _), (crouch_h, crouch_k, _, _), (leap_h, leap_k, leap_a, _), (_, leap_left_k, _, _)) = (stand_pose.leg_joints(1.0), crouch_pose.leg_joints(1.0), leap_pose.leg_joints(1.0), leap_pose.leg_joints(-1.0));
    assert!(crouch_k.z > stand_k.z + 0.18 && crouch_h.y < stand_h.y - 0.30 && leap_k.z < leap_h.z - 0.06 && leap_a.z < leap_k.z - 0.06 && (leap_k.z - leap_left_k.z).abs() < 1e-5, "Sneak and jump must bend both knees deeply and lower the hips");
    assert!(crouch_pose.crown_frame().point(0.0, 0.0, 0.0).z < stand_pose.crown_frame().point(0.0, 0.0, 0.0).z - 0.15, "Upper body and neck must lean forward during sneaking");
    assert!((leap_pose.crown_frame().point(0.0, 0.0, 0.0).y - stand_pose.crown_frame().point(0.0, 0.0, 0.0).y).abs() < 1e-5, "Head must remain attached to the torso during a jump");
    let ((_, stand_elbow, stand_wrist, _), (_, crouch_elbow, crouch_wrist, _)) = (stand_pose.external_arm(1.0, false, MartialStyle::Unarmed), crouch_pose.external_arm(1.0, false, MartialStyle::Unarmed));
    assert!(crouch_elbow.z < stand_elbow.z - 0.08 && crouch_wrist.point(0.0, 0.0, 0.0).z < crouch_elbow.z - 0.12 && crouch_wrist.point(0.0, 0.0, 0.0).z < stand_wrist.point(0.0, 0.0, 0.0).z - 0.20 && crouch_wrist.point(0.0, 0.0, 0.0).x < crouch_elbow.x - 0.05 && crouch_wrist.point(0.0, 0.0, 0.064).x < crouch_wrist.point(0.0, 0.0, 0.0).x - 0.02 && crouch_wrist.point(0.0, 0.036, 0.0).x < crouch_wrist.point(0.0, 0.0, 0.0).x - 0.005, "Sneaking arms must bend forward, angle forearms inward, and rotate hands diagonally inward");
    for step_idx in 0..16 {
        let run_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::new((step_idx as f32) * std::f32::consts::TAU / 16.0, 1.0), YouthPhysique::SlenderSquire);
        let (shoulder_w, elbow_w, _, _) = run_pose.external_arm(1.0, false, MartialStyle::Unarmed);
        assert!(shoulder_w.y - elbow_w.y >= 0.20 && elbow_w.x > stand_elbow.x + 0.04, "Running elbow must abduct slightly while remaining below shoulder");
    }

    let world = BiomedCubeWorld::centripetal(32, 2.5, 4.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let (mut avatar, controls) = (TroupeAvatar::near_spawn(Box::new(PlanetaryAvatar::at(spawn)), spawn, &world, attire), KeyMotion::idle());
    avatar.step_motion(1.0 / 60.0, &controls, &world);
    let allocs = count_allocations(|| { for _ in 0..120 { avatar.step_motion(1.0 / 60.0, &controls, &world); } });
    assert_eq!(allocs, 0, "TroupeAvatar step_motion must allocate zero heap memory");
}

#[test]
fn test_axe_pickup_and_throw_cycle_near_spawn() {
    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let mut avatar = TroupeAvatar::near_spawn(Box::new(PlanetaryAvatar::at(spawn)), spawn, &world, YouthAttire::squire());
    let controls = KeyMotion::idle();

    let mut initial_quads = QuadCollector::empty();
    avatar.emit_appearance(&mut initial_quads);

    // Press E at spawn (within 2.6m of the axe at 1.6m) to pick up the WoodmanAxe in the right hand
    assert!(avatar.dispatch_wardrobe_key(KeyCode::KeyE));
    avatar.step_motion(1.0 / 60.0, &controls, &world);

    let mut wielded_quads = QuadCollector::empty();
    avatar.emit_appearance(&mut wielded_quads);
    assert!(wielded_quads.count() > initial_quads.count(), "Picking up the axe must equip it on the player and the 3 showcase characters");

    // Press E a second time to pick up the KnightlySword into the left hand while keeping the WoodmanAxe in the right hand
    assert!(avatar.dispatch_wardrobe_key(KeyCode::KeyE));
    avatar.step_motion(1.0 / 60.0, &controls, &world);
    let mut dual_quads = QuadCollector::empty();
    avatar.emit_appearance(&mut dual_quads);
    assert!(dual_quads.count() > wielded_quads.count(), "Player must be able to hold items in both hands simultaneously");

    // Right mouse click triggers right-hand weapon strike, left mouse click triggers left-hand weapon strike
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

    // Press Q twice to throw both held weapons back onto the ground
    assert!(avatar.dispatch_wardrobe_key(KeyCode::KeyQ));
    avatar.step_motion(1.0 / 60.0, &controls, &world);
    assert!(avatar.dispatch_wardrobe_key(KeyCode::KeyQ));
    for _ in 0..45 { avatar.step_motion(1.0 / 60.0, &controls, &world); }
    let mut thrown_quads = QuadCollector::empty();
    avatar.emit_appearance(&mut thrown_quads);
    assert_eq!(thrown_quads.count(), initial_quads.count(), "Throwing both weapons must unequip them from both hands and land them on the ground");

    let rest_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let swing_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::with_cleave(0.0, 0.0, 0.62), YouthPhysique::SlenderSquire);
    let ((_, rest_elbow, rest_wrist, _), (_, axe_elbow, axe_wrist, _)) = (rest_pose.external_arm(1.0, true, MartialStyle::LumberjackCleave), swing_pose.external_arm(1.0, true, MartialStyle::LumberjackCleave));
    let ((_, _, fp_axe_wrist, _), (_, sword_elbow, sword_wrist, _)) = (swing_pose.first_person_arm(1.0, MartialStyle::LumberjackCleave), swing_pose.external_arm(1.0, true, MartialStyle::SwordplaySlash));
    assert!(rest_elbow.z - axe_elbow.z > 0.12 && rest_elbow.z - sword_elbow.z > 0.12, "Upper arm elbow must surge forward during both axe and sword swings");
    let (ext_head, fp_head) = (axe_wrist.point(-0.025, 0.24, 0.054), fp_axe_wrist.point(-0.025, 0.24, 0.054));
    let fp_left_head = swing_pose.first_person_arm(-1.0, MartialStyle::LumberjackCleave).2.point(0.025, 0.24, 0.054);
    let (ext_wrist, fp_wrist, sword_tip) = (axe_wrist.point(0.0, 0.0, 0.0), fp_axe_wrist.point(0.0, 0.0, 0.0), sword_wrist.point(-0.025, 0.68, 0.054));
    assert!(axe_wrist.point(0.0, 0.0, 0.064).y - ext_wrist.y < rest_wrist.point(0.0, 0.0, 0.064).y - rest_wrist.point(0.0, 0.0, 0.0).y, "Hand must hinge downward at the wrist during a strike");
    assert!(axe_wrist.point(0.0, 0.036, 0.0).x - ext_wrist.x > rest_wrist.point(0.0, 0.036, 0.0).x - rest_wrist.point(0.0, 0.0, 0.0).x + 0.01, "Hand and axe must rotate outward together during an axe strike");
    let full_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::with_cleave(0.0, 0.0, 0.68), YouthPhysique::SlenderSquire);
    let mid_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::with_cleave(0.0, 0.0, 0.51), YouthPhysique::SlenderSquire);
    let (eye, (_, _, fp_full_axe, _), (_, _, fp_full_sword, _)) = (full_pose.eye_origin(), full_pose.first_person_arm(1.0, MartialStyle::LumberjackCleave), full_pose.first_person_arm(1.0, MartialStyle::SwordplaySlash));
    let (axe_blade_hit, sword_tip_hit) = (fp_full_axe.point(-0.025, 0.218, 0.140) - eye, fp_full_sword.point(-0.025, 0.680, 0.054) - eye);
    assert!(axe_blade_hit.x.abs() < 0.005 && axe_blade_hit.y.abs() < 0.005 && sword_tip_hit.x.abs() < 0.005 && sword_tip_hit.y.abs() < 0.005, "Axe blade and sword tip must aim precisely at the center crosshair at full strike");
    let wind_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::with_cleave(0.0, 0.0, 0.34), YouthPhysique::SlenderSquire);
    let ((_, wind_axe_elbow, wind_axe_wrist, _), (_, wind_sword_elbow, _, _)) = (wind_pose.external_arm(1.0, true, MartialStyle::LumberjackCleave), wind_pose.external_arm(1.0, true, MartialStyle::SwordplaySlash));
    let (wind_w, mid_w, full_w) = (wind_pose.first_person_arm(1.0, MartialStyle::LumberjackCleave).2.point(0.0, 0.0, 0.0), mid_pose.first_person_arm(1.0, MartialStyle::LumberjackCleave).2.point(0.0, 0.0, 0.0), fp_full_axe.point(0.0, 0.0, 0.0));
    assert!(wind_axe_wrist.point(0.0, 0.0, 0.0).y > wind_axe_elbow.y + 0.12 && mid_w.x > wind_w.x + 0.05 && mid_w.x > full_w.x + 0.25 && wind_w.y > mid_w.y && mid_w.y > full_w.y && (full_w.z - mid_w.z).abs() < 0.02, "Axe swing must raise forearm upward and sweep downward in a semicircular arc without shooting forward");
    assert!(sword_elbow.x <= rest_elbow.x && wind_sword_elbow.x <= rest_elbow.x + 0.01, "Elbow must not flare outward during a sword attack");
    assert!(ext_head.x > ext_wrist.x + 0.04 && fp_head.x > fp_wrist.x + 0.04, "Axe head must tilt laterally outward in 3D and 2D screen space");
    assert!(ext_head.z < ext_wrist.z - 0.14 && fp_head.z < fp_wrist.z - 0.14 && sword_tip.z < sword_wrist.point(0.0, 0.0, 0.0).z - 0.50 && fp_full_sword.point(-0.025, 0.680, 0.054).z < fp_full_sword.point(0.0, 0.0, 0.0).z - 0.62, "Downward wrist hinge must tilt axe and sword strongly forward at the end of the strike");
    assert!(ext_head.x >= 0.0 && fp_head.x >= 0.0 && fp_left_head.x <= 0.0 && (fp_head.x + fp_left_head.x).abs() < 1e-4, "Both hands must converge symmetrically onto the center crosshair");
    assert_ne!(ext_wrist, sword_wrist.point(0.0, 0.0, 0.0), "Axe and sword must have distinct combat swing trajectories");
}

#[test]
fn test_spawn_troupe_synchronizes_wardrobe_with_key_changes() {
    let world = BiomedCubeWorld::beveled(16, 1.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 9.0, 0.0), 1.0);
    let mut troupe = SpawnTroupe::near_spawn(spawn, &world, YouthAttire::squire());
    let mut before = QuadCollector::empty();
    troupe.emit(&mut before);
    troupe.sync_wardrobe(KeyCode::KeyH);
    let mut after = QuadCollector::empty();
    troupe.emit(&mut after);
    assert!(after.count() > before.count());
}
