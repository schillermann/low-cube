use glam::Vec3;
use world::{GravityField, MeshTarget, PlanetaryWorld};
use crate::flight::{GravitationalFlight, SurfaceFlight};
use crate::gait::WalkGait;

struct FlatWorld;

impl GravityField for FlatWorld {
    fn pull_direction_at(&self, _position: Vec3) -> Vec3 {
        Vec3::NEG_Y
    }
    fn surface_up_at(&self, _position: Vec3) -> Vec3 {
        Vec3::Y
    }
}

impl PlanetaryWorld for FlatWorld {
    fn emit_surface(&self, _target: &mut dyn MeshTarget) {}
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        Vec3::new(position.x, player_height, position.z)
    }
}

struct CliffWorld {
    cliff_z: f32,
    base_y: f32,
    cliff_y: f32,
}

impl CliffWorld {
    fn new(cliff_z: f32, base_y: f32, cliff_y: f32) -> Self {
        Self {
            cliff_z,
            base_y,
            cliff_y,
        }
    }
}

impl GravityField for CliffWorld {
    fn pull_direction_at(&self, _position: Vec3) -> Vec3 {
        Vec3::NEG_Y
    }
    fn surface_up_at(&self, _position: Vec3) -> Vec3 {
        Vec3::Y
    }
}

impl PlanetaryWorld for CliffWorld {
    fn emit_surface(&self, _target: &mut dyn MeshTarget) {}
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        let y = if position.z >= self.cliff_z {
            self.cliff_y
        } else {
            self.base_y
        };
        Vec3::new(position.x, y + player_height, position.z)
    }
}

#[test]
fn test_grounded_remains_on_surface() {
    let world = FlatWorld;
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let pos = Vec3::new(0.0, 1.0, 0.0);

    let next = flight.advance(1.0 / 60.0, pos, Vec3::ZERO, &WalkGait::new(), Vec3::Y, &world);
    assert_eq!(next, pos);
}

#[test]
fn test_jump_and_fall_cycle() {
    let world = FlatWorld;
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let initial_pos = Vec3::new(0.0, 1.0, 0.0);

    flight.jump();

    let step_pos = flight.advance(1.0 / 60.0, initial_pos, Vec3::ZERO, &WalkGait::new(), Vec3::Y, &world);
    assert!(step_pos.y > initial_pos.y);

    let mut current_pos = step_pos;
    for _ in 0..100 {
        current_pos = flight.advance(1.0 / 60.0, current_pos, Vec3::ZERO, &WalkGait::new(), Vec3::Y, &world);
    }

    assert_eq!(current_pos, initial_pos);
}

#[test]
fn test_no_double_jump_in_mid_air() {
    let world = FlatWorld;
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let initial_pos = Vec3::new(0.0, 1.0, 0.0);

    flight.jump();
    let step_pos = flight.advance(1.0 / 60.0, initial_pos, Vec3::ZERO, &WalkGait::new(), Vec3::Y, &world);

    flight.jump();
    let next_pos = flight.advance(1.0 / 60.0, step_pos, Vec3::ZERO, &WalkGait::new(), Vec3::Y, &world);

    let expected_v = 3.2 - 9.81 * (2.0 / 60.0);
    assert!((next_pos.y - step_pos.y - expected_v * (1.0 / 60.0)).abs() < 1e-4);
}

#[test]
fn test_jump_against_steep_cliff_blocked_mid_air() {
    let world = CliffWorld::new(1.0, 10.0, 12.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let start_pos = Vec3::new(0.0, 11.0, 0.95);
    let stride = Vec3::new(0.0, 0.0, 0.1);

    flight.jump();
    let mut pos = start_pos;
    for _ in 0..60 {
        pos = flight.advance(1.0 / 60.0, pos, stride, &WalkGait::new(), Vec3::Y, &world);
    }

    assert!(pos.z < 1.0);
    assert_eq!(pos.y, 11.0);
}
