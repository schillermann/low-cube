use glam::Vec3;
use world::{
    Block, CubeFace, Face, GravityField, MeshTarget, PlanarFace, PlanetaryWorld,
    StoneBlock, SurfaceFrame, VoxelCluster, WedgeHeading,
};
use crate::flight::{GravitationalFlight, SurfaceFlight};
use crate::gait::{SneakGait, WalkGait};

struct ObstacleWorld {
    cluster: VoxelCluster,
}

impl ObstacleWorld {
    fn with_block(block: Box<dyn Block>) -> Self {
        let face = Box::new(PlanarFace::on_cube(CubeFace::Top, Vec3::ONE, 8.0, 16));
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

    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, &WalkGait::upright(), Vec3::Y, &world);

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

    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, &WalkGait::upright(), Vec3::Y, &world);

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

    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, &WalkGait::upright(), Vec3::Y, &world);

    // Forward motion into the wall is blocked, but lateral dodging motion proceeds
    assert_eq!(next_pos.x, start_pos.x);
    assert!(next_pos.z > start_pos.z);
    assert_eq!(next_pos.y, start_pos.y);
}

#[test]
fn test_walk_off_elevated_block_falls_when_not_sneaking() {
    let world = ObstacleWorld::with_block(Box::new(StoneBlock::cube(1, 0, 0.0)));
    let mut flight = SurfaceFlight::terrestrial(1.0);
    // On the 1.0m cube at x = 1.95, z = 0.5 (edge is at x = 2.0, elevation 10.0)
    let start_pos = world.constrain_to_surface(Vec3::new(1.95, 5.0, 0.5), 1.0);
    assert_eq!(start_pos.y, 10.0);
    let stride = Vec3::new(0.1, 0.0, 0.0);

    // Walking normally without sneaking steps over the edge into open air
    let step_pos = flight.advance(1.0 / 60.0, start_pos, stride, &WalkGait::upright(), Vec3::Y, &world);
    assert!(step_pos.x > 2.0);

    // Subsequent flight steps experience gravitational fall down to ground (y = 9.0)
    let mut pos = step_pos;
    for _ in 0..50 {
        pos = flight.advance(1.0 / 60.0, pos, Vec3::ZERO, &WalkGait::upright(), Vec3::Y, &world);
    }
    assert_eq!(pos.y, 9.0);
}

#[test]
fn test_sneak_on_elevated_block_prevents_falling() {
    let world = ObstacleWorld::with_block(Box::new(StoneBlock::cube(1, 0, 0.0)));
    let mut flight = SurfaceFlight::terrestrial(1.0);
    // On the 1.0m cube at x = 1.95, z = 0.5 (edge is at x = 2.0, elevation 10.0)
    let start_pos = world.constrain_to_surface(Vec3::new(1.95, 5.0, 0.5), 1.0);
    assert_eq!(start_pos.y, 10.0);
    let stride = Vec3::new(0.1, 0.0, 0.0);

    // Sneaking gait prevents falling off the edge of the elevated block
    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, &SneakGait::crouching(), Vec3::Y, &world);

    assert_eq!(next_pos.x, start_pos.x);
    assert_eq!(next_pos.y, start_pos.y);
    assert_eq!(next_pos.z, start_pos.z);
}

#[test]
fn test_sneak_on_elevated_block_diagonal_slides_along_edge() {
    let world = ObstacleWorld::with_block(Box::new(StoneBlock::cube(1, 0, 0.0)));
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let start_pos = world.constrain_to_surface(Vec3::new(1.95, 5.0, 0.5), 1.0);
    let stride = Vec3::new(0.1, 0.0, 0.1);

    let next_pos = flight.advance(1.0 / 60.0, start_pos, stride, &SneakGait::crouching(), Vec3::Y, &world);

    // Forward motion over edge is blocked, lateral motion along edge proceeds without falling
    assert_eq!(next_pos.x, start_pos.x);
    assert!(next_pos.z > start_pos.z);
    assert_eq!(next_pos.y, start_pos.y);
}

#[test]
fn test_sneak_paces_stride_distance() {
    use crate::controls::LocomotionTarget;
    use crate::order::LocomotionOrder;

    let mut walk_order = LocomotionOrder::idle();
    walk_order.travel(Vec3::X);
    let walk_stride = walk_order.surface_stride(Vec3::Y, 1.0);

    let mut sneak_order = LocomotionOrder::idle();
    sneak_order.travel(Vec3::X);
    sneak_order.sneak();
    let sneak_stride = sneak_order.surface_stride(Vec3::Y, 1.0);

    assert_eq!(walk_stride.x, 1.0);
    assert!((sneak_stride.x - 0.35).abs() < 1e-5);
}
