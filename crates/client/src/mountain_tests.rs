use glam::Vec3;
use world::{ClimateZone, CubePlanet, NullFace, PlanarFace, PlanetaryWorld, VoxelMountain};
use crate::flight::{GravitationalFlight, SurfaceFlight};
use crate::gait::WalkGait;

fn mountain_world() -> Box<dyn PlanetaryWorld> {
    let half = 8.0;
    let base = Box::new(PlanarFace::in_zone(ClimateZone::Temperate, Vec3::ZERO, half, 16));
    let mountain = Box::new(VoxelMountain::alpine(base, half));
    Box::new(CubePlanet::new(vec![mountain], Box::new(NullFace)))
}

#[test]
fn test_walk_up_mountain_continuous_slope() {
    let world = mountain_world();
    let mut flight = SurfaceFlight::terrestrial(1.0);
    // On Tier 0 Wedge (grid u=1, v=2) at x = 1.5, z = 2.2
    let start_pos = world.constrain_to_surface(Vec3::new(1.5, 5.0, 2.2), 1.0);
    let stride = Vec3::new(0.0, 0.0, 0.1);

    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, &WalkGait::upright(), Vec3::Y, world.as_ref());

    assert!(next_pos.z > start_pos.z);
    assert!(next_pos.y > start_pos.y);
}

#[test]
fn test_walk_blocked_by_mountain_entrance_shelf() {
    let world = mountain_world();
    let mut flight = SurfaceFlight::terrestrial(1.0);
    // In front of the 0.5m entrance rock slab with smooth edges at z = 1.95 (shelf at z = 2.0)
    let start_pos = world.constrain_to_surface(Vec3::new(0.5, 5.0, 1.95), 1.0);
    let stride = Vec3::new(0.0, 0.0, 0.1);

    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, &WalkGait::upright(), Vec3::Y, world.as_ref());

    // Stride is blocked by the 0.5m vertical entrance rock shelf
    assert_eq!(next_pos.z, start_pos.z);
    assert_eq!(next_pos.y, start_pos.y);
}

#[test]
fn test_jump_clears_mountain_entrance_shelf() {
    let world = mountain_world();
    let mut flight = SurfaceFlight::terrestrial(1.0);
    // In front of the entrance rock shelf at z = 1.95
    let start_pos = world.constrain_to_surface(Vec3::new(0.5, 5.0, 1.95), 1.0);
    let stride = Vec3::new(0.0, 0.0, 0.1);

    flight.jump();
    let mut pos = start_pos;
    for _ in 0..5 {
        pos = flight.advance(1.0 / 60.0, pos, stride, &WalkGait::upright(), Vec3::Y, world.as_ref());
    }
    for _ in 0..25 {
        pos = flight.advance(1.0 / 60.0, pos, Vec3::ZERO, &WalkGait::upright(), Vec3::Y, world.as_ref());
    }

    // Successfully ascended onto the entrance rock slab (grid v=2, elevation 0.5m -> y = 9.5)
    assert!(pos.z > 2.0 && pos.z < 3.0);
    assert_eq!(pos.y, 9.5);
}
