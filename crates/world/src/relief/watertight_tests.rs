use glam::Vec3;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::{MeshTarget, Quad};
use super::{ReliefElevation, ReliefFacet, ReliefPalette};

struct QuadListCollector {
    quads: Vec<Quad>,
}

impl QuadListCollector {
    fn new() -> Self {
        Self { quads: Vec::new() }
    }
}

impl MeshTarget for QuadListCollector {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

struct TestReliefGrid(fn(i32, i32) -> i32);

impl ReliefElevation for TestReliefGrid {
    fn elevation_at(&self, u: i32, v: i32) -> i32 {
        (self.0)(u, v)
    }

    fn surface_height_at(&self, u: f32, v: f32) -> f32 {
        (self.0)(u as i32, v as i32) as f32
    }
}

#[test]
fn test_stacked_cubes_adjacent_to_ramp_wedge_seals_lower_cube() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let facet = ReliefFacet::new();
    let mut collector = QuadListCollector::new();
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);

    // Cell (0, 0) has height 2 (two stacked Full-Cubes).
    // Cell (1, 0) has height 1 and slopes North towards (1, 1) which has height 0.
    let relief = TestReliefGrid(|u, v| {
        if (u, v) == (0, 0) {
            2
        } else if (u, v) == (1, 0) {
            1
        } else if (u, v) == (1, 1) {
            0
        } else {
            2
        }
    });

    let cell = GridCell::new(0, 0);
    facet.emit_facet(cell, &frame, &relief, 16, &palette, &mut collector);

    // Find the East wall quad (normal along Vec3::X, tangent)
    let east_wall = collector
        .quads
        .iter()
        .find(|q| q.transfer(|_, _, _, _, norm, _| norm.dot(Vec3::X) > 0.9))
        .expect("Cell (0, 0) must emit East wall facing cell (1, 0)");

    let min_elevation = east_wall.transfer(|v0, v1, v2, v3, _, _| {
        [frame.height_of(v0), frame.height_of(v1), frame.height_of(v2), frame.height_of(v3)]
            .into_iter()
            .fold(f32::MAX, f32::min)
    });

    // The East wall must reach down to 0.0 (the ramp's low edge) to seal the lower cube
    assert!(
        min_elevation <= 0.001,
        "East wall must extend down to elevation 0.0 to seal interior of lower Full-Cube, observed min_elevation = {min_elevation}"
    );
}

#[test]
fn test_ramp_wedge_seals_against_higher_neighbor() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let facet = ReliefFacet::new();
    let mut collector = QuadListCollector::new();
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);

    // Cell (0, 0) has height 3 (three Full-Cubes high).
    // South neighbor (0, -1) has height 1 and slopes East towards (1, -1) which has height 0.
    let relief = TestReliefGrid(|u, v| {
        if (u, v) == (0, 0) {
            3
        } else if (u, v) == (0, -1) {
            1
        } else if (u, v) == (1, -1) {
            0
        } else {
            3
        }
    });

    let cell = GridCell::new(0, 0);
    facet.emit_facet(cell, &frame, &relief, 16, &palette, &mut collector);

    // Find the South wall quad (normal along -Vec3::Z, -bitangent)
    let south_wall = collector
        .quads
        .iter()
        .find(|q| q.transfer(|_, _, _, _, norm, _| norm.dot(-Vec3::Z) > 0.9))
        .expect("Cell (0, 0) must emit South wall facing cell (0, -1)");

    let min_elevation = south_wall.transfer(|v0, v1, v2, v3, _, _| {
        [frame.height_of(v0), frame.height_of(v1), frame.height_of(v2), frame.height_of(v3)]
            .into_iter()
            .fold(f32::MAX, f32::min)
    });

    // The South wall must reach down to 0.0 at the low corner of the ramp wedge
    assert!(
        min_elevation <= 0.001,
        "South wall must extend down to elevation 0.0 to follow ramp wedge slope, observed min_elevation = {min_elevation}"
    );
}

#[test]
fn test_single_full_cube_adjacent_to_ramp_wedge_seals_cube_wall() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let facet = ReliefFacet::new();
    let mut collector = QuadListCollector::new();
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);

    // Cell (0, 0) has height 1 (single Full-Cube, with cliff at (0, 1) of height 2).
    // Cell (1, 0) has height 1 and slopes North towards (1, 1) which has height 0.
    let relief = TestReliefGrid(|u, v| {
        if (u, v) == (0, 1) {
            2
        } else if (u, v) == (1, 1) {
            0
        } else {
            1
        }
    });

    let cell = GridCell::new(0, 0);
    facet.emit_facet(cell, &frame, &relief, 16, &palette, &mut collector);

    // Find the East wall quad (normal along Vec3::X, tangent)
    let east_wall = collector
        .quads
        .iter()
        .find(|q| q.transfer(|_, _, _, _, norm, _| norm.dot(Vec3::X) > 0.9))
        .expect("Cell (0, 0) must emit East wall facing Ramp-Wedge neighbor cell (1, 0)");

    let min_elevation = east_wall.transfer(|v0, v1, v2, v3, _, _| {
        [frame.height_of(v0), frame.height_of(v1), frame.height_of(v2), frame.height_of(v3)]
            .into_iter()
            .fold(f32::MAX, f32::min)
    });

    // The East wall must reach down to 0.0 (the ramp's low edge) to seal the cube interior
    assert!(
        min_elevation <= 0.001,
        "East wall must extend down to elevation 0.0 to seal interior of Full-Cube, observed min_elevation = {min_elevation}"
    );
}

