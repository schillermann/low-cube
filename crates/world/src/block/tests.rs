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

    // Inside form: elevation 1.0, player height 1.0 -> y = 8.0 + 1.0 + 1.0 = 10.0
    let pos = form.constrain_surface(Vec3::new(1.5, 9.0, 1.5), 1.5, 1.5, &frame, 1.0);
    assert_eq!(pos.y, 10.0);

    // Outside form: position unchanged
    let out = form.constrain_surface(Vec3::new(0.5, 9.0, 0.5), 0.5, 0.5, &frame, 1.0);
    assert_eq!(out.y, 9.0);
}

#[test]
fn test_material_blocks_derived_from_forms() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 8.0);

    // StoneBlock derived from wedge
    let stone_wedge = StoneBlock::wedge(0, 0, 0.0, WedgeHeading::North);
    let on_wedge = stone_wedge.constrain_surface(Vec3::new(0.5, 9.0, 0.5), 0.5, 0.5, &frame, 1.0);
    assert!((on_wedge.y - 9.5).abs() < 1e-4);

    // SandBlock derived from slab
    let sand_slab = SandBlock::slab(1, 0, 0.0);
    let on_slab = sand_slab.constrain_surface(Vec3::new(1.5, 9.0, 0.5), 1.5, 0.5, &frame, 1.0);
    assert_eq!(on_slab.y, 9.5);

    // DirtBlock derived from cube
    let dirt_cube = DirtBlock::cube(2, 0, 0.0);
    let on_cube = dirt_cube.constrain_surface(Vec3::new(2.5, 9.0, 0.5), 2.5, 0.5, &frame, 1.0);
    assert_eq!(on_cube.y, 10.0);

    // SnowBlock derived from ridge corner
    let snow_ridge = SnowBlock::ridge_corner(0, 1, 0.0, CornerHeading::NorthEast);
    let on_ridge = snow_ridge.constrain_surface(Vec3::new(0.75, 9.0, 1.75), 0.75, 1.75, &frame, 1.0);
    assert!((on_ridge.y - 9.5).abs() < 1e-4);

    // SnowBlock derived from pyramidal peak (0.5m height, 45° slope)
    let snow_peak = SnowBlock::peak(1, 1, 0.0);
    let on_peak_tip = snow_peak.constrain_surface(Vec3::new(1.5, 9.0, 1.5), 1.5, 1.5, &frame, 1.0);
    assert_eq!(on_peak_tip.y, 9.5);

    // StoneBlock derived from stair
    let stone_stair = StoneBlock::stair(2, 1, 0.0, WedgeHeading::North);
    let on_stair_lower = stone_stair.constrain_surface(Vec3::new(2.5, 9.0, 1.25), 2.5, 1.25, &frame, 1.0);
    assert_eq!(on_stair_lower.y, 9.5);
    let on_stair_upper = stone_stair.constrain_surface(Vec3::new(2.5, 9.0, 1.75), 2.5, 1.75, &frame, 1.0);
    assert_eq!(on_stair_upper.y, 10.0);
}

#[test]
fn test_elevated_shape_decorator() {
    use super::{CuboidShape, ElevatedShape, VoxelShape};

    let shape = Box::new(CuboidShape::half_slab());
    let elevated = ElevatedShape::new(shape, 1.0);

    // Half slab (0.5m) elevated by 1.0m -> elevation 1.5m
    assert_eq!(elevated.elevation_at(0.5, 0.5), 1.5);
}

#[test]
fn test_gentle_wedge_and_corner_shapes() {
    use super::{GentleRidge, GentleValley, GentleWedge, StairCorner, VoxelShape};

    // Lower gentle wedge: rises 0.0 to 0.5 along North (+V)
    let lower_wedge = GentleWedge::lower(WedgeHeading::North);
    assert_eq!(lower_wedge.elevation_at(0.5, 0.0), 0.0);
    assert_eq!(lower_wedge.elevation_at(0.5, 0.5), 0.25);
    assert_eq!(lower_wedge.elevation_at(0.5, 1.0), 0.5);

    // Upper gentle wedge: rises 0.5 to 1.0 along North (+V)
    let upper_wedge = GentleWedge::upper(WedgeHeading::North);
    assert_eq!(upper_wedge.elevation_at(0.5, 0.0), 0.5);
    assert_eq!(upper_wedge.elevation_at(0.5, 0.5), 0.75);
    assert_eq!(upper_wedge.elevation_at(0.5, 1.0), 1.0);

    // Gentle ridge corner: convex lower (0.0 to 0.5)
    let lower_ridge = GentleRidge::lower(CornerHeading::NorthEast);
    assert_eq!(lower_ridge.elevation_at(0.0, 0.0), 0.0);
    assert_eq!(lower_ridge.elevation_at(1.0, 1.0), 0.5);

    // Gentle valley corner: concave lower (0.0 to 0.5)
    let lower_valley = GentleValley::lower(CornerHeading::NorthEast);
    assert_eq!(lower_valley.elevation_at(0.0, 0.0), 0.0);
    assert_eq!(lower_valley.elevation_at(1.0, 0.0), 0.5);

    // Stair corner: outer convex and inner concave
    let convex_stair = StairCorner::convex(CornerHeading::NorthEast);
    assert_eq!(convex_stair.elevation_at(0.2, 0.2), 0.5);
    assert_eq!(convex_stair.elevation_at(0.8, 0.8), 1.0);

    let concave_stair = StairCorner::concave(CornerHeading::NorthEast);
    assert_eq!(concave_stair.elevation_at(0.2, 0.2), 0.5);
    assert_eq!(concave_stair.elevation_at(0.8, 0.2), 1.0);
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

