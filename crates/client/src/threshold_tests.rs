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
    assert!(threshold.cliff_impact(-0.5));
    assert!(!threshold.cliff_impact(-0.1));
}

#[test]
fn test_clearance_semantics() {
    let threshold = StepThreshold::human_stride();
    let floor = SurfaceClearance::Planar(0.0);
    assert!(!floor.precipice());
    assert!(floor.floor_contact(threshold));
    assert!(!floor.cliff_impact(threshold));

    let cliff = SurfaceClearance::Planar(-0.5);
    assert!(!cliff.precipice());
    assert!(!cliff.floor_contact(threshold));
    assert!(cliff.cliff_impact(threshold));

    let drop = SurfaceClearance::Planar(0.1);
    assert!(drop.precipice());

    let seam = SurfaceClearance::FoldedSeam;
    assert!(!seam.precipice());
    assert!(seam.floor_contact(threshold));
    assert!(!seam.cliff_impact(threshold));
}

#[test]
fn test_surface_step_walkability() {
    let threshold = StepThreshold::human_stride();
    let walkable = SurfaceStep::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.2, 0.1), Vec3::Y);
    assert!(walkable.walkable_with(threshold));

    let blocked = SurfaceStep::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.5, 0.1), Vec3::Y);
    assert!(!blocked.walkable_with(threshold));
}
