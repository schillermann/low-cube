use glam::Vec3;
use crate::{BeveledGravity, BiomedCubeWorld, GravityField, PlanetaryWorld};

const SIZE: u32 = 16;
const HALF_SIZE: f32 = 8.0;
const MARGIN: f32 = 0.5;

#[test]
fn test_beveled_world_normal_smoothing_in_seam_zone() {
    let world = BiomedCubeWorld::beveled(SIZE, MARGIN);
    let gravity = BeveledGravity::decorating(BiomedCubeWorld::barren(SIZE), HALF_SIZE, MARGIN);

    let top_normal = world.surface_up_at(Vec3::new(0.0, 9.0, 0.0));
    assert!((top_normal - Vec3::Y).length() < 1e-5);

    let bisector = Vec3::new(0.0, 9.0, 9.0);
    let edge_normal = world.surface_up_at(bisector);
    let expected = (Vec3::Y + Vec3::Z).normalize();
    assert!((edge_normal - expected).length() < 1e-5);
    assert!((gravity.surface_up_at(bisector) - expected).length() < 1e-5);

    let pull = world.pull_direction_at(bisector);
    assert!((pull - (-expected)).length() < 1e-5);
    assert!((gravity.pull_direction_at(bisector) - (-expected)).length() < 1e-5);

    let constrained = world.constrain_to_surface(Vec3::new(0.0, 5.0, 0.0), 1.0);
    assert_eq!(constrained, Vec3::new(0.0, 9.0, 0.0));
}
