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

    let next = flight.advance(1.0 / 60.0, pos, Vec3::ZERO, &WalkGait::upright(), Vec3::Y, &world);
    assert_eq!(next, pos);
}

#[test]
fn test_jump_and_fall_cycle() {
    let world = FlatWorld;
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let initial_pos = Vec3::new(0.0, 1.0, 0.0);

    flight.jump();

    let step_pos = flight.advance(1.0 / 60.0, initial_pos, Vec3::ZERO, &WalkGait::upright(), Vec3::Y, &world);
    assert!(step_pos.y > initial_pos.y);

    let mut current_pos = step_pos;
    for _ in 0..100 {
        current_pos = flight.advance(1.0 / 60.0, current_pos, Vec3::ZERO, &WalkGait::upright(), Vec3::Y, &world);
    }

    assert_eq!(current_pos, initial_pos);
}

#[test]
fn test_no_double_jump_in_mid_air() {
    let world = FlatWorld;
    let mut flight = SurfaceFlight::terrestrial(1.0);
    let initial_pos = Vec3::new(0.0, 1.0, 0.0);

    flight.jump();
    let step_pos = flight.advance(1.0 / 60.0, initial_pos, Vec3::ZERO, &WalkGait::upright(), Vec3::Y, &world);

    flight.jump();
    let next_pos = flight.advance(1.0 / 60.0, step_pos, Vec3::ZERO, &WalkGait::upright(), Vec3::Y, &world);

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
        pos = flight.advance(1.0 / 60.0, pos, stride, &WalkGait::upright(), Vec3::Y, &world);
    }

    assert!(pos.z < 1.0);
    assert_eq!(pos.y, 11.0);
}

#[test]
fn test_pyramid_sector_gravity_indestructible_core_floor_and_shaft_width() {
    use world::BiomedCubeWorld;
    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let mut flight = SurfaceFlight::terrestrial(1.0);

    // 1. Pyramid-sector gravity holds deep inside the planet near cube edges
    assert_eq!(world.surface_up_at(Vec3::new(3.9, 4.1, 0.0)), Vec3::Y);
    assert_eq!(world.surface_up_at(Vec3::new(4.1, 3.9, 0.0)), Vec3::X);

    // 2. Dig a shaft at (0.25, y, 0.25) through the mantle and 4-layer crust until it stops on the solid core floor
    for step in 0..8 {
        let floor_y = world.constrain_to_surface(Vec3::new(0.25, 17.0, 0.25), 0.0).y;
        if step == 4 {
            assert!(
                (floor_y - 14.0).abs() < 1e-4 && !world.lethal_at(Vec3::new(0.25, floor_y + 1.0, 0.25)),
                "After 4 blocks of digging avatar must still be in solid ground above the core floor"
            );
        }
        assert!(world.excavate_surface(Vec3::new(0.25, floor_y + 1.0, 0.25), Vec3::NEG_Y), "Layer {step} must excavate");
    }
    let core_floor_y = world.constrain_to_surface(Vec3::new(0.25, 17.0, 0.25), 0.0).y;
    assert!((core_floor_y - 12.0).abs() < 1e-4, "Excavated shaft must stop on the solid core floor at y = 12.0, got {core_floor_y}");
    assert!(!world.excavate_surface(Vec3::new(0.25, 13.0, 0.25), Vec3::NEG_Y), "Solid core floor must not be diggable");

    // 3. Touching the lethal core floor (y = 13.0, feet = 12.0) kills the avatar in place instead of teleporting to the surface
    let core_contact = Vec3::new(0.25, 13.0, 0.25);
    assert!(world.lethal_at(core_contact));
    let perished_pos = flight.advance(1.0 / 60.0, core_contact, Vec3::ZERO, &WalkGait::upright(), Vec3::Y, &world);
    assert_eq!(perished_pos, core_contact, "Core floor contact must arrest the dead avatar at the floor without teleporting to the surface");

    // 4. Digging only 1 block wide vertically down does NOT let the wider upper body fall in
    let mut world64 = BiomedCubeWorld::beveled(64, 1.0);
    let surface_eye_y = world64.constrain_to_surface(Vec3::new(0.25, 35.0, 0.25), 1.0).y;
    for _ in 0..15 {
        let floor_y = world64.constrain_to_surface(Vec3::new(0.25, 33.0, 0.25), 0.0).y;
        assert!(world64.excavate_surface(Vec3::new(0.25, floor_y + 1.0, 0.25), Vec3::NEG_Y));
    }
    assert!(
        (world64.constrain_to_surface(Vec3::new(0.25, 35.0, 0.25), 1.0).y - surface_eye_y).abs() < 1e-4,
        "1-block-wide vertical hole must not let the wider upper body fall in"
    );
}
