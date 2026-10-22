use glam::Vec3;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::{MeshTarget, Quad};
use super::{ReliefElevation, ReliefFacet, ReliefPalette};

struct QuadSink {
    quads: Vec<Quad>,
}

impl QuadSink {
    fn new() -> Self {
        Self { quads: Vec::new() }
    }
}

impl MeshTarget for QuadSink {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

struct TestRelief(fn(i32, i32) -> i32);

impl ReliefElevation for TestRelief {
    fn elevation_at(&self, u: i32, v: i32) -> i32 {
        (self.0)(u, v)
    }

    fn surface_height_at(&self, u: f32, v: f32) -> f32 {
        (self.0)(u as i32, v as i32) as f32
    }
}

#[test]
fn test_flat_surface_emits_no_interior_walls() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let facet = ReliefFacet::new();
    let mut sink = QuadSink::new();
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);

    // Uniform flat relief of height 1 everywhere
    let relief = TestRelief(|_, _| 1);
    let cell = GridCell::new(0, 0);
    facet.emit_facet(cell, &frame, &relief, 16, &palette, &mut sink);

    // Only the top surface quad should be emitted; 0 walls!
    assert_eq!(sink.quads.len(), 1, "Flat terrain must emit exactly 1 top quad and 0 walls");
}

#[test]
fn test_full_cube_adjacent_to_ridge_corner_seals_wall() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let facet = ReliefFacet::new();
    let mut sink = QuadSink::new();
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);

    // Cell (0, 0) has height 1 (Full-Cube with higher wall to the north).
    // Cell (1, 0) has height 1 and forms a Ridge-Corner mode 0 (NW corner drops to 0.0).
    let relief = TestRelief(|u, v| {
        if (u, v) == (0, 1) {
            2
        } else if (u, v) == (1, 1) || (u, v) == (2, 0) {
            0
        } else {
            1
        }
    });

    let cell = GridCell::new(0, 0);
    facet.emit_facet(cell, &frame, &relief, 16, &palette, &mut sink);

    // East wall facing cell (1, 0) (normal along Vec3::X, tangent)
    let east_wall = sink
        .quads
        .iter()
        .find(|q| q.transfer(|_, _, _, _, norm, _| norm.dot(Vec3::X) > 0.9))
        .expect("Cell (0, 0) must emit East wall facing Ridge-Corner neighbor cell (1, 0)");

    let min_elevation = east_wall.transfer(|v0, v1, v2, v3, _, _| {
        [frame.height_of(v0), frame.height_of(v1), frame.height_of(v2), frame.height_of(v3)]
            .into_iter()
            .fold(f32::MAX, f32::min)
    });

    assert!(
        min_elevation <= 0.001,
        "East wall must reach down to 0.0 to seal interior against Ridge-Corner, got {min_elevation}"
    );
}

#[test]
fn test_lateral_ridge_corner_attachment_at_highest_pillar_block_without_ground_contact() {
    use crate::block::{BlockMold, CardinalTurn, MoldArchetype};
    use crate::Face;
    use super::FacetedReliefFace;

    let mut face = FacetedReliefFace::in_zone(ClimateZone::Temperate, 8.0, 32);
    let avatar_pos = Vec3::new(0.25, 8.0, 0.25);
    let eye = Vec3::new(0.25, 8.92, 0.25);
    let aim_pillar = Vec3::new(0.0, -0.75, -0.75).normalize();

    // Stack 4 full cubes on top of each other at cell (0, -2) -> height 2.0m
    for _ in 0..4 {
        assert!(face.place_molded_block(eye, aim_pillar, avatar_pos, BlockMold::cube()));
    }
    let pillar_top = face.constrain(Vec3::new(0.25, 12.0, -0.75), 0.0).y - 8.0;
    assert!((pillar_top - 2.0).abs() < 1e-4, "Four stacked blocks must reach 2.0m");

    // Aim at the topmost 4th block of the pillar and attach RidgeCorner laterally into (0, -1)
    let aim_high_side = Vec3::new(0.0, 1.35, -1.0).normalize();
    let ridge_mold = BlockMold::new(MoldArchetype::ConvexRidge, CardinalTurn::Forward);
    assert!(face.place_molded_block(eye, aim_high_side, avatar_pos, ridge_mold));

    // In cell (0, -1), the attached RidgeCorner is at base 1.5..2.0m on the SouthWest half, while the NorthEast half is open air down to ground 0.0m!
    let attached_h = face.constrain(Vec3::new(0.05, 12.0, -0.45), 0.0).y - 8.0;
    let open_ground_h = face.constrain(Vec3::new(0.45, 12.0, -0.05), 0.0).y - 8.0;
    assert!(attached_h > 1.8, "Laterally attached RidgeCorner must sit at 4th block height (1.5..2.0m), got {attached_h}");
    assert!(open_ground_h.abs() < 1e-4, "Uncovered half and space beneath RidgeCorner must have ground at 0.0m without ground contact, got {open_ground_h}");

    let mut sink = QuadSink::new();
    face.emit_surface(&mut sink);
    let has_floating_underside = sink.quads.iter().any(|q| {
        q.transfer(|v0, v1, v2, v3, n, _| {
            n.y < -0.9 && v2 == v3 && (v0.y - 9.5).abs() < 1e-3 && (v1.y - 9.5).abs() < 1e-3
        })
    });
    assert!(has_floating_underside, "Floating RidgeCorner must emit its triangular underside at elevation 1.5m above ground");

    // Also attach a FullCube laterally at the topmost 4th block on the East side of the pillar
    let eye_east = Vec3::new(1.25, 8.92, -0.75);
    let aim_east_side = Vec3::new(-1.0, 1.1, 0.0).normalize();
    assert!(face.place_molded_block(eye_east, aim_east_side, Vec3::new(1.25, 8.0, -0.75), BlockMold::cube()));
    let attached_cube_h = face.constrain(Vec3::new(0.75, 12.0, -0.75), 0.0).y - 8.0;
    assert!((attached_cube_h - 2.0).abs() < 1e-4, "Laterally attached FullCube at 4th block must reach 2.0m, got {attached_cube_h}");
}

#[test]
fn test_ridge_corner_stacked_on_ridge_corner_preserves_lower_block() {
    use crate::block::{CornerHeading, CuboidShape, RidgeCorner};
    use super::TerracedRelief;

    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let facet = ReliefFacet::new();
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut terraced = TerracedRelief::in_span(Box::new(TestRelief(|_, _| 0)), 16);

    assert!(terraced.stack_molded_cell(0, 0, 0.0, Box::new(RidgeCorner::new(CornerHeading::NorthEast)), false));
    assert!(terraced.stack_molded_cell(0, 0, 0.5, Box::new(RidgeCorner::new(CornerHeading::NorthEast)), false));

    let mut sink = QuadSink::new();
    facet.emit_facet(GridCell::new(0, 0), &frame, &terraced, 16, &palette, &mut sink);
    let slope_count = sink.quads.iter().filter(|q| q.transfer(|_, _, _, _, n, _| n.y > 0.4 && n.y < 0.8)).count();
    assert_eq!(slope_count, 2, "Both the lower and upper RidgeCorner blocks must retain their sloped faces");
    assert!((terraced.surface_height_at(0.05, 0.05) - 0.0).abs() < 1e-4, "Uncovered corner of stacked RidgeCorners must remain at ground 0.0");
    assert!(terraced.surface_height_at(0.45, 0.45) > 0.85, "Peak corner of stacked RidgeCorners must reach 2nd block height");

    // Natural terrain RidgeCorner at (1, 0): drops to North (1, 1) and East (2, 0)
    let mut nat_terraced = TerracedRelief::in_span(
        Box::new(TestRelief(|u, v| if (u, v) == (1, 1) || (u, v) == (2, 0) { 1 } else { 2 })),
        16,
    );
    assert!(nat_terraced.stack_molded_cell(1, 0, 1.0, Box::new(RidgeCorner::new(CornerHeading::SouthWest)), false));
    let mut nat_sink = QuadSink::new();
    facet.emit_facet(GridCell::new(1, 0), &frame, &nat_terraced, 16, &palette, &mut nat_sink);
    let nat_slopes = nat_sink.quads.iter().filter(|q| q.transfer(|_, _, _, _, n, _| n.y > 0.4 && n.y < 0.8)).count();
    assert_eq!(nat_slopes, 2, "Placing RidgeCorner on natural terrain RidgeCorner must preserve the terrain RidgeCorner slope");

    // Excavating the penultimate block (0.5..1.0) in a 3-block stack [0.0..0.5, 0.5..1.0, 1.0..1.5] must keep the topmost block (1.0..1.5)
    assert!(terraced.stack_molded_cell(2, 2, 0.0, Box::new(CuboidShape::new(0.5)), true));
    assert!(terraced.stack_molded_cell(2, 2, 0.5, Box::new(CuboidShape::new(0.5)), true));
    assert!(terraced.stack_molded_cell(2, 2, 1.0, Box::new(CuboidShape::new(0.5)), true));
    assert!(terraced.excavate_layer(2, 2, 0.75).1, "Excavating the penultimate block at 0.5..1.0 must succeed");
    assert!((terraced.surface_height_at(1.25, 1.25) - 1.5).abs() < 1e-4, "Topmost block at 1.0..1.5 must remain after excavating penultimate block");
    assert_eq!(terraced.hit_top_at(1.25, 1.25, 0.75), None, "Excavated penultimate block slot 0.5..1.0 must now be empty air");
    assert_eq!(terraced.hit_top_at(1.25, 1.25, 1.25), Some(1.5), "Topmost block at 1.0..1.5 must still be hit by sight rays");
}
