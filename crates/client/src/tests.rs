use glam::Vec3;
use world::BiomedCubeWorld;
use crate::flight::{GravitationalFlight, SurfaceFlight};

#[test]
fn test_grounded_remains_on_surface() {
    let world = BiomedCubeWorld::of_size(16);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let pos = Vec3::new(0.0, 9.0, 0.0);

    let next = flight.advance(1.0 / 60.0, pos, Vec3::Y, &world);
    assert_eq!(next, pos);
}

#[test]
fn test_jump_and_fall_cycle() {
    let world = BiomedCubeWorld::of_size(16);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let initial_pos = Vec3::new(0.0, 9.0, 0.0);

    flight.jump();

    let step_pos = flight.advance(1.0 / 60.0, initial_pos, Vec3::Y, &world);
    assert!(step_pos.y > initial_pos.y);

    let mut current_pos = step_pos;
    for _ in 0..100 {
        current_pos = flight.advance(1.0 / 60.0, current_pos, Vec3::Y, &world);
    }

    assert_eq!(current_pos, initial_pos);
}

#[test]
fn test_no_double_jump_in_mid_air() {
    let world = BiomedCubeWorld::of_size(16);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let initial_pos = Vec3::new(0.0, 9.0, 0.0);

    flight.jump();
    let step_pos = flight.advance(1.0 / 60.0, initial_pos, Vec3::Y, &world);

    flight.jump();
    let next_pos = flight.advance(1.0 / 60.0, step_pos, Vec3::Y, &world);

    let expected_v = 3.2 - 9.81 * (2.0 / 60.0);
    assert!((next_pos.y - step_pos.y - expected_v * (1.0 / 60.0)).abs() < 1e-4);
}
