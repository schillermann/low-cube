use glam::Vec3;
use crate::{ClimateZone, FacetedReliefFace, SurfaceFrame};
use super::arboreal::Arboreal;
use super::full_cube_ground::FullCubeGround;
use super::planting_ground::PlantingGround;
use super::solid_arboreal::SolidArboreal;
use super::standing::StandingTree;
use super::surface_ground::SurfaceGround;
use super::temperate_oak::TemperateOak;
use super::woodland_ground::WoodlandGround;

#[test]
fn test_woodland_ground_decorator_differentiates_clearing_and_groves() {
    let frame = ClimateZone::Temperate.surface_frame(16.0);
    let face = Box::new(FacetedReliefFace::in_zone(ClimateZone::Temperate, 16.0, 32));
    let base = Box::new(SurfaceGround::new(face.as_ref(), &frame, 16.0));
    let woodland = WoodlandGround::in_zone(base, ClimateZone::Temperate);

    let mut clearings = 0;
    let mut inhabited = 0;
    for i in -20i32..20i32 {
        let hash = i.wrapping_mul(73856093);
        if woodland.receptive(i, i, hash) {
            inhabited += 1;
        } else {
            clearings += 1;
        }
    }

    assert!(clearings > 0, "Must have open clearings with zero trees");
    assert!(inhabited > 0, "Must have inhabited tree sites");
}

#[test]
fn test_full_cube_ground_decorator_identifies_flat_full_cube() {
    let frame = ClimateZone::Temperate.surface_frame(16.0);
    let face = Box::new(FacetedReliefFace::in_zone(ClimateZone::Temperate, 16.0, 32));
    let base = Box::new(SurfaceGround::new(face.as_ref(), &frame, 16.0));
    let full_cube = FullCubeGround::new(base, face.as_ref(), &frame, 16.0);

    assert!(full_cube.receptive(0, 0, 0), "Flat spawn sanctuary cell must be receptive Full-Cube");
}

#[test]
fn test_solid_arboreal_decorator_enforces_physical_trunk_collision() {
    let frame = SurfaceFrame::new(Vec3::Y, Vec3::X, Vec3::Z, 16.0);
    let tree = Box::new(StandingTree::of_species(Box::new(TemperateOak::new()), frame, 0.0, 0.0, 1.5));
    let solid = SolidArboreal::rooted(tree, 0.0, 0.0, 1.5, 0.65, 3.0);

    let hill_base = frame.point_at(0.2, 0.2, 1.5);
    let constrained = solid.constrain_surface(hill_base, 0.2, 0.2, &frame, 0.0);
    assert!(
        frame.elevation_of(constrained) >= 4.4,
        "SolidArboreal must elevate surface to block passage within trunk radius at root elevation"
    );

    let tunnel_base = frame.point_at(0.2, 0.2, -0.5);
    let tunnel_constrained = solid.constrain_surface(tunnel_base, 0.2, 0.2, &frame, 0.0);
    assert_eq!(
        tunnel_constrained,
        tunnel_base,
        "SolidArboreal must not block passage in an excavated tunnel beneath the tree's root elevation"
    );
}

#[test]
fn test_standing_tree_canopy_drag_and_cleared_plot_zero_drag() {
    let frame = SurfaceFrame::new(Vec3::Y, Vec3::X, Vec3::Z, 16.0);
    let tree = Box::new(StandingTree::of_species(Box::new(TemperateOak::new()), frame, 0.0, 0.0, 0.0));
    let solid = SolidArboreal::new(tree, 0.0, 0.0, 0.65, 3.0);

    // Inside oak canopy: u=0.5, v=0.5, height=3.0
    let inside_drag = solid.canopy_drag(0.5, 0.5, 3.0);
    assert!(inside_drag > 0.8, "Inside standing tree crown must provide high canopy drag, got {}", inside_drag);

    // Outside oak canopy: u=10.0, v=10.0, height=3.0
    let outside_drag = solid.canopy_drag(10.0, 10.0, 3.0);
    assert_eq!(outside_drag, 0.0, "Outside crown must provide zero drag");

    // Strike and fell tree
    struct MockDepository;
    impl super::depository::TimberDepository for MockDepository {
        fn deposit_timber(&mut self, _yield_type: super::timber_yield::TimberYield, _quantity: u32) {}
    }
    let mut dep = MockDepository;
    let felled = solid.strike(10, &mut dep);
    assert_eq!(
        felled.canopy_drag(0.5, 0.5, 3.0),
        0.0,
        "Felled cleared plot must provide zero canopy drag"
    );
}
