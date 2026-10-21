use glam::Vec3;
use super::{
    Block, CornerHeading, DirtBlock, FormBlock, SandBlock, SnowBlock,
    StoneBlock, WedgeHeading,
};
use crate::frame::SurfaceFrame;

#[test]
fn test_form_block_cube_constraint() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 8.0);
    let form = FormBlock::cube(1, 1, 0.0);

    // Inside form (u in [0.5, 1.0], v in [0.5, 1.0]): elevation 0.5, player height 1.0 -> y = 8.0 + 0.5 + 1.0 = 9.5
    let pos = form.constrain_surface(Vec3::new(0.75, 9.0, 0.75), 0.75, 0.75, &frame, 1.0);
    assert_eq!(pos.y, 9.5);

    // Outside form: position unchanged
    let out = form.constrain_surface(Vec3::new(0.25, 9.0, 0.25), 0.25, 0.25, &frame, 1.0);
    assert_eq!(out.y, 9.0);
}

#[test]
fn test_material_blocks_derived_from_forms() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 8.0);

    // StoneBlock derived from wedge at (0, 0) -> u in [0.0, 0.5], v in [0.0, 0.5]
    let stone_wedge = StoneBlock::wedge(0, 0, 0.0, WedgeHeading::North);
    let on_wedge = stone_wedge.constrain_surface(Vec3::new(0.25, 9.0, 0.25), 0.25, 0.25, &frame, 1.0);
    assert!((on_wedge.y - 9.25).abs() < 1e-4);

    // SandBlock derived from slab at (1, 0) -> u in [0.5, 1.0], v in [0.0, 0.5]
    let sand_slab = SandBlock::slab(1, 0, 0.0);
    let on_slab = sand_slab.constrain_surface(Vec3::new(0.75, 9.0, 0.25), 0.75, 0.25, &frame, 1.0);
    assert_eq!(on_slab.y, 9.25);

    // DirtBlock derived from cube at (2, 0) -> u in [1.0, 1.5], v in [0.0, 0.5]
    let dirt_cube = DirtBlock::cube(2, 0, 0.0);
    let on_cube = dirt_cube.constrain_surface(Vec3::new(1.25, 9.0, 0.25), 1.25, 0.25, &frame, 1.0);
    assert_eq!(on_cube.y, 9.5);

    // SnowBlock derived from ridge corner at (0, 1) -> u in [0.0, 0.5], v in [0.5, 1.0]
    let snow_ridge = SnowBlock::ridge_corner(0, 1, 0.0, CornerHeading::NorthEast);
    let on_ridge = snow_ridge.constrain_surface(Vec3::new(0.375, 9.0, 0.875), 0.375, 0.875, &frame, 1.0);
    assert!((on_ridge.y - 9.25).abs() < 1e-4);

    // SnowBlock derived from pyramidal peak (0.25m height, 45° slope) at (1, 1) -> center (0.75, 0.75)
    let snow_peak = SnowBlock::peak(1, 1, 0.0);
    let on_peak_tip = snow_peak.constrain_surface(Vec3::new(0.75, 9.0, 0.75), 0.75, 0.75, &frame, 1.0);
    assert_eq!(on_peak_tip.y, 9.25);

    // StoneBlock derived from stair at (2, 1) -> u in [1.0, 1.5], v in [0.5, 1.0]
    let stone_stair = StoneBlock::stair(2, 1, 0.0, WedgeHeading::North);
    let on_stair_lower = stone_stair.constrain_surface(Vec3::new(1.25, 9.0, 0.625), 1.25, 0.625, &frame, 1.0);
    assert_eq!(on_stair_lower.y, 9.25);
    let on_stair_upper = stone_stair.constrain_surface(Vec3::new(1.25, 9.0, 0.875), 1.25, 0.875, &frame, 1.0);
    assert_eq!(on_stair_upper.y, 9.5);
}

#[test]
fn test_elevated_shape_decorator() {
    use super::{CuboidShape, ElevatedShape, VoxelShape};

    let shape = Box::new(CuboidShape::half_slab());
    let elevated = ElevatedShape::new(shape, 0.5);

    // Half slab (0.25m) elevated by 0.5m -> elevation 0.75m
    assert_eq!(elevated.elevation_at(0.5, 0.5), 0.75);
}

#[test]
fn test_gentle_wedge_and_corner_shapes() {
    use super::{GentleRidge, GentleValley, GentleWedge, StairCorner, VoxelShape};

    // Lower gentle wedge: rises 0.0 to 0.25 along North (+V)
    let lower_wedge = GentleWedge::lower(WedgeHeading::North);
    assert_eq!(lower_wedge.elevation_at(0.5, 0.0), 0.0);
    assert_eq!(lower_wedge.elevation_at(0.5, 0.5), 0.125);
    assert_eq!(lower_wedge.elevation_at(0.5, 1.0), 0.25);

    // Upper gentle wedge: rises 0.25 to 0.5 along North (+V)
    let upper_wedge = GentleWedge::upper(WedgeHeading::North);
    assert_eq!(upper_wedge.elevation_at(0.5, 0.0), 0.25);
    assert_eq!(upper_wedge.elevation_at(0.5, 0.5), 0.375);
    assert_eq!(upper_wedge.elevation_at(0.5, 1.0), 0.5);

    // Gentle ridge corner: convex lower (0.0 to 0.25)
    let lower_ridge = GentleRidge::lower(CornerHeading::NorthEast);
    assert_eq!(lower_ridge.elevation_at(0.0, 0.0), 0.0);
    assert_eq!(lower_ridge.elevation_at(1.0, 1.0), 0.25);

    // Gentle valley corner: concave lower (0.0 to 0.25)
    let lower_valley = GentleValley::lower(CornerHeading::NorthEast);
    assert_eq!(lower_valley.elevation_at(0.0, 0.0), 0.0);
    assert_eq!(lower_valley.elevation_at(1.0, 0.0), 0.25);

    // Stair corner: outer convex and inner concave
    let convex_stair = StairCorner::convex(CornerHeading::NorthEast);
    assert_eq!(convex_stair.elevation_at(0.2, 0.2), 0.25);
    assert_eq!(convex_stair.elevation_at(0.8, 0.8), 0.5);

    let concave_stair = StairCorner::concave(CornerHeading::NorthEast);
    assert_eq!(concave_stair.elevation_at(0.2, 0.2), 0.25);
    assert_eq!(concave_stair.elevation_at(0.8, 0.2), 0.5);

    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 8.0);
    for heading in [CornerHeading::NorthEast, CornerHeading::NorthWest, CornerHeading::SouthEast, CornerHeading::SouthWest] {
        for concave in [false, true] {
            let mut null = NullSink;
            let mut sink = crate::CountedMeshTarget::new(&mut null);
            StairCorner::new(heading, concave).emit_shape(super::GridCell::new(0, 0), 0.0, &frame, Vec3::ONE, &mut sink);
            assert_eq!(sink.count(), 14, "Every convex and concave StairCorner heading must emit all 14 quads (4 treads + 2 risers + 8 outer walls)");
        }
    }
}

struct NullSink;
impl crate::MeshTarget for NullSink {
    fn push_quad(&mut self, _quad: crate::Quad) {}
}

#[test]
fn test_voxel_shapes_have_distinct_signature_colors() {
    use super::{CuboidShape, RampWedge, RidgeCorner, ValleyCorner, PyramidPeak, StairFlight, VoxelShape};
    let cube = CuboidShape::full_cube();
    let slab = CuboidShape::half_slab();
    let wedge = RampWedge::new(WedgeHeading::North);
    let ridge = RidgeCorner::new(CornerHeading::NorthEast);
    let valley = ValleyCorner::new(CornerHeading::NorthEast);
    let peak = PyramidPeak::slope_aligned();
    let stair = StairFlight::new(WedgeHeading::North);

    assert_eq!(cube.shape_color(), Vec3::new(0.0, 0.9, 1.0));
    assert_eq!(slab.shape_color(), Vec3::new(0.1, 0.95, 0.7));
    assert_eq!(wedge.shape_color(), Vec3::new(1.0, 0.55, 0.0));
    assert_eq!(ridge.shape_color(), Vec3::new(1.0, 0.0, 0.8));
    assert_eq!(valley.shape_color(), Vec3::new(1.0, 0.95, 0.0));
    assert_eq!(peak.shape_color(), Vec3::new(0.75, 0.2, 1.0));
    assert_eq!(stair.shape_color(), Vec3::new(0.1, 0.95, 0.2));
}

#[test]
fn test_diagonal_slope_shape_elevation_and_constraint() {
    use super::{CornerHeading, DiagonalSlope, VoxelShape};
    let slope_ne = DiagonalSlope::new(CornerHeading::NorthEast);
    assert_eq!(slope_ne.elevation_at(0.0, 0.0), 0.0);
    assert_eq!(slope_ne.elevation_at(0.5, 0.5), 0.25);
    assert_eq!(slope_ne.elevation_at(1.0, 1.0), 0.5);

    let slope_sw = DiagonalSlope::new(CornerHeading::SouthWest);
    assert_eq!(slope_sw.elevation_at(1.0, 1.0), 0.0);
    assert_eq!(slope_sw.elevation_at(0.5, 0.5), 0.25);
    assert_eq!(slope_sw.elevation_at(0.0, 0.0), 0.5);
}

#[test]
fn test_block_mold_cycles_all_sixteen_forms_and_rotates_all_four_directions() {
    use super::{BlockMold, CardinalTurn, MoldArchetype};
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 8.0);
    let mut mold = BlockMold::cube();
    assert!(mold.full_height_flat());
    for expected in 1..16u8 {
        mold = mold.next_form();
        assert_eq!(mold, BlockMold::new(MoldArchetype::from_code(expected), CardinalTurn::Forward));
        assert!(!mold.full_height_flat());
    }
    assert_eq!(mold.next_form(), BlockMold::cube());
    assert_eq!(BlockMold::cube().prev_form(), BlockMold::new(MoldArchetype::ConcaveStairCorner, CardinalTurn::Forward));

    // Verify 4-way rotation of a RampWedge along -Z (South) -> +X (East) -> +Z (North) -> -X (West)
    let wedge_fwd = BlockMold::new(MoldArchetype::RampWedge, CardinalTurn::Forward);
    let s_fwd = wedge_fwd.shape_for(Vec3::NEG_Z, &frame);
    assert!((s_fwd.elevation_at(0.5, 0.0) - 0.5).abs() < 1e-4 && s_fwd.elevation_at(0.5, 1.0).abs() < 1e-4);

    let s_right = wedge_fwd.turn_clockwise().shape_for(Vec3::NEG_Z, &frame);
    assert!((s_right.elevation_at(1.0, 0.5) - 0.5).abs() < 1e-4 && s_right.elevation_at(0.0, 0.5).abs() < 1e-4);

    let s_back = wedge_fwd.turn_clockwise().turn_clockwise().shape_for(Vec3::NEG_Z, &frame);
    assert!((s_back.elevation_at(0.5, 1.0) - 0.5).abs() < 1e-4 && s_back.elevation_at(0.5, 0.0).abs() < 1e-4);

    let s_left = wedge_fwd.turn_counter_clockwise().shape_for(Vec3::NEG_Z, &frame);
    assert!((s_left.elevation_at(0.0, 0.5) - 0.5).abs() < 1e-4 && s_left.elevation_at(1.0, 0.5).abs() < 1e-4);
}

