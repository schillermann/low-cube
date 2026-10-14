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
