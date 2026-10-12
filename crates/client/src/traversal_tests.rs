use glam::Vec3;
use world::{
    Block, Face, GravityField, MeshTarget, PlanarFace, PlanetaryWorld,
    StoneBlock, SurfaceFrame, VoxelCluster, WedgeHeading,
};
use crate::flight::{GravitationalFlight, SurfaceFlight};

struct ObstacleWorld {
    cluster: VoxelCluster,
}

impl ObstacleWorld {
    fn with_block(block: Box<dyn Block>) -> Self {
        let face = Box::new(PlanarFace::new(Vec3::Y, Vec3::X, Vec3::Z, Vec3::ONE, 8.0, 16));
        let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 8.0);
        let cluster = VoxelCluster::new(face, frame, vec![block]);
        Self { cluster }
    }
}

impl GravityField for ObstacleWorld {
    fn pull_direction_at(&self, _position: Vec3) -> Vec3 {
        Vec3::NEG_Y
    }
    fn surface_up_at(&self, _position: Vec3) -> Vec3 {
        Vec3::Y
    }
}

impl PlanetaryWorld for ObstacleWorld {
    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.cluster.emit_surface(target);
    }
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.cluster.constrain(position, player_height)
    }
}

#[test]
fn test_walk_into_solid_cube_blocked() {
    let world = ObstacleWorld::with_block(Box::new(StoneBlock::cube(1, 0, 0.0)));
    let mut flight = SurfaceFlight::terrestrial(1.0);
    // At x = 0.95, z = 0.5: just in front of the SolidCube at grid (1, 0)
    let start_pos = world.constrain_to_surface(Vec3::new(0.95, 5.0, 0.5), 1.0);
    let stride = Vec3::new(0.1, 0.0, 0.0);

    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, Vec3::Y, &world);

    // Blocked by the sheer 1.0m vertical cube wall
    assert_eq!(next_pos.x, start_pos.x);
    assert_eq!(next_pos.y, start_pos.y);
}

#[test]
fn test_walk_up_ramp_wedge_smoothly() {
    let world = ObstacleWorld::with_block(Box::new(StoneBlock::wedge(0, 0, 0.0, WedgeHeading::North)));
    let mut flight = SurfaceFlight::terrestrial(1.0);
    // At x = 0.5, z = 0.1: on the 45° RampWedge rising along +Z
    let start_pos = world.constrain_to_surface(Vec3::new(0.5, 5.0, 0.1), 1.0);
    let stride = Vec3::new(0.0, 0.0, 0.1);

    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, Vec3::Y, &world);

    // Continuous on-foot ascent without jumping
    assert!(next_pos.z > start_pos.z);
    assert!(next_pos.y > start_pos.y);
}

#[test]
fn test_walk_into_wall_while_strafing_preserves_lateral_motion() {
    let world = ObstacleWorld::with_block(Box::new(StoneBlock::cube(1, 0, 0.0)));
    let mut flight = SurfaceFlight::terrestrial(1.0);
    // At x = 0.95, z = 0.5: in front of the SolidCube at grid (1, 0)
    let start_pos = world.constrain_to_surface(Vec3::new(0.95, 5.0, 0.5), 1.0);
    // Running against the wall (+X) while dodging/strafing sideways (+Z)
    let stride = Vec3::new(0.1, 0.0, 0.1);

    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, Vec3::Y, &world);

    // Forward motion into the wall is blocked, but lateral dodging motion proceeds
    assert_eq!(next_pos.x, start_pos.x);
    assert!(next_pos.z > start_pos.z);
    assert_eq!(next_pos.y, start_pos.y);
}
