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
    let on_ridge = snow_ridge.constrain_surface(Vec3::new(0.5, 9.0, 1.5), 0.5, 1.5, &frame, 1.0);
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
