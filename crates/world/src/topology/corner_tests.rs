use glam::Vec3;
use crate::{ClimateZone, Face, FoldedFace, PlanarFace};

const HALF_SIZE: f32 = 16.0;
const EYE_HEIGHT: f32 = 1.0;

fn top_folded_face() -> FoldedFace {
    let base = Box::new(PlanarFace::in_zone(ClimateZone::Temperate, Vec3::ONE, HALF_SIZE, 16));
    FoldedFace::in_zone(base, ClimateZone::Temperate, HALF_SIZE)
}

#[test]
fn test_planar_face_constrain_delegates_to_seam_without_corner_pinning() {
    let face = top_folded_face();

    // Corner coordinate (0, 17, 17):
    // Top face has surface elevation Y = 16 + 1 = 17.
    // Coordinate Z = 17 exceeds outer perimeter threshold (16.0) by 1.0 across PositiveV.
    let corner = Vec3::new(0.0, 17.0, 17.0);
    let folded = face.constrain(corner, EYE_HEIGHT);

    // Verify coordinate is not clamped to perimeter boundary (16.0)
    assert_ne!(
        folded.z, 16.0,
        "Position must not be pinned or clamped at perimeter boundary 16.0"
    );

    // Expected position folded onto Front face (+Z):
    // Elevation above Front face surface: Z = 16.0 + 1.0 = 17.0
    // Penetration depth into Front face: Y = 16.0 - 1.0 = 15.0
    // Tangential X coordinate remains 0.0
    assert_eq!(
        folded,
        Vec3::new(0.0, 15.0, 17.0),
        "Corner coordinate must fold seamlessly onto Front face"
    );
}

#[test]
fn test_corner_coordinates_all_four_edges_fold_without_clamping() {
    let face = top_folded_face();

    // 1. PositiveU (+X) over edge: X = 17.0, Y = 17.0, Z = 0.0
    // Folds onto Right face (+X): X = 17.0 (elevation), Y = 15.0 (depth), Z = 0.0
    let over_pos_u = face.constrain(Vec3::new(17.0, 17.0, 0.0), EYE_HEIGHT);
    assert_ne!(over_pos_u.x, 16.0);
    assert_eq!(over_pos_u, Vec3::new(17.0, 15.0, 0.0));

    // 2. NegativeU (-X) over edge: X = -17.0, Y = 17.0, Z = 0.0
    // Folds onto Left face (-X): X = -17.0 (elevation), Y = 15.0 (depth), Z = 0.0
    let over_neg_u = face.constrain(Vec3::new(-17.0, 17.0, 0.0), EYE_HEIGHT);
    assert_ne!(over_neg_u.x, -16.0);
    assert_eq!(over_neg_u, Vec3::new(-17.0, 15.0, 0.0));

    // 3. PositiveV (+Z) over edge: X = 0.0, Y = 17.0, Z = 17.0
    // Folds onto Front face (+Z): X = 0.0, Y = 15.0 (depth), Z = 17.0 (elevation)
    let over_pos_v = face.constrain(Vec3::new(0.0, 17.0, 17.0), EYE_HEIGHT);
    assert_ne!(over_pos_v.z, 16.0);
    assert_eq!(over_pos_v, Vec3::new(0.0, 15.0, 17.0));

    // 4. NegativeV (-Z) over edge: X = 0.0, Y = 17.0, Z = -17.0
    // Folds onto Back face (-Z): X = 0.0, Y = 15.0 (depth), Z = -17.0 (elevation)
    let over_neg_v = face.constrain(Vec3::new(0.0, 17.0, -17.0), EYE_HEIGHT);
    assert_ne!(over_neg_v.z, -16.0);
    assert_eq!(over_neg_v, Vec3::new(0.0, 15.0, -17.0));
}

#[test]
fn test_diagonal_corner_coordinate_resolution() {
    let face = top_folded_face();

    // Vertex corner coordinate (17, 17, 17): both U and V exceed perimeter
    let vertex = Vec3::new(17.0, 17.0, 17.0);
    let folded = face.constrain(vertex, EYE_HEIGHT);

    // Resolves to dominant edge without panicking or pinning
    assert_ne!(folded.x, 16.0);
    assert_ne!(folded.z, 16.0);
}
