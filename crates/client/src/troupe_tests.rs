use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{BiomedQuadSphere, MeshTarget, PlanetaryWorld, Quad};
use crate::allocation_audit::count_allocations;
use crate::attire::{BodyPose, CrewPhysique, EvaAttire, HeldRelic, MartialStyle, SightMode, StrideRhythm, Wardrobe};
use crate::avatar::{
    Avatar as _, AvatarMotor as _, AvatarOptics as _, AvatarPresence as _, PlanetaryAvatar,
};
use crate::controls::{KeyboardControls, KeyMotion};
use crate::menu::StudioAnimation;

struct QuadLedger { quads: Vec<Quad> }
impl QuadLedger {
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

impl MeshTarget for QuadLedger {
    fn push_quad(&mut self, quad: Quad) { self.quads.push(quad); }
}

#[test]
fn test_spawn_contains_only_player_avatar_without_persons_or_items() {
    let world = BiomedQuadSphere::of_size(32);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let attire = EvaAttire::astronaut();

    let avatar = PlanetaryAvatar::at(spawn);
    let mut spawn_collector = QuadLedger::empty();
    avatar.emit_appearance(&mut spawn_collector);

    let first_person_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::idle(), CrewPhysique::OrbitalCadet);
    let mut expected_collector = QuadLedger::empty();
    attire.emit(&first_person_pose, SightMode::FirstPersonSight, &mut expected_collector);

    assert_eq!(spawn_collector.count(), expected_collector.count());
    let vp = avatar.view_projection_matrix(16.0 / 9.0);
    assert_eq!(spawn_collector.visible_in_frustum(vp), 0);
}

#[test]
fn test_standing_walking_and_running_figures_animate_distinct_gaits_with_zero_allocations() {
    let pos = Vec3::new(0.0, 10.0, 0.0);
    let attire = EvaAttire::astronaut();

    let (mut stand_quads, mut sneak_quads, mut walk_quads, mut run_quads, mut jump_quads) = (
        QuadLedger::empty(), QuadLedger::empty(), QuadLedger::empty(), QuadLedger::empty(), QuadLedger::empty(),
    );
    StudioAnimation::StandingIdle.emit_figure(pos, 0.0, 0.25, &attire, &mut stand_quads);
    StudioAnimation::CrouchingSneak.emit_figure(pos, 0.0, 0.25, &attire, &mut sneak_quads);
    StudioAnimation::WalkingStride.emit_figure(pos, 0.0, 0.25, &attire, &mut walk_quads);
    StudioAnimation::RunningSprint.emit_figure(pos, 0.0, 0.25, &attire, &mut run_quads);
    StudioAnimation::JumpingLeap.emit_figure(pos, 0.0, 0.55, &attire, &mut jump_quads);

    let extract_vertices = |collector: &QuadLedger| -> Vec<Vec3> {
        collector.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect()
    };

    let (v_stand, v_sneak, v_walk, v_run, v_jump) = (
        extract_vertices(&stand_quads), extract_vertices(&sneak_quads), extract_vertices(&walk_quads), extract_vertices(&run_quads), extract_vertices(&jump_quads),
    );
    assert_ne!(v_stand, v_sneak);
    assert_ne!(v_sneak, v_walk);
    assert_ne!(v_walk, v_run);
    assert_ne!(v_run, v_jump);

    let stand_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::idle(), CrewPhysique::OrbitalCadet);
    let crouch_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 0.36, StrideRhythm::idle(), CrewPhysique::OrbitalCadet);
    let leap_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 1.16, StrideRhythm::idle(), CrewPhysique::OrbitalCadet);
    let ((stand_h, stand_k, _, _), (crouch_h, crouch_k, _, _), (leap_h, leap_k, leap_a, _), (_, leap_left_k, _, _)) =
        (stand_pose.leg_joints(1.0), crouch_pose.leg_joints(1.0), leap_pose.leg_joints(1.0), leap_pose.leg_joints(-1.0));
    assert!(crouch_k.z > stand_k.z + 0.18 && crouch_h.y < stand_h.y - 0.30 && leap_k.z < leap_h.z - 0.06 && leap_a.z < leap_k.z - 0.06 && (leap_k.z - leap_left_k.z).abs() < 1e-5);
    assert!(crouch_pose.crown_frame().point(0.0, 0.0, 0.0).z < stand_pose.crown_frame().point(0.0, 0.0, 0.0).z - 0.15);
    assert!((leap_pose.crown_frame().point(0.0, 0.0, 0.0).y - stand_pose.crown_frame().point(0.0, 0.0, 0.0).y).abs() < 1e-5);
    let ((_, stand_elbow, stand_wrist, _), (_, crouch_elbow, crouch_wrist, _)) =
        (stand_pose.arm(1.0, MartialStyle::Unarmed).external(false), crouch_pose.arm(1.0, MartialStyle::Unarmed).external(false));
    assert!(crouch_elbow.z < stand_elbow.z - 0.08 && crouch_wrist.point(0.0, 0.0, 0.0).z < crouch_elbow.z - 0.12 && crouch_wrist.point(0.0, 0.0, 0.0).z < stand_wrist.point(0.0, 0.0, 0.0).z - 0.20);
    for step_idx in 0..16 {
        let run_pose = BodyPose::new(pos, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::new((step_idx as f32) * std::f32::consts::TAU / 16.0, 1.0), CrewPhysique::OrbitalCadet);
        let (shoulder_w, elbow_w, _, _) = run_pose.arm(1.0, MartialStyle::Unarmed).external(false);
        assert!(shoulder_w.y - elbow_w.y >= 0.20 && elbow_w.x > stand_elbow.x + 0.04);
    }

    let world = BiomedQuadSphere::of_size(32);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let (mut avatar, controls) = (PlanetaryAvatar::at(spawn), KeyMotion::idle());
    avatar.step_motion(1.0 / 60.0, &controls, &world);
    let allocs = count_allocations(|| { for _ in 0..120 { avatar.step_motion(1.0 / 60.0, &controls, &world); } });
    assert_eq!(allocs, 0, "PlanetaryAvatar step_motion must allocate zero heap memory");
}

#[test]
fn test_dual_hand_eva_tools_and_torque_trajectories() {
    let mut world = BiomedQuadSphere::of_size(32);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let mut avatar = PlanetaryAvatar::at(spawn);
    let controls = KeyMotion::idle();

    avatar.apply_attire(EvaAttire::eva_equipped());
    let mut dual_quads = QuadLedger::empty();
    avatar.emit_appearance(&mut dual_quads);

    avatar.use_right_hand(&mut world);
    for _ in 0..8 { avatar.step_motion(1.0 / 60.0, &controls, &world); }
    let mut right_striking_quads = QuadLedger::empty();
    avatar.emit_appearance(&mut right_striking_quads);
    let vertices = |c: &QuadLedger| c.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect::<Vec<_>>();
    assert_ne!(vertices(&dual_quads), vertices(&right_striking_quads));

    avatar.use_left_hand(&mut world);
    for _ in 0..8 { avatar.step_motion(1.0 / 60.0, &controls, &world); }
    let mut left_striking_quads = QuadLedger::empty();
    avatar.emit_appearance(&mut left_striking_quads);
    assert_ne!(vertices(&right_striking_quads), vertices(&left_striking_quads));

    let rest_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::idle(), CrewPhysique::OrbitalCadet);
    let swing_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::with_cleave(0.0, 0.0, 0.62), CrewPhysique::OrbitalCadet);
    let ((_, rest_elbow, rest_wrist, _), (_, wrench_elbow, wrench_wrist, _)) = (rest_pose.arm(1.0, MartialStyle::RigTorque).external(true), swing_pose.arm(1.0, MartialStyle::RigTorque).external(true));
    let ((_, _, fp_wrench_wrist, _), (_, radio_elbow, radio_wrist, _)) = (swing_pose.arm(1.0, MartialStyle::RigTorque).first_person(), swing_pose.arm(1.0, MartialStyle::RadioSignal).external(true));
    assert!(rest_elbow.z - wrench_elbow.z > 0.12 && rest_elbow.z - radio_elbow.z > 0.12);
    let (ext_head, fp_head, fp_left_head) = (wrench_wrist.point(-0.025, 0.24, 0.054), fp_wrench_wrist.point(-0.025, 0.24, 0.054), swing_pose.arm(-1.0, MartialStyle::RigTorque).first_person().2.point(0.025, 0.24, 0.054));
    let (ext_wrist, fp_wrist, radio_tip) = (wrench_wrist.point(0.0, 0.0, 0.0), fp_wrench_wrist.point(0.0, 0.0, 0.0), radio_wrist.point(-0.025, 0.68, 0.054));
    assert!(wrench_wrist.point(0.0, 0.0, 0.064).y - ext_wrist.y < rest_wrist.point(0.0, 0.0, 0.064).y - rest_wrist.point(0.0, 0.0, 0.0).y);
    assert!(wrench_wrist.point(0.0, 0.036, 0.0).x - ext_wrist.x > rest_wrist.point(0.0, 0.036, 0.0).x - rest_wrist.point(0.0, 0.0, 0.0).x + 0.01);
    let full_pose = BodyPose::new(spawn, Quat::IDENTITY, Quat::IDENTITY, 0.8, StrideRhythm::with_cleave(0.0, 0.0, 0.68), CrewPhysique::OrbitalCadet);
    let (eye, (_, _, fp_full_wrench, _), (_, _, fp_full_radio, _)) = (full_pose.eye_origin(), full_pose.arm(1.0, MartialStyle::RigTorque).first_person(), full_pose.arm(1.0, MartialStyle::RadioSignal).first_person());
    let (wrench_jaw_hit, radio_tip_hit) = (fp_full_wrench.point(-0.025, 0.218, 0.140) - eye, fp_full_radio.point(-0.025, 0.680, 0.054) - eye);
    assert!(wrench_jaw_hit.x.abs() < 0.005 && wrench_jaw_hit.y.abs() < 0.005 && radio_tip_hit.x.abs() < 0.005 && radio_tip_hit.y.abs() < 0.005);
    assert!(ext_head.x > ext_wrist.x + 0.04 && fp_head.x > fp_wrist.x + 0.04 && ext_head.z < ext_wrist.z - 0.14 && radio_tip.z < radio_wrist.point(0.0, 0.0, 0.0).z - 0.50);
    assert!(ext_head.x >= 0.0 && fp_head.x >= 0.0 && fp_left_head.x <= 0.0 && (fp_head.x + fp_left_head.x).abs() < 1e-4);
}

#[test]
fn test_wardrobe_excludes_removed_hand_cycle_keys() {
    let cadet = EvaAttire::astronaut();
    for key in [KeyCode::KeyH, KeyCode::KeyG, KeyCode::Digit8, KeyCode::Digit9] {
        assert!(cadet.with_wardrobe_key(key).is_none());
    }
}

#[test]
fn test_satchel_open_shows_eva_tools_behind_and_outside_satchel_without_overlap() {
    let world = BiomedQuadSphere::of_size(32);
    let spawn = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let (mut unarmed, mut armed) = (PlanetaryAvatar::at(spawn), PlanetaryAvatar::at(spawn));
    unarmed.rotate_view(0.0, 380.0);
    armed.rotate_view(0.0, 380.0);
    armed.apply_attire(EvaAttire::eva_equipped());

    let mut closed_unarmed = QuadLedger::empty();
    unarmed.emit_appearance(&mut closed_unarmed);
    assert!(unarmed.dispatch_wardrobe_key(KeyCode::Tab) && armed.dispatch_wardrobe_key(KeyCode::Tab));
    let (mut open_unarmed, mut open_armed, mut satchel_mesh) =
        (QuadLedger::empty(), QuadLedger::empty(), QuadLedger::empty());
    unarmed.emit_appearance(&mut open_unarmed);
    armed.emit_appearance(&mut open_armed);
    armed.emit_inspection(&mut satchel_mesh);
    assert_eq!(open_unarmed.count(), closed_unarmed.count());
    assert!(open_armed.count() > open_unarmed.count());
    assert!(satchel_mesh.count() > 100);

    let vp = armed.view_projection_matrix(16.0 / 9.0);
    let satchel_max_x = satchel_mesh.quads.iter().flat_map(|q| q.transfer(|v0, v1, v2, v3, _, _| [v0, v1, v2, v3]))
        .map(|v| { let c = vp * v.extend(1.0); (c.x / c.w).abs() }).fold(0.0_f32, f32::max);
    let tool_quads = &open_armed.quads[closed_unarmed.count()..];
    let tool_min_x = tool_quads.iter().flat_map(|q| q.transfer(|v0, v1, v2, v3, _, _| [v0, v1, v2, v3]))
        .map(|v| { let c = vp * v.extend(1.0); (c.x / c.w).abs() }).fold(f32::INFINITY, f32::min);
    assert!(satchel_max_x + 0.05 < tool_min_x);

    let mut controls = KeyMotion::idle();
    controls.register_key(KeyCode::KeyW, true);
    controls.register_key(KeyCode::Space, true);
    armed.rotate_view(120.0, -80.0);
    for _ in 0..10 { armed.step_motion(1.0 / 60.0, &controls, &world); }
    assert!(armed.view_projection_matrix(16.0 / 9.0) == vp && armed.dispatch_wardrobe_key(KeyCode::Escape) && !armed.dispatch_wardrobe_key(KeyCode::Escape));

    let mut dual_wrenches = PlanetaryAvatar::at(spawn);
    dual_wrenches.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::RigWrench, HeldRelic::RigWrench));
    let (mut initial_mesh, mut unpacked_mesh) = (QuadLedger::empty(), QuadLedger::empty());
    dual_wrenches.emit_appearance(&mut initial_mesh);
    assert!(dual_wrenches.dispatch_wardrobe_key(KeyCode::KeyQ) && dual_wrenches.dispatch_wardrobe_key(KeyCode::KeyE));
    assert!(dual_wrenches.dispatch_wardrobe_key(KeyCode::KeyQ) && dual_wrenches.dispatch_wardrobe_key(KeyCode::KeyQ));
    assert!(dual_wrenches.dispatch_wardrobe_key(KeyCode::KeyE) && dual_wrenches.dispatch_wardrobe_key(KeyCode::KeyE));
    dual_wrenches.emit_appearance(&mut unpacked_mesh);
    assert_eq!(unpacked_mesh.count(), initial_mesh.count());
}

#[test]
fn test_multiplayer_companion_figure_renders_ore_relics_and_live_host_guest_sync() {
    use sanctuary::{HostedPlanet, JoinTarget, RemotePlanet, SanctuaryHost};
    use crate::companion::CompanionFigure;

    let base_world: Box<dyn PlanetaryWorld> = Box::new(BiomedQuadSphere::of_size(16));
    let raw_host = SanctuaryHost::bind("127.0.0.1:0", base_world, 16).expect("bind");
    let ticket = raw_host.ticket().encode();
    let mut host_planet = HostedPlanet::new(raw_host);
    let mut guest_planet = RemotePlanet::dial(JoinTarget::parse(&ticket).expect("ticket"), 16, 0.5).expect("dial");

    let spawn = host_planet.constrain_to_surface(Vec3::new(0.25, 12.0, 0.25), 1.0);
    let mut host_avatar = PlanetaryAvatar::at(spawn);
    let mut guest_avatar = PlanetaryAvatar::at(spawn + Vec3::X * 1.5);
    host_avatar.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::GoldOre, HeldRelic::CopperOre));
    guest_avatar.apply_attire(EvaAttire::astronaut().with_relics(HeldRelic::IronOre, HeldRelic::RigWrench));

    for _ in 0..8 {
        host_avatar.sync_world_presence(&mut host_planet);
        guest_avatar.sync_world_presence(&mut guest_planet);
        let _ = guest_planet.step_surface(0.06);
        let _ = host_planet.step_surface(0.06);
    }

    let (mut host_view_companions, mut guest_view_companions) = (QuadLedger::empty(), QuadLedger::empty());
    CompanionFigure::emit_all(&host_planet, &mut host_view_companions);
    CompanionFigure::emit_all(&guest_planet, &mut guest_view_companions);
    assert!(host_view_companions.count() > 80);
    assert!(guest_view_companions.count() > 80);
}
