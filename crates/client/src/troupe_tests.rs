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

    let peer_pose = BodyPose::new(
        spawn,
        Quat::IDENTITY,
        Quat::IDENTITY,
        0.8,
        StrideRhythm::idle(),
        YouthPhysique::SlenderSquire,
    );
    let mut peer_collector = QuadCollector::empty();
    attire.emit(&peer_pose, SightMode::LookingGlassSight, &mut peer_collector);

    let troupe_avatar = TroupeAvatar::near_spawn(
        Box::new(PlanetaryAvatar::at(spawn)),
        spawn,
        &world,
        attire,
    );
    let mut troupe_collector = QuadCollector::empty();
    troupe_avatar.emit_appearance(&mut troupe_collector);

    // TroupeAvatar must emit the player's first-person mesh + 3 third-person characters + grounded axe
    assert!(
        troupe_collector.count() > solo_collector.count() + 3 * peer_collector.count(),
        "TroupeAvatar must emit first-person body, 3 complete third-person characters, and the ground axe near spawn"
    );

    let vp = troupe_avatar.view_projection_matrix(16.0 / 9.0);
    assert!(
        troupe_collector.visible_in_frustum(vp) > solo_collector.visible_in_frustum(vp) + 2 * peer_collector.count(),
        "All three characters and the axe near spawn must stand inside the player's initial field of view"
    );
}

#[test]
fn test_standing_walking_and_running_figures_animate_distinct_gaits_with_zero_allocations() {
    let pos = Vec3::new(0.0, 10.0, 0.0);
    let attire = YouthAttire::squire();

    let stand = GaitFigure::standing_at(pos, Quat::IDENTITY, Quat::IDENTITY).step(0.25);
    let walk = GaitFigure::walking_at(pos, Quat::IDENTITY, Quat::IDENTITY).step(0.25);
    let run = GaitFigure::running_at(pos, Quat::IDENTITY, Quat::IDENTITY).step(0.25);

    let mut stand_quads = QuadCollector::empty();
    let mut walk_quads = QuadCollector::empty();
    let mut run_quads = QuadCollector::empty();

    stand.emit(&attire, &mut stand_quads);
    walk.emit(&attire, &mut walk_quads);
    run.emit(&attire, &mut run_quads);

    let extract_vertices = |collector: &QuadCollector| -> Vec<Vec3> {
        collector.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect()
    };

    let (v_stand, v_walk, v_run) = (extract_vertices(&stand_quads), extract_vertices(&walk_quads), extract_vertices(&run_quads));
    assert_ne!(v_stand, v_walk, "Standing and walking animations must produce distinct limb postures");
    assert_ne!(v_walk, v_run, "Walking and running animations must produce distinct limb postures");
    assert_ne!(v_stand, v_run, "Standing and running animations must produce distinct limb postures");

    // Verify that elbows abduct slightly outward when running while remaining well below shoulder height
    let stand_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::idle(), YouthPhysique::SlenderSquire);
    let (_, stand_elbow, _, _) = stand_pose.external_arm(1.0, false, MartialStyle::Unarmed);
    for step_idx in 0..16 {
        let phase = (step_idx as f32) * std::f32::consts::TAU / 16.0;
        let run_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::new(phase, 1.0), YouthPhysique::SlenderSquire);
        let (shoulder_w, elbow_w, _, _) = run_pose.external_arm(1.0, false, MartialStyle::Unarmed);
        assert!(shoulder_w.y - elbow_w.y >= 0.20, "Elbow must remain at least 20cm below shoulder during running");
        assert!(elbow_w.x > stand_elbow.x + 0.04, "Running elbow must lift slightly outward laterally compared to standing");
    }

    // Verify zero heap allocations during troupe step_motion
    let world = BiomedCubeWorld::centripetal(32, 2.5, 4.0);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let mut avatar = TroupeAvatar::near_spawn(Box::new(PlanetaryAvatar::at(spawn)), spawn, &world, attire);
    let controls = KeyMotion::idle();
    avatar.step_motion(1.0 / 60.0, &controls, &world);

    let allocs = count_allocations(|| {
        for _ in 0..120 {
            avatar.step_motion(1.0 / 60.0, &controls, &world);
        }
    });
    assert_eq!(allocs, 0, "TroupeAvatar step_motion must allocate zero heap memory");
}

#[test]
fn test_axe_pickup_and_throw_cycle_near_spawn() {
    let world = BiomedCubeWorld::beveled(32, 1.0);
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

    // Mouse click triggers weapon strike animation
    avatar.trigger_strike();
    for _ in 0..8 { avatar.step_motion(1.0 / 60.0, &controls, &world); }
    let mut striking_quads = QuadCollector::empty();
    avatar.emit_appearance(&mut striking_quads);
    let vertices = |c: &QuadCollector| c.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect::<Vec<_>>();
    assert_ne!(vertices(&dual_quads), vertices(&striking_quads), "Triggering a strike must animate the weapon posture");

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
    let (_, rest_elbow, _, _) = rest_pose.external_arm(1.0, true, MartialStyle::LumberjackCleave);
    let (_, axe_elbow, axe_wrist, _) = swing_pose.external_arm(1.0, true, MartialStyle::LumberjackCleave);
    let (_, _, fp_axe_wrist, _) = swing_pose.first_person_arm(1.0, MartialStyle::LumberjackCleave);
    let (_, sword_elbow, sword_wrist, _) = swing_pose.external_arm(1.0, true, MartialStyle::SwordplaySlash);
    assert!(rest_elbow.z - axe_elbow.z > 0.12 && rest_elbow.z - sword_elbow.z > 0.12, "Upper arm elbow must surge forward during both axe and sword swings");
    let (ext_head, fp_head) = (axe_wrist.point(-0.025, 0.24, 0.054), fp_axe_wrist.point(-0.025, 0.24, 0.054));
    let (ext_wrist, fp_wrist) = (axe_wrist.point(0.0, 0.0, 0.0), fp_axe_wrist.point(0.0, 0.0, 0.0));
    assert!(ext_head.x < ext_wrist.x - 0.08 && fp_head.x < fp_wrist.x - 0.08 && fp_head.z < fp_wrist.z, "Axe handle must tilt inward and forward in 3D and 2D screen space");
    assert!(ext_head.x >= 0.0 && fp_head.x >= 0.0, "Axe head must stop at or before the body center line (x >= 0.0)");
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
