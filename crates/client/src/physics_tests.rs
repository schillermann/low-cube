use std::fs;
use glam::Vec3;
use world::{BiomedCubeWorld, PlanetaryWorld};
use crate::argument_physics::ArgumentPhysics;
use crate::environment_physics::EnvironmentPhysics;
use crate::file_physics::FilePhysics;
use crate::gait::WalkGait;
use crate::physics::{EarthPhysics, PlanetaryPhysics, TunedPhysics};
use crate::server_physics::ServerPhysics;

fn evaluate_leap(physics: &dyn PlanetaryPhysics) -> f32 {
    let world = BiomedCubeWorld::of_size(16);
    let mut flight = physics.surface_flight(1.0);

    let start_pos = world.constrain_to_surface(Vec3::new(0.5, 5.0, 1.95), 1.0);
    let stride = Vec3::new(0.0, 0.0, 0.1);

    flight.jump();
    let mut pos = start_pos;
    for _ in 0..5 {
        pos = flight.advance(1.0 / 60.0, pos, stride, &WalkGait::upright(), Vec3::Y, &world);
    }
    for _ in 0..60 {
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
fn test_low_jump_height_fails_to_clear_shelf() {
    let final_y = evaluate_leap(&TunedPhysics::new(0.2, 9.81));
    assert_eq!(final_y, 9.0);
}

#[test]
fn test_high_jump_height_clears_shelf() {
    let final_y = evaluate_leap(&TunedPhysics::new(1.0, 9.81));
    assert_eq!(final_y, 9.5);
}

#[test]
fn test_argument_physics_parses_separate_value() {
    let args = vec!["--jump-height".to_string(), "0.2".to_string()];
    let physics = ArgumentPhysics::new(args, Box::new(EarthPhysics::new()));
    let final_y = evaluate_leap(&physics);
    assert_eq!(final_y, 9.0);
}

#[test]
fn test_argument_physics_parses_inline_value() {
    let args = vec!["--jump-height=0.2".to_string()];
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
    let temp_path = std::env::temp_dir().join("test_server_jump_elegant.toml");
    fs::write(
        &temp_path,
        "# Test config\n[physics]\njump_height = 0.2\ngravity = 9.81\n",
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
