use glam::Vec3;
use crate::{BiomedCubeWorld, CentripetalWorld, GravityField};

#[test]
fn test_centripetal_world_delegates_to_surface_at_ground_level() {
    let base = Box::new(BiomedCubeWorld::of_size(16));
    let centripetal = CentripetalWorld::new(base, 8.0, 10.0);

    // On top face (+Y) at ground level (Y = 8.0), including near edges and corners
    let ground_pos = Vec3::new(0.0, 8.0, 0.0);
    assert_eq!(centripetal.surface_up_at(ground_pos), Vec3::Y);
    assert_eq!(centripetal.pull_direction_at(ground_pos), Vec3::NEG_Y);

    let near_edge = Vec3::new(7.9, 8.0, 0.0);
    assert_eq!(centripetal.surface_up_at(near_edge), Vec3::Y);

    let near_corner = Vec3::new(7.9, 8.0, 7.9);
    assert_eq!(centripetal.surface_up_at(near_corner), Vec3::Y);
}

#[test]
fn test_centripetal_world_blends_to_radial_at_high_altitude() {
    let base = Box::new(BiomedCubeWorld::of_size(16));
    let half_size = 8.0;
    let blend_margin = 10.0;
    let centripetal = CentripetalWorld::new(base, half_size, blend_margin);

    // At high diagonal altitude above the corner (altitude >= blend_margin)
    let diagonal_space = Vec3::new(20.0, 20.0, 20.0);
    let up = centripetal.surface_up_at(diagonal_space);
    let expected_radial = diagonal_space.normalize();

    assert!((up - expected_radial).length() < 1e-4);
    assert_eq!(centripetal.pull_direction_at(diagonal_space), -up);
}

#[test]
fn test_centripetal_world_smooth_interpolation_in_transition_zone() {
    let base = Box::new(BiomedCubeWorld::of_size(16));
    let half_size = 8.0;
    let blend_margin = 10.0;
    let centripetal = CentripetalWorld::new(base, half_size, blend_margin);

    // Mid-air above corner (x=10.0, y=10.0, z=0.0): length is ~14.14, altitude is ~6.14
    let mid_air = Vec3::new(10.0, 10.0, 0.0);
    let up = centripetal.surface_up_at(mid_air);

    // Up vector must be normalized
    assert!((up.length() - 1.0).abs() < 1e-5);
    // X and Y components should be positive and balanced
    assert!(up.x > 0.0);
    assert!(up.y > 0.0);
}
