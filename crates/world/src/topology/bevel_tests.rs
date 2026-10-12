use glam::Vec3;
use super::{CubeFace, PerimeterEdge, PlanetaryTopology, SeamBevel};

const HALF_SIZE: f32 = 8.0;
const MARGIN: f32 = 0.5;

#[test]
fn test_seam_bevel_blend_factor_and_bounds() {
    let bevel = SeamBevel::between(CubeFace::Top, CubeFace::Front, HALF_SIZE, MARGIN);

    // Deep in source face (Top +Y): Y = 9.0, Z = 8.0 - 1.0 = 7.0 -> delta = -2.0 -> clamped to 0.0
    let inside_top = Vec3::new(0.0, 9.0, 7.0);
    assert_eq!(bevel.blend_factor(inside_top), 0.0);
    assert!(!bevel.contains(inside_top));

    // Entry to bevel zone on Top side: delta = -0.5
    let entry_top = Vec3::new(0.0, 9.0, 8.5);
    assert!((bevel.blend_factor(entry_top) - 0.0).abs() < 1e-5);
    assert!(bevel.contains(entry_top));

    // Exact edge bisector: Y = 9.0, Z = 9.0 -> delta = 0.0 -> blend = 0.5
    let bisector = Vec3::new(0.0, 9.0, 9.0);
    assert!((bevel.blend_factor(bisector) - 0.5).abs() < 1e-5);
    assert!(bevel.contains(bisector));

    // Exit from bevel zone on Front side: delta = +0.5
    let exit_front = Vec3::new(0.0, 8.5, 9.0);
    assert!((bevel.blend_factor(exit_front) - 1.0).abs() < 1e-5);
    assert!(bevel.contains(exit_front));

    // Deep in destination face (Front +Z): Y = 7.0, Z = 9.0 -> delta = +2.0 -> clamped to 1.0
    let inside_front = Vec3::new(0.0, 7.0, 9.0);
    assert_eq!(bevel.blend_factor(inside_front), 1.0);
    assert!(!bevel.contains(inside_front));
}

#[test]
fn test_seam_bevel_smooth_normal_transition() {
    let bevel = SeamBevel::between(CubeFace::Top, CubeFace::Front, HALF_SIZE, MARGIN);

    // On Top face: normal is pure +Y
    let top_pt = Vec3::new(0.0, 9.0, 7.0);
    let n_top = bevel.normal_at(top_pt);
    assert!((n_top - Vec3::Y).length() < 1e-5);

    // On exact 45° seam edge: normal is (Y + Z).normalize()
    let bisector = Vec3::new(0.0, 9.0, 9.0);
    let n_bisector = bevel.normal_at(bisector);
    let expected_bisector = (Vec3::Y + Vec3::Z).normalize();
    assert!((n_bisector - expected_bisector).length() < 1e-5);
    assert!((n_bisector.length() - 1.0).abs() < 1e-5);

    // On Front face: normal is pure +Z
    let front_pt = Vec3::new(0.0, 7.0, 9.0);
    let n_front = bevel.normal_at(front_pt);
    assert!((n_front - Vec3::Z).length() < 1e-5);

    // Intermediate progression: verify unit length across all steps
    for step in 0..=10 {
        let t = (step as f32) / 10.0;
        let delta = -MARGIN + 2.0 * MARGIN * t;
        let pt = Vec3::new(0.0, 8.5, 8.5 + delta);
        let normal = bevel.normal_at(pt);
        assert!(
            (normal.length() - 1.0).abs() < 1e-5,
            "Beveled normal must remain unit length, got length {}",
            normal.length()
        );
    }
}

#[test]
fn test_seam_bevel_tell_dont_ask_evaluate() {
    let bevel = SeamBevel::between(CubeFace::Top, CubeFace::Front, HALF_SIZE, MARGIN);
    let bisector = Vec3::new(0.0, 9.0, 9.0);

    bevel.evaluate(bisector, |normal, factor| {
        assert!((factor - 0.5).abs() < 1e-5);
        let expected = (Vec3::Y + Vec3::Z).normalize();
        assert!((normal - expected).length() < 1e-5);
    });
}

#[test]
fn test_seam_bevel_derivation_from_topology() {
    let topology = PlanetaryTopology::cubic(HALF_SIZE);
    let bevel = topology.bevel(CubeFace::Top, PerimeterEdge::PositiveV, HALF_SIZE, MARGIN);

    let bisector = Vec3::new(0.0, 9.0, 9.0);
    assert!(bevel.contains(bisector));
    let n = bevel.normal_at(bisector);
    assert!((n - (Vec3::Y + Vec3::Z).normalize()).length() < 1e-5);
}
