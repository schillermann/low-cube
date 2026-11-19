use glam::Vec3;
use crate::{ClimateZone, Face, FacetedReliefFace, PerimeterFold};

const HALF_SIZE: f32 = 16.0;
const EYE_HEIGHT: f32 = 1.0;

fn constrain_top_folded(point: Vec3) -> Vec3 {
    let base = FacetedReliefFace::in_zone(ClimateZone::Temperate, HALF_SIZE, 32);
    let fold = PerimeterFold::in_zone(ClimateZone::Temperate, HALF_SIZE);
    fold.fold(point, base.constrain(point, EYE_HEIGHT))
}

#[test]
fn test_planar_face_constrain_delegates_to_seam_without_corner_pinning() {
    let corner = Vec3::new(0.0, 17.0, 17.0);
    let folded = constrain_top_folded(corner);

    assert_ne!(
        folded.z, 16.0,
        "Position must not be pinned or clamped at perimeter boundary 16.0"
    );
    assert_eq!(
        folded,
        Vec3::new(0.0, 15.0, 17.0),
        "Corner coordinate must fold seamlessly onto Front face"
    );
}

#[test]
fn test_corner_coordinates_all_four_edges_fold_without_clamping() {
    let over_pos_u = constrain_top_folded(Vec3::new(17.0, 17.0, 0.0));
    assert_ne!(over_pos_u.x, 16.0);
    assert_eq!(over_pos_u, Vec3::new(17.0, 15.0, 0.0));

    let over_neg_u = constrain_top_folded(Vec3::new(-17.0, 17.0, 0.0));
    assert_ne!(over_neg_u.x, -16.0);
    assert_eq!(over_neg_u, Vec3::new(-17.0, 15.0, 0.0));

    let over_pos_v = constrain_top_folded(Vec3::new(0.0, 17.0, 17.0));
    assert_ne!(over_pos_v.z, 16.0);
    assert_eq!(over_pos_v, Vec3::new(0.0, 15.0, 17.0));

    let over_neg_v = constrain_top_folded(Vec3::new(0.0, 17.0, -17.0));
    assert_ne!(over_neg_v.z, -16.0);
    assert_eq!(over_neg_v, Vec3::new(0.0, 15.0, -17.0));
}

#[test]
fn test_diagonal_corner_coordinate_resolution() {
    let vertex = Vec3::new(17.0, 17.0, 17.0);
    let folded = constrain_top_folded(vertex);

    assert_ne!(folded.x, 16.0);
    assert_ne!(folded.z, 16.0);
}
