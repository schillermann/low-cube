use glam::Vec3;
use crate::{ClimateZone, CubePlanet, FoldedFace, NullFace, PlanarFace, PlanetaryWorld, VoxelMountain};

#[test]
fn test_mountain_slope_and_summit_elevations() {
    let half = 8.0;
    let base = Box::new(PlanarFace::in_zone(ClimateZone::Temperate, Vec3::ZERO, half, 16));
    let mountain = Box::new(VoxelMountain::alpine(base, half));
    let folded = Box::new(FoldedFace::in_zone(mountain, ClimateZone::Temperate, half));
    let planet = CubePlanet::new(vec![folded], Box::new(NullFace));

    // Flat ground at spawn point (u = 0.0, v = 0.0 -> base floor is 8.0 + 1.0 = 9.0)
    let supported = planet.constrain_to_surface(Vec3::new(0.0, 5.0, 0.0), 1.0);
    assert_eq!(supported, Vec3::new(0.0, 9.0, 0.0));

    // On mountain entrance rock shelf with smooth edges at (u = 0.25, v = 1.25): height = 0.5 -> floor is 8.0 + 0.5 + 1.0 = 9.5m
    let on_entrance_slab = planet.constrain_to_surface(Vec3::new(0.25, 5.0, 1.25), 1.0);
    assert!((on_entrance_slab.y - 9.5).abs() < 1e-4);

    // On mountain 45° slope wedge at (u = 0.75, v = 1.25): Wedge North at base 0.0, height = 0.25 -> floor is 8.0 + 0.25 + 1.0 = 9.25m
    let on_slope = planet.constrain_to_surface(Vec3::new(0.75, 5.0, 1.25), 1.0);
    assert!((on_slope.y - 9.25).abs() < 1e-4);

    // On summit 45° pyramidal peak apex (center u = 0.25, v = 2.25 -> peak at base 1.0 + 0.25 -> elevation +1.25m -> floor is 10.25m)
    let on_summit = planet.constrain_to_surface(Vec3::new(0.25, 5.0, 2.25), 1.0);
    assert!((on_summit.y - 10.25).abs() < 1e-4);

    // Coordinates crossing edge fold seamlessly onto adjacent Front face (+Z)
    let over_edge = planet.constrain_to_surface(Vec3::new(0.0, 9.0, 8.5), 1.0);
    assert_eq!(over_edge, Vec3::new(0.0, 7.5, 9.0));
}
