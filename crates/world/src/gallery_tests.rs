use glam::Vec3;
use crate::{BlockGallery, ClimateZone, CubePlanet, NullFace, PlanarFace, PlanetaryWorld};

#[test]
fn test_block_gallery_parallel_lines_exhibition() {
    let half = 16.0;
    let base = Box::new(PlanarFace::in_zone(ClimateZone::Temperate, Vec3::ZERO, half, 32));
    let gallery = Box::new(BlockGallery::parallel_lines(base, half));
    let world = CubePlanet::new(vec![gallery], Box::new(NullFace));

    // Stone Line (u = -3): Stone Cube at v = -8 -> base floor 16.0 + 1.0 (cube) + 1.0 (player) = 18.0
    let on_stone_cube = world.constrain_to_surface(Vec3::new(-2.5, 20.0, -7.5), 1.0);
    assert_eq!(on_stone_cube.y, 18.0);

    // Stone Line (u = -3): Stone Stair at v = -6 (North heading)
    // Lower step at local v = 0.25 (z = -5.75) -> 16.0 + 0.5 + 1.0 = 17.5
    let on_stair_lower = world.constrain_to_surface(Vec3::new(-2.5, 20.0, -5.75), 1.0);
    assert_eq!(on_stair_lower.y, 17.5);
    // Upper step at local v = 0.75 (z = -5.25) -> 16.0 + 1.0 + 1.0 = 18.0
    let on_stair_upper = world.constrain_to_surface(Vec3::new(-2.5, 20.0, -5.25), 1.0);
    assert_eq!(on_stair_upper.y, 18.0);

    // Dirt Line (u = -1): Dirt Slab at v = -7 -> base floor 16.0 + 0.5 (slab) + 1.0 = 17.5
    let on_dirt_slab = world.constrain_to_surface(Vec3::new(-0.5, 20.0, -6.5), 1.0);
    assert_eq!(on_dirt_slab.y, 17.5);

    // Sand Line (u = 1): Sand Wedge at v = -5 rising along +Z -> at local v = 0.5, elevation is 0.5 -> y = 17.5
    let on_sand_wedge = world.constrain_to_surface(Vec3::new(1.5, 20.0, -4.5), 1.0);
    assert!((on_sand_wedge.y - 17.5).abs() < 1e-4);

    // Snow Line (u = 3): Snow Peak at v = -2 -> apex elevation 0.5 at center (3.5, -1.5) -> y = 16.0 + 0.5 + 1.0 = 17.5
    let on_snow_peak = world.constrain_to_surface(Vec3::new(3.5, 20.0, -1.5), 1.0);
    assert_eq!(on_snow_peak.y, 17.5);

    // 1-block spacing aisle between Stone and Dirt at u = -2 (x = -1.5) -> flat ground: base floor is 16.0 + 1.0 = 17.0
    let on_stone_dirt_aisle = world.constrain_to_surface(Vec3::new(-1.5, 20.0, -5.0), 1.0);
    assert_eq!(on_stone_dirt_aisle.y, 17.0);

    // Central aisle at u = 0 (x = 0.5) -> flat ground: base floor is 17.0
    let on_central_aisle = world.constrain_to_surface(Vec3::new(0.5, 20.0, -5.0), 1.0);
    assert_eq!(on_central_aisle.y, 17.0);

    // 1-block spacing aisle between Sand and Snow at u = 2 (x = 2.5) -> flat ground: base floor is 17.0
    let on_sand_snow_aisle = world.constrain_to_surface(Vec3::new(2.5, 20.0, -5.0), 1.0);
    assert_eq!(on_sand_snow_aisle.y, 17.0);
}
