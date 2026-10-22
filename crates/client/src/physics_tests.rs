use std::fs;
use glam::Vec3;
use world::{ClimateZone, CubePlanet, NullFace, PlanarFace, PlanetaryWorld, VoxelMountain};
use crate::argument_physics::ArgumentPhysics;
use crate::environment_physics::EnvironmentPhysics;
use crate::file_physics::FilePhysics;
use crate::gait::WalkGait;
use crate::physics::{EarthPhysics, PlanetaryPhysics, TunedPhysics};
use crate::server_physics::ServerPhysics;

fn evaluate_leap(physics: &dyn PlanetaryPhysics) -> f32 {
    let half = 8.0;
    let base = Box::new(PlanarFace::in_zone(ClimateZone::Temperate, Vec3::ZERO, half, 16));
    let mountain = Box::new(VoxelMountain::alpine(base, half));
    let world = CubePlanet::new(vec![mountain], Box::new(NullFace));
    let mut flight = physics.surface_flight(1.0);

    let start_pos = world.constrain_to_surface(Vec3::new(0.25, 5.0, 0.85), 1.0);
    let stride = Vec3::new(0.0, 0.0, 0.1);

    flight.jump();
    let mut pos = start_pos;
    for _ in 0..5 {
        pos = flight.advance(1.0 / 60.0, pos, stride, &WalkGait::upright(), Vec3::Y, &world);
    }
    for _ in 0..120 {
        pos = flight.advance(1.0 / 60.0, pos, Vec3::ZERO, &WalkGait::upright(), Vec3::Y, &world);
    }
    pos.y
}

#[test]
fn test_earth_physics_clears_half_meter_shelf() {
    let final_y = evaluate_leap(&EarthPhysics::new());
    assert_eq!(final_y, 9.5);
}

#[test]
fn test_heavy_gravity_fails_to_clear_shelf() {
    let final_y = evaluate_leap(&TunedPhysics::new(35.0));
    assert_eq!(final_y, 9.0);
}

#[test]
fn test_low_gravity_clears_shelf() {
    let final_y = evaluate_leap(&TunedPhysics::new(5.0));
    assert_eq!(final_y, 9.5);
}

#[test]
fn test_argument_physics_parses_separate_value() {
    let args = vec!["--gravity".to_string(), "35".to_string()];
    let physics = ArgumentPhysics::new(args, Box::new(EarthPhysics::new()));
    let final_y = evaluate_leap(&physics);
    assert_eq!(final_y, 9.0);
}

#[test]
fn test_argument_physics_parses_inline_value() {
    let args = vec!["--gravity=35".to_string()];
    let physics = ArgumentPhysics::new(args, Box::new(EarthPhysics::new()));
    let final_y = evaluate_leap(&physics);
    assert_eq!(final_y, 9.0);
}

#[test]
fn test_argument_physics_fallback_when_empty() {
    let args = vec!["--other-flag".to_string()];
    let physics = ArgumentPhysics::new(args, Box::new(EarthPhysics::new()));
    let final_y = evaluate_leap(&physics);
    assert_eq!(final_y, 9.5);
}

#[test]
fn test_environment_physics_fallback_when_vars_unset() {
    let physics = EnvironmentPhysics::new(Box::new(EarthPhysics::new()));
    let final_y = evaluate_leap(&physics);
    assert_eq!(final_y, 9.5);
}

#[test]
fn test_file_physics_reads_custom_configuration() {
    let temp_path = std::env::temp_dir().join("test_server_gravity_elegant.toml");
    fs::write(
        &temp_path,
        "# Test config\n[physics]\ngravity = 35\n",
    )
    .expect("Failed to write temporary test config");

    let fallback = Box::new(EarthPhysics::new());
    let file_physics = FilePhysics::new(temp_path.to_str().unwrap().to_string(), fallback);
    let final_y = evaluate_leap(&file_physics);

    let _ = fs::remove_file(temp_path);
    assert_eq!(final_y, 9.0);
}

#[test]
fn test_file_physics_missing_file_falls_back_to_earth() {
    let fallback = Box::new(EarthPhysics::new());
    let file_physics = FilePhysics::new("non_existent_config_file_98765.toml".to_string(), fallback);
    let final_y = evaluate_leap(&file_physics);
    assert_eq!(final_y, 9.5);
}

#[test]
fn test_server_physics_fallback_chain() {
    let server_physics = ServerPhysics::new(Box::new(EarthPhysics::new()));
    let final_y = evaluate_leap(&server_physics);
    assert_eq!(final_y, 9.5);
}

#[test]
fn test_doorframe_blocks_one_block_narrow_gap_and_low_lintel_but_allows_two_block_wide_four_block_high_doorway() {
    use world::{BlockMold, Face, FacetedReliefFace};

    let mut face = FacetedReliefFace::in_zone(ClimateZone::Temperate, 8.0, 32);
    let (avatar_pos, eye) = (Vec3::new(0.25, 8.0, 0.75), Vec3::new(0.25, 8.92, 0.75));
    // Build left pillar at u = -1 (x in [-0.5, 0.0], z in [-0.5, 0.0]) and right pillar at u = 1 (x in [0.5, 1.0], z in [-0.5, 0.0]) with overhead lintel -> 1-block (0.5m) doorframe at u = 0
    for _ in 0..5 {
        assert!(face.place_molded_block(Vec3::new(-0.25, 8.92, 0.75), Vec3::new(0.0, -0.75, -0.75).normalize(), avatar_pos, BlockMold::cube()));
        assert!(face.place_molded_block(Vec3::new(0.75, 8.92, 0.75), Vec3::new(0.0, -0.75, -0.75).normalize(), avatar_pos, BlockMold::cube()));
    }
    assert!(face.place_molded_block(Vec3::new(0.25, 8.92, -0.25), Vec3::new(-1.0, 1.45, 0.0).normalize(), avatar_pos, BlockMold::cube()));
    let narrow_world = CubePlanet::new(vec![Box::new(face)], Box::new(NullFace));
    let mut flight = EarthPhysics::new().surface_flight(1.0);
    let mut pos = narrow_world.constrain_to_surface(Vec3::new(0.25, 12.0, 0.25), 1.0);
    let base_y = pos.y;
    for step in 0..45 {
        if step == 10 {
            flight.jump();
        }
        pos = flight.advance(1.0 / 60.0, pos, Vec3::new(0.0, 0.0, -0.05), &WalkGait::upright(), Vec3::Y, &narrow_world);
    }
    assert!(
        pos.z > -0.10 && pos.y <= base_y + 0.60,
        "1-block-wide (0.5m) doorway must block walking and jumping without catapulting avatar over lintel, got {pos:?}"
    );

    // Build 2-block-wide doorway (pillars at u = -1 and u = 2, opening at u = 0..1 -> x in [0.0, 1.0]) with 5th-block lintel (2.0m clearance)
    let mut wide_face = FacetedReliefFace::in_zone(ClimateZone::Temperate, 8.0, 32);
    for _ in 0..5 {
        assert!(wide_face.place_molded_block(Vec3::new(-0.25, 8.92, 0.75), Vec3::new(0.0, -0.75, -0.75).normalize(), avatar_pos, BlockMold::cube()));
        assert!(wide_face.place_molded_block(Vec3::new(1.25, 8.92, 0.75), Vec3::new(0.0, -0.75, -0.75).normalize(), avatar_pos, BlockMold::cube()));
    }
    assert!(wide_face.place_molded_block(Vec3::new(0.75, 8.92, -0.25), Vec3::new(-1.0, 1.45, 0.0).normalize(), avatar_pos, BlockMold::cube()));
    let wide_world = CubePlanet::new(vec![Box::new(wide_face)], Box::new(NullFace));
    let mut wide_pos = wide_world.constrain_to_surface(Vec3::new(0.50, 12.0, 0.25), 1.0);
    for _ in 0..30 {
        wide_pos = flight.advance(1.0 / 60.0, wide_pos, Vec3::new(0.0, 0.0, -0.05), &WalkGait::upright(), Vec3::Y, &wide_world);
    }
    assert!(wide_pos.z < -0.50 && (wide_pos.y - base_y).abs() < 0.05, "2-block-wide doorway with 4-block vertical clearance must allow passage, got {wide_pos:?}");
    let _ = eye;
}
