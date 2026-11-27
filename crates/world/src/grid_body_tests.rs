use glam::Vec3;
use crate::{
    CentrifugalRing, CoriolisDeflection, DeckDynamics, InertialForce, MagBootGrip, ThrustGravity,
    ZeroGravityField,
};

#[test]
fn test_thrust_gravity_pushes_crew_opposite_to_burn() {
    let field = ThrustGravity::new(ZeroGravityField::new(), Vec3::new(0.0, 9.81, 0.0));
    let a = field.acceleration_at(Vec3::ZERO, Vec3::ZERO);
    assert!((a.y + 9.81).abs() < 1e-5);
}

#[test]
fn test_centrifugal_ring_yields_omega_squared_radius() {
    let omega = Vec3::new(0.0, 0.5, 0.0);
    let field = CentrifugalRing::new(ZeroGravityField::new(), omega, Vec3::ZERO);
    let a = field.acceleration_at(Vec3::new(20.0, 0.0, 0.0), Vec3::ZERO);
    assert!((a.x - 0.25 * 20.0).abs() < 1e-4);
}

#[test]
fn test_coriolis_deflects_moving_crew_sideways() {
    let field = CoriolisDeflection::new(ZeroGravityField::new(), Vec3::new(0.0, 1.0, 0.0));
    let a = field.acceleration_at(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0));
    assert!((a - Vec3::new(0.0, 0.0, 2.0)).length() < 1e-5);
}

#[test]
fn test_deck_dynamics_without_thrust_or_spin_is_weightless() {
    let dynamics = DeckDynamics::new(Vec3::ZERO, Vec3::ZERO, Vec3::ZERO);
    assert!(dynamics.fictitious_acceleration(Vec3::ONE, Vec3::ONE).length() < 1e-6);
}

#[test]
fn test_mag_boots_latch_in_zero_g_and_release_on_kick() {
    let up = Vec3::Y;
    assert!(MagBootGrip::evaluate(true, up, false, 0.0).latched());
    assert!(!MagBootGrip::evaluate(true, up, true, 0.0).latched());
    assert!(!MagBootGrip::evaluate(true, up, false, 96.0).latched());
    assert!(!MagBootGrip::evaluate(false, up, false, 0.0).latched());
}
