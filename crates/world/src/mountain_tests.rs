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

    // On mountain entrance rock slab with smooth edges at (u = 0.5, v = 2.5): Slab at base 0.0, height = 0.5 -> floor is 8.0 + 0.5 + 1.0 = 9.5m
    let on_entrance_slab = planet.constrain_to_surface(Vec3::new(0.5, 5.0, 2.5), 1.0);
    assert!((on_entrance_slab.y - 9.5).abs() < 1e-4);

    // On mountain 45° slope wedge at (u = 1.5, v = 2.5): Wedge North at base 0.0, height = 0.5 -> floor is 8.0 + 0.5 + 1.0 = 9.5m
    let on_slope = planet.constrain_to_surface(Vec3::new(1.5, 5.0, 2.5), 1.0);
    assert!((on_slope.y - 9.5).abs() < 1e-4);

    // On summit 45° pyramidal peak apex (center u = 0.5, v = 4.5 -> peak at base 2.0 + 0.5 -> elevation +2.5m -> floor is 11.5m)
    let on_summit = planet.constrain_to_surface(Vec3::new(0.5, 5.0, 4.5), 1.0);
    assert!((on_summit.y - 11.5).abs() < 1e-4);

    // Coordinates crossing edge fold seamlessly onto adjacent Front face (+Z)
    let over_edge = planet.constrain_to_surface(Vec3::new(0.0, 9.0, 8.5), 1.0);
    assert_eq!(over_edge, Vec3::new(0.0, 7.5, 9.0));
}
