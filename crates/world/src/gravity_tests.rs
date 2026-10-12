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
