use glam::Vec3;
use world::{GravityField, MeshTarget, PlanetaryWorld};
use super::canopy::CanopyBraking;
use super::gravity::{AsymmetricGravity, FlightGravity, UniformGravity};
use super::slope::{SkiLaunch, SlopeSurfing};
use crate::flight::GravitationalFlight;

struct FlatPlane;

impl GravityField for FlatPlane {
    fn pull_direction_at(&self, _position: Vec3) -> Vec3 {
        Vec3::NEG_Y
    }
    fn surface_up_at(&self, _position: Vec3) -> Vec3 {
        Vec3::Y
    }
}

impl PlanetaryWorld for FlatPlane {
    fn emit_surface(&self, _target: &mut dyn MeshTarget) {}
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        Vec3::new(position.x, player_height, position.z)
    }
}

struct RampWorld {
    slope: f32,
}

impl GravityField for RampWorld {
    fn pull_direction_at(&self, _position: Vec3) -> Vec3 {
        Vec3::NEG_Y
    }
    fn surface_up_at(&self, _position: Vec3) -> Vec3 {
        Vec3::Y
    }
}

impl PlanetaryWorld for RampWorld {
    fn emit_surface(&self, _target: &mut dyn MeshTarget) {}
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        Vec3::new(position.x, position.z * self.slope + player_height, position.z)
    }
}

#[test]
fn test_uniform_gravity_constant_delta() {
    let uniform = UniformGravity::new(10.0);
    assert_eq!(uniform.delta_velocity(5.0, 0.1), 1.0);
    assert_eq!(uniform.delta_velocity(0.0, 0.1), 1.0);
    assert_eq!(uniform.delta_velocity(-5.0, 0.1), 1.0);
}

#[test]
fn test_asymmetric_gravity_ascent_apex_and_descent() {
    let asymmetric = AsymmetricGravity::terrestrial(10.0);
    // Ascent (> 0.5): 1.0 ratio -> 1.0
    let ascent = asymmetric.delta_velocity(3.0, 0.1);
    assert!((ascent - 1.0).abs() < 1e-5, "Ascent gravity must be 1.0x, got {}", ascent);

    // Apex (|v| <= 0.35): 0.75 ratio -> 0.75 (hang time float)
    let apex = asymmetric.delta_velocity(0.1, 0.1);
    assert!((apex - 0.75).abs() < 1e-5, "Apex gravity must float at 0.75x, got {}", apex);

    // Descent (< -0.35): 1.50 ratio -> 1.50 (snappy fall)
    let descent = asymmetric.delta_velocity(-3.0, 0.1);
    assert!((descent - 1.50).abs() < 1e-5, "Descent gravity must snap at 1.50x, got {}", descent);
}

#[test]
fn test_canopy_braking_decelerates_heavy_fall_and_permits_ascent() {
    let braking = CanopyBraking::calibrated();

    // Upward jump through canopy foliage must not be braked downward
    let ascent = braking.brake_fall(3.0, 0.9, 0.1);
    assert_eq!(ascent, 3.0, "Upward velocity must not be braked by canopy");

    // Heavy fall must decelerate rapidly towards terminal descent speed
    let falling = -12.0;
    let braked = braking.brake_fall(falling, 0.9, 0.1);
    assert!(braked > falling, "Foliage drag must brake downward fall");
    assert!(braked <= -1.5, "Fall speed must approach terminal speed");

    // Zero drag outside crown
    let unbraked = braking.brake_fall(-12.0, 0.0, 0.1);
    assert_eq!(unbraked, -12.0, "Outside crown drag must not brake fall");
}

#[test]
fn test_ski_launch_evaluates_ramp_incline() {
    let flat = FlatPlane;
    let ramp = RampWorld { slope: 1.0 }; // 45 degree ramp (rise = 0.1 for stride 0.1)
    let ski = SkiLaunch::calibrated();

    let stride = Vec3::new(0.0, 0.0, 0.1);
    let surface_up = Vec3::Y;
    let dt = 1.0 / 60.0;

    // Flat ground: standard launch without boost
    let (flat_speed, flat_momentum) = ski.evaluate_launch(3.2, stride, surface_up, Vec3::new(0.0, 1.0, 0.0), dt, 1.0, &flat);
    assert_eq!(flat_speed, 3.2);
    assert_eq!(flat_momentum, Vec3::ZERO);

    // Continuous 45 deg ramp: boosted launch velocity and forward momentum
    let (ramp_speed, ramp_momentum) = ski.evaluate_launch(3.2, stride, surface_up, Vec3::new(0.0, 2.0, 1.0), dt, 1.0, &ramp);
    assert!(ramp_speed > 3.2, "Ramp incline must boost vertical launch speed");
    assert!(ramp_momentum.z > 0.0, "Ramp incline must catapult forward momentum");
}

#[test]
fn test_slope_surfing_deflects_downhill_impact_and_friction_damps() {
    let down_ramp = RampWorld { slope: -1.0 }; // Downward slope
    let surfing = SlopeSurfing::calibrated();

    let stride = Vec3::new(0.0, 0.0, 0.1);
    let surface_up = Vec3::Y;

    // Touchdown on downward slope converts impact into forward slide momentum
    let momentum = surfing.evaluate_touchdown(4.0, stride, surface_up, Vec3::new(0.0, 1.0, 2.0), 1.0, &down_ramp);
    assert!(momentum.z > 0.0, "Touchdown on downslope must yield forward slide momentum");

    // Momentum damps under surface friction
    let damped = surfing.damp_momentum(momentum, 0.1);
    assert!(damped.length() < momentum.length(), "Friction must damp slide momentum");
}

#[test]
fn test_jump_while_running_on_slope_never_penetrates_ground() {
    let ramp = RampWorld { slope: 1.0 };
    let mut flight = crate::flight::SurfaceFlight::terrestrial(1.0);
    let mut pos = ramp.constrain_to_surface(Vec3::new(0.0, 5.0, 1.0), 1.0);
    flight.jump();
    let stride = Vec3::new(0.0, 0.0, 0.1);
    for frame in 0..60 {
        pos = flight.advance(1.0 / 60.0, pos, stride, &crate::gait::WalkGait::upright(), Vec3::Y, &ramp);
        let ground = ramp.constrain_to_surface(pos, 1.0);
        assert!(
            pos.y >= ground.y - 1e-4,
            "Frame {}: Avatar ({}) must never penetrate below ramp surface ({})",
            frame,
            pos.y,
            ground.y
        );
    }
}

#[test]
fn test_incline_pace_downhill_faster_than_uphill() {
    let pace = super::incline::InclinePace::calibrated();
    let flat = FlatPlane;
    let up_ramp = RampWorld { slope: 0.5 };
    let down_ramp = RampWorld { slope: -0.5 };
    let stride = Vec3::new(0.0, 0.0, 0.1);
    let pos = Vec3::new(0.0, 1.0, 0.0);
    let surface_up = Vec3::Y;

    let flat_stride = pace.pace_stride(stride, pos, surface_up, 1.0, &flat);
    let uphill_stride = pace.pace_stride(stride, pos, surface_up, 1.0, &up_ramp);
    let downhill_stride = pace.pace_stride(stride, pos, surface_up, 1.0, &down_ramp);

    assert_eq!(flat_stride.length(), stride.length(), "Flat ground stride must be unchanged");
    assert!(uphill_stride.length() < flat_stride.length(), "Uphill stride must be penalized");
    assert!(downhill_stride.length() > flat_stride.length(), "Downhill stride must be boosted");
    assert!(downhill_stride.length() > uphill_stride.length(), "Downhill must be strictly faster than uphill");
}
