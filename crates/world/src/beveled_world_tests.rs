use glam::Vec3;
use crate::{BeveledWorld, BiomedCubeWorld, GravityField, PlanetaryWorld};

const SIZE: u32 = 16;
const HALF_SIZE: f32 = 8.0;
const MARGIN: f32 = 0.5;

#[test]
fn test_beveled_world_normal_smoothing_in_seam_zone() {
    let base = Box::new(BiomedCubeWorld::of_size(SIZE));
    let world = BeveledWorld::decorating(base, HALF_SIZE, MARGIN);

    // Deep in top face: Y = 9.0, Z = 0.0 -> pure +Y
    let top_normal = world.surface_up_at(Vec3::new(0.0, 9.0, 0.0));
    assert!((top_normal - Vec3::Y).length() < 1e-5);

    // On exact 45° seam edge between Top (+Y) and Front (+Z): Y = 9.0, Z = 9.0
    let bisector = Vec3::new(0.0, 9.0, 9.0);
    let edge_normal = world.surface_up_at(bisector);
    let expected = (Vec3::Y + Vec3::Z).normalize();
    assert!((edge_normal - expected).length() < 1e-5);

    // Pull direction opposes surface up
    let pull = world.pull_direction_at(bisector);
    assert!((pull - (-expected)).length() < 1e-5);

    // Surface constraint delegates to underlying world
    let constrained = world.constrain_to_surface(Vec3::new(0.0, 5.0, 0.0), 1.0);
    assert_eq!(constrained, Vec3::new(0.0, 9.0, 0.0));
}
