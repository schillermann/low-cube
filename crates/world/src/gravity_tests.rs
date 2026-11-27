use glam::Vec3;
use crate::{BiomedQuadSphere, GravityField};

#[test]
fn test_gravity_orientation_across_faces() {
    let planet = BiomedQuadSphere::of_size(16);

    assert_eq!(planet.surface_up_at(Vec3::new(0.0, 10.0, 0.0)), Vec3::Y);
    assert_eq!(planet.pull_direction_at(Vec3::new(0.0, 10.0, 0.0)), Vec3::NEG_Y);

    assert_eq!(planet.surface_up_at(Vec3::new(10.0, 0.0, 0.0)), Vec3::X);
    assert_eq!(planet.pull_direction_at(Vec3::new(10.0, 0.0, 0.0)), Vec3::NEG_X);

    assert_eq!(planet.surface_up_at(Vec3::new(0.0, 0.0, 10.0)), Vec3::Z);
    assert_eq!(planet.pull_direction_at(Vec3::new(0.0, 0.0, 10.0)), Vec3::NEG_Z);

    assert_eq!(planet.surface_up_at(Vec3::new(0.0, -10.0, 0.0)), Vec3::NEG_Y);
    assert_eq!(planet.pull_direction_at(Vec3::new(0.0, -10.0, 0.0)), Vec3::Y);

    assert_eq!(planet.surface_up_at(Vec3::new(-10.0, 0.0, 0.0)), Vec3::NEG_X);
    assert_eq!(planet.pull_direction_at(Vec3::new(-10.0, 0.0, 0.0)), Vec3::X);

    assert_eq!(planet.surface_up_at(Vec3::new(0.0, 0.0, -10.0)), Vec3::NEG_Z);
    assert_eq!(planet.pull_direction_at(Vec3::new(0.0, 0.0, -10.0)), Vec3::Z);
}

#[test]
fn test_smooth_radial_gravity_across_meridian() {
    let planet = BiomedQuadSphere::of_size(16);
    let a = Vec3::new(0.0, 9.0, 7.9);
    let b = Vec3::new(0.0, 7.9, 9.0);
    assert!((planet.surface_up_at(a) - a.normalize()).length() < 1e-5);
    assert!((planet.surface_up_at(b) - b.normalize()).length() < 1e-5);
    assert!(planet.surface_up_at(a).dot(planet.surface_up_at(b)) > 0.98);
}
