use glam::Vec3;
use crate::threshold::{StepThreshold, SurfaceClearance, SurfaceStep};

#[test]
fn test_threshold_permits_human_stride() {
    let threshold = StepThreshold::human_stride();
    assert!(threshold.permits_rise(0.2));
    assert!(threshold.permits_rise(0.35));
    assert!(!threshold.permits_rise(0.36));
}

#[test]
fn test_threshold_detects_floor_and_cliff() {
    let threshold = StepThreshold::human_stride();
    assert!(threshold.permits_floor_contact(0.0));
    assert!(threshold.permits_floor_contact(-0.2));
    assert!(!threshold.permits_floor_contact(-0.5));
    assert!(threshold.is_cliff_impact(-0.5));
    assert!(!threshold.is_cliff_impact(-0.1));
}

#[test]
fn test_clearance_semantics() {
    let threshold = StepThreshold::human_stride();
    let floor = SurfaceClearance::new(0.0);
    assert!(!floor.is_precipice());
    assert!(floor.is_floor_contact(threshold));
    assert!(!floor.is_cliff_impact(threshold));

    let cliff = SurfaceClearance::new(-0.5);
    assert!(!cliff.is_precipice());
    assert!(!cliff.is_floor_contact(threshold));
    assert!(cliff.is_cliff_impact(threshold));

    let drop = SurfaceClearance::new(0.1);
    assert!(drop.is_precipice());
}

#[test]
fn test_surface_step_walkability() {
    let threshold = StepThreshold::human_stride();
    let walkable = SurfaceStep::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.2, 0.1), Vec3::Y);
    assert!(walkable.is_walkable_with(threshold));

    let blocked = SurfaceStep::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.5, 0.1), Vec3::Y);
    assert!(!blocked.is_walkable_with(threshold));
}
