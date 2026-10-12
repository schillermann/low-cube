use glam::Vec3;
use super::{
    CubeFace, FacePerimeter, PerimeterEdge, PlanetaryTopology, Topology,
};

const HALF_SIZE: f32 = 8.0;

#[test]
fn test_face_perimeter_crossing_evaluation_and_clamping() {
    let perimeter = FacePerimeter::square(HALF_SIZE);
    let frame = CubeFace::Top.surface_frame(HALF_SIZE);

    assert!(perimeter.contains(0.0, 0.0));
    assert!(perimeter.contains_point(&frame, Vec3::new(0.0, 8.0, 0.0)));
    assert!(!perimeter.contains(HALF_SIZE + 0.1, 0.0));

    let inside = perimeter.crossing_point(&frame, Vec3::new(0.0, 8.0, 0.0));
    assert!(inside.is_interior());

    let cross_u = perimeter.crossing_at(HALF_SIZE + 0.75, 2.0);
    assert!(cross_u.is_crossing());
    let (edge, overshoot) = cross_u.dispatch(
        || unreachable!("Must be crossing"),
        |e, o| (e, o),
    );
    assert_eq!(edge, PerimeterEdge::PositiveU);
    assert!((overshoot - 0.75).abs() < 1e-5);

    let cross_neg_u = perimeter.crossing_at(-HALF_SIZE - 0.5, 2.0);
    assert_eq!(cross_neg_u.dispatch(|| unreachable!(), |e, _| e), PerimeterEdge::NegativeU);

    let cross_v = perimeter.crossing_at(2.0, HALF_SIZE + 0.25);
    assert_eq!(cross_v.dispatch(|| unreachable!(), |e, _| e), PerimeterEdge::PositiveV);

    let cross_neg_v = perimeter.crossing_at(2.0, -HALF_SIZE - 0.3);
    assert_eq!(cross_neg_v.dispatch(|| unreachable!(), |e, _| e), PerimeterEdge::NegativeV);

    assert_eq!(perimeter.clamp(12.0, -15.0), (HALF_SIZE, -HALF_SIZE));
}

#[test]
fn test_all_12_undirected_edge_neighborhoods_reciprocity() {
    let topology = PlanetaryTopology::cubic(HALF_SIZE);
    assert_eq!(topology.fold_count(), 24);

    for face in CubeFace::all() {
        for edge in PerimeterEdge::all() {
            let seam = topology.seam(face, edge);
            assert!(seam.connects(face, edge));

            let return_seam = seam.reciprocal_on(&topology);

            assert!(seam.is_reciprocal_to(return_seam));
            assert!(return_seam.is_reciprocal_to(seam));
        }
    }
}

#[test]
fn test_seam_boundary_point_continuity_across_all_24_folds() {
    let topology = PlanetaryTopology::cubic(HALF_SIZE);

    for face in CubeFace::all() {
        let frame = face.surface_frame(HALF_SIZE);
        for edge in PerimeterEdge::all() {
            let seam = topology.seam(face, edge);

            let (u, v) = match edge {
                PerimeterEdge::PositiveU => (HALF_SIZE, 3.0),
                PerimeterEdge::NegativeU => (-HALF_SIZE, -2.5),
                PerimeterEdge::PositiveV => (1.5, HALF_SIZE),
                PerimeterEdge::NegativeV => (-4.0, -HALF_SIZE),
            };

            let edge_point = frame.point_at(u, v, 0.0);
            let folded_point = seam.fold_point(edge_point);
            assert!(
                (folded_point - edge_point).length() < 1e-4,
                "Point on seam {:?} {:?} must remain identical when folded, delta {}",
                face,
                edge,
                (folded_point - edge_point).length()
            );
        }
    }
}

#[test]
fn test_normal_vector_90_degree_switch_across_all_24_folds() {
    let topology = PlanetaryTopology::cubic(HALF_SIZE);

    for face in CubeFace::all() {
        for edge in PerimeterEdge::all() {
            let seam = topology.seam(face, edge);
            let rotated_normal = seam.fold_vector(face.normal());
            let target_normal = seam.destination().normal();

            assert!(
                (rotated_normal - target_normal).length() < 1e-4,
                "Normal for {:?} {:?} must fold to destination normal {:?}, got {:?}",
                face,
                edge,
                target_normal,
                rotated_normal
            );
        }
    }
}

#[test]
fn test_overshoot_step_folding_onto_adjacent_face() {
    let topology = PlanetaryTopology::cubic(HALF_SIZE);
    let seam = topology.seam(CubeFace::Top, PerimeterEdge::PositiveU);

    let top_frame = CubeFace::Top.surface_frame(HALF_SIZE);
    let right_frame = CubeFace::Right.surface_frame(HALF_SIZE);

    let overshoot = 0.5;
    let initial_elevation = 1.8;
    let lateral_v = 3.0;

    let world_pt = top_frame.point_at(HALF_SIZE + overshoot, lateral_v, initial_elevation);
    let folded = seam.fold_step(world_pt);

    assert!(folded.is_on_face(CubeFace::Right));
    assert_eq!(folded.surface_up(), Vec3::X);
    assert_eq!(folded.pull_direction(), Vec3::NEG_X);

    let (u_dst, v_dst) = right_frame.decompose(folded.point());
    let height_dst = right_frame.height_of(folded.point());

    assert!((height_dst - (HALF_SIZE + initial_elevation)).abs() < 1e-4);
    assert!((v_dst - (HALF_SIZE - overshoot)).abs() < 1e-4);
    assert!((u_dst - (-lateral_v)).abs() < 1e-4);

    folded.transfer(|face, pt| {
        assert_eq!(face, CubeFace::Right);
        assert_eq!(pt, folded.point());
    });
}

#[test]
fn test_closed_360_degree_planetary_circuit() {
    let topology = PlanetaryTopology::cubic(HALF_SIZE);

    // Circuit: Top (+Y) -> Right (+X) -> Bottom (-Y) -> Left (-X) -> Top (+Y)
    let circuit = [
        (CubeFace::Top, PerimeterEdge::PositiveU),
        (CubeFace::Right, PerimeterEdge::NegativeV),
        (CubeFace::Bottom, PerimeterEdge::NegativeU),
        (CubeFace::Left, PerimeterEdge::PositiveV),
    ];

    let mut forward_vector = Vec3::X;
    let mut normal_vector = Vec3::Y;

    for (face, edge) in circuit {
        forward_vector = topology.fold_vector(face, edge, forward_vector);
        normal_vector = topology.fold_vector(face, edge, normal_vector);
    }

    assert!(
        (forward_vector - Vec3::X).length() < 1e-4,
        "Forward vector must return to +X after 360° circuit, got {:?}",
        forward_vector
    );
    assert!(
        (normal_vector - Vec3::Y).length() < 1e-4,
        "Normal vector must return to +Y after 360° circuit, got {:?}",
        normal_vector
    );
}
