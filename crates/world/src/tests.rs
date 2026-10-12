use glam::Vec3;
use crate::{BiomedCubeWorld, GravityField};

#[test]
fn test_gravity_orientation_across_faces() {
    let planet = BiomedCubeWorld::of_size(16);

    // Top face (+Y)
    assert_eq!(planet.surface_up_at(Vec3::new(0.0, 10.0, 0.0)), Vec3::Y);
    assert_eq!(planet.pull_direction_at(Vec3::new(0.0, 10.0, 0.0)), Vec3::NEG_Y);

    // Right face (+X)
    assert_eq!(planet.surface_up_at(Vec3::new(10.0, 0.0, 0.0)), Vec3::X);
    assert_eq!(planet.pull_direction_at(Vec3::new(10.0, 0.0, 0.0)), Vec3::NEG_X);

    // Front face (+Z)
    assert_eq!(planet.surface_up_at(Vec3::new(0.0, 0.0, 10.0)), Vec3::Z);
    assert_eq!(planet.pull_direction_at(Vec3::new(0.0, 0.0, 10.0)), Vec3::NEG_Z);

    // Bottom face (-Y)
    assert_eq!(planet.surface_up_at(Vec3::new(0.0, -10.0, 0.0)), Vec3::NEG_Y);
    assert_eq!(planet.pull_direction_at(Vec3::new(0.0, -10.0, 0.0)), Vec3::Y);

    // Left face (-X)
    assert_eq!(planet.surface_up_at(Vec3::new(-10.0, 0.0, 0.0)), Vec3::NEG_X);
    assert_eq!(planet.pull_direction_at(Vec3::new(-10.0, 0.0, 0.0)), Vec3::X);

    // Back face (-Z)
    assert_eq!(planet.surface_up_at(Vec3::new(0.0, 0.0, -10.0)), Vec3::NEG_Z);
    assert_eq!(planet.pull_direction_at(Vec3::new(0.0, 0.0, -10.0)), Vec3::Z);
}

#[test]
fn test_edge_transition_switch() {
    let planet = BiomedCubeWorld::of_size(16);

    // On top face near edge: Y = 9.0, Z = 7.9 -> Dominant is Y
    assert_eq!(planet.surface_up_at(Vec3::new(0.0, 9.0, 7.9)), Vec3::Y);

    // Over the edge on front face: Y = 7.9, Z = 9.0 -> Dominant is Z
    assert_eq!(planet.surface_up_at(Vec3::new(0.0, 7.9, 9.0)), Vec3::Z);
}

#[test]
fn test_surface_floor_constraint() {
    use crate::PlanetaryWorld;
    let planet = BiomedCubeWorld::of_size(16);

    // Flat ground at spawn point (u = 0.0, v = 0.0 -> base floor is 8.0 + 1.0 = 9.0)
    let supported = planet.constrain_to_surface(Vec3::new(0.0, 5.0, 0.0), 1.0);
    assert_eq!(supported, Vec3::new(0.0, 9.0, 0.0));

    // On mountain 45° slope (u = 0.0, v = 2.0 -> elevation +1.3333m -> floor is 10.3333m)
    let on_slope = planet.constrain_to_surface(Vec3::new(0.0, 5.0, 2.0), 1.0);
    assert!((on_slope.y - (9.0 + 4.0 / 3.0)).abs() < 1e-4);

    // On summit rock shelf (center u = 0.0, v = 4.0 -> elevation +2.5m -> floor is 11.5m)
    let on_summit = planet.constrain_to_surface(Vec3::new(0.0, 5.0, 4.0), 1.0);
    assert!((on_summit.y - 11.5).abs() < 1e-4);

    // Tangent coordinate unconstrained past edge
    let over_edge = planet.constrain_to_surface(Vec3::new(0.0, 9.0, 8.5), 1.0);
    assert_eq!(over_edge.z, 8.5);
}
