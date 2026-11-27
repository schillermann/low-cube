use glam::{Quat, Vec3};
use world::{BiomedCubeWorld, PlanetaryWorld, WorldEarthworks as _};
use crate::avatar::{Avatar as _, AvatarMotor as _, AvatarOptics as _, PlanetaryAvatar};
use crate::controls::KeyMotion;
use super::*;

#[test]
fn test_sensitivity_look_rotation() {
    let mut look = SensitivityLook::calibrated(Box::new(MouseLook::idle()));
    look.rotate(10.0, 5.0);
    let q = look.combine_with_surface(Quat::IDENTITY);
    assert_ne!(q, Quat::IDENTITY);
}

#[test]
fn test_cone_look_confines_mouse_movement_within_small_circular_radius() {
    let mut look = ConeLook::centered(Box::new(MouseLook::idle()), 0.52);
    look.rotate(0.10, 0.08);
    let slight = (look.combine_with_surface(Quat::IDENTITY) * Vec3::NEG_Z).normalize();
    assert!(slight.dot(Vec3::NEG_Z) < 0.995 && slight.dot(Vec3::NEG_Z) > 0.985);

    look.rotate(2.40, 1.90);
    let clamped_diag = (look.combine_with_surface(Quat::IDENTITY) * Vec3::NEG_Z).normalize();
    let max_cos = 0.522f32.cos();
    assert!(
        clamped_diag.dot(Vec3::NEG_Z) >= max_cos && clamped_diag.dot(Vec3::NEG_Z) < 0.880,
        "Diagonal sweep must clamp cleanly onto the circular radius rim, got {clamped_diag:?}"
    );

    look.anchor_heading(0.0, 0.0);
    let recentered = (look.combine_with_surface(Quat::IDENTITY) * Vec3::NEG_Z).normalize();
    assert!((recentered - Vec3::NEG_Z).length() < 1e-5);
}

#[test]
fn test_avatar_in_two_by_two_shaft_cannot_turn_fully_until_widened() {
    use winit::keyboard::KeyCode;
    use crate::controls::KeyboardControls;
    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let rim_y = world.constrain_to_surface(Vec3::new(0.25, 20.0, 0.25), 1.0).y;
    let mut walk_in = KeyMotion::idle();
    walk_in.register_key(KeyCode::KeyW, true);
    for _ in 0..3 {
        let y = world.constrain_to_surface(Vec3::new(0.25, 20.0, 0.25), 0.0).y;
        assert!(world.excavate_surface(Vec3::new(0.25, y + 1.0, 0.25), Vec3::NEG_Y));
    }
    let mut walker_1x1 = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.25, 20.0, 0.65), 1.0));
    for _ in 0..15 {
        walker_1x1.step_motion(1.0 / 60.0, &walk_in, &world);
    }
    let hole_1x1_y = walker_1x1.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO).y;
    assert!((hole_1x1_y - (rim_y + 0.8)).abs() < 0.15, "Avatar must NOT fall into a 1x1 hole (got {hole_1x1_y} vs rim eye {})", rim_y + 0.8);
    for _ in 0..3 {
        let y = world.constrain_to_surface(Vec3::new(0.75, 20.0, 0.25), 0.0).y;
        assert!(world.excavate_surface(Vec3::new(0.75, y + 1.0, 0.25), Vec3::NEG_Y));
    }
    let mut walker_2x1 = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.50, 20.0, 0.65), 1.0));
    for _ in 0..25 {
        walker_2x1.step_motion(1.0 / 60.0, &walk_in, &world);
    }
    let hole_2x1_y = walker_2x1.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO).y;
    assert!(hole_2x1_y < rim_y + 0.8 - 0.8, "Avatar MUST fall into a 2x1 hole (got {hole_2x1_y} vs rim eye {})", rim_y + 0.8);

    let pos = world.constrain_to_surface(Vec3::new(0.50, 20.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(pos);
    avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);

    let forward_of = |a: &PlanetaryAvatar| -> Vec3 {
        let inv = a.view_projection_matrix(1.0).inverse();
        (inv.project_point3(Vec3::new(0.0, 0.0, 0.5)) - inv.project_point3(Vec3::ZERO)).normalize()
    };
    let initial_fwd = forward_of(&avatar);
    avatar.rotate_degrees(25.8, 0.0);
    let slight_fwd = forward_of(&avatar);
    assert!(slight_fwd.x.abs() > 0.25, "Avatar must be able to turn partially inside a 2x1 shaft, got {slight_fwd:?}");

    avatar.rotate_degrees(258.0, 0.0);
    let clamped_right = forward_of(&avatar);
    assert!(clamped_right.dot(initial_fwd) > 0.35, "Avatar in 2x1 shaft must not be able to turn 180/360 degrees because upper body is wider than 1 block, got {clamped_right:?}");

    avatar.rotate_degrees(-515.0, 0.0);
    let clamped_left = forward_of(&avatar);
    assert!(clamped_left.dot(initial_fwd) > 0.35, "Avatar in 2x1 shaft must stop against opposite wall when turning the other way, got {clamped_left:?}");

    let broad_attire = crate::attire::EvaAttire::astronaut()
        .with_adjusted_region(crate::attire::BodyRegion::Torso, 4, 4)
        .with_adjusted_region(crate::attire::BodyRegion::Arms, 4, 4);
    avatar.apply_attire(broad_attire);
    avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
    let broad_clamped = forward_of(&avatar);
    assert!(broad_clamped.dot(initial_fwd) > clamped_left.dot(initial_fwd) + 0.15, "Broader customized upper body (Torso/Arms) must have tighter turning play in a 2x1 shaft ({} vs {})", broad_clamped.dot(initial_fwd), clamped_left.dot(initial_fwd));

    avatar.apply_attire(crate::attire::EvaAttire::astronaut());
    for u in 0..2 {
        let (x, z) = ((u as f32) * 0.5 + 0.25, 0.75);
        for _ in 0..3 {
            let y = world.constrain_to_surface(Vec3::new(x, 20.0, z), 0.0).y;
            assert!(world.excavate_surface(Vec3::new(x, y + 1.0, z), Vec3::NEG_Y));
        }
    }
    let mut walker_2x2 = PlanetaryAvatar::at(world.constrain_to_surface(Vec3::new(0.50, 20.0, 1.15), 1.0));
    for _ in 0..35 {
        walker_2x2.step_motion(1.0 / 60.0, &walk_in, &world);
    }
    let hole_2x2_y = walker_2x2.view_projection_matrix(1.0).inverse().project_point3(Vec3::ZERO).y;
    assert!(hole_2x2_y < rim_y + 0.8 - 1.0, "Avatar walking into a 2x2 hole must fall into it instead of gliding across (got {hole_2x2_y} vs rim eye {})", rim_y + 0.8);
    avatar.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
    avatar.rotate_degrees(180.0, 0.0);
    let full_turn_fwd = forward_of(&avatar);
    assert!(full_turn_fwd.dot(initial_fwd) < -0.5, "Widening shaft to 2x2 must allow full 180/360-degree rotation when upper body width is smaller than 2 blocks, got {full_turn_fwd:?}");

    for (x, z, layers) in [(0.25, 0.25, 4), (0.75, 0.25, 4), (0.25, 0.75, 2), (0.75, 0.75, 2)] {
        for _ in 0..layers {
            let y = world.constrain_to_surface(Vec3::new(x, 20.0, z), 0.0).y;
            assert!(world.excavate_surface(Vec3::new(x, y + 1.0, z), Vec3::NEG_Y));
        }
    }
    let deep_pos = world.constrain_to_surface(Vec3::new(0.45, 20.0, 0.25), 1.0);
    let mut deep_miner = PlanetaryAvatar::at(deep_pos);
    deep_miner.apply_attire(crate::attire::EvaAttire::astronaut().with_relics(crate::attire::HeldRelic::RigWrench, crate::attire::HeldRelic::EmptyPalm));
    deep_miner.step_motion(1.0 / 60.0, &KeyMotion::idle(), &world);
    deep_miner.rotate_degrees(90.0, 20.6);
    assert!(forward_of(&deep_miner).x > 0.85, "Deep 2x2 shaft with 2-block internal step must not wedge shoulders");
    let side_top = world.constrain_to_surface(Vec3::new(1.25, 20.0, 0.25), 0.0).y;
    assert!(!deep_miner.use_right_hand(&mut world) && deep_miner.use_right_hand(&mut world), "Must dig side wall from inside deep 2x2 shaft");
    assert!((world.constrain_to_surface(Vec3::new(1.25, 20.0, 0.25), 0.0).y - side_top).abs() < 1e-4, "Mining a side block in a shaft must keep all blocks above it intact");
}
