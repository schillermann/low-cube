use glam::Vec3;
use crate::block::GridCell;
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

struct FlatRelief;

impl ReliefElevation for FlatRelief {
    fn elevation_at(&self, _u: i32, _v: i32) -> i32 {
        0
    }

    fn surface_height_at(&self, _u: f32, _v: f32) -> f32 {
        0.0
    }
}

#[test]
fn test_quad_sphere_meridian_border_cells_meet_flush_on_sphere() {
    let facet = ReliefFacet::new();
    let relief = FlatRelief;
    let half_blocks = 32;
    let radius = (half_blocks as f32) * 0.5;

    let frame_y = ClimateZone::Temperate.surface_frame(radius);
    let pal_y = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut sink_y = QuadSink::new();
    facet.emit_facet(GridCell::new(31, 0), &frame_y, &relief, half_blocks, &pal_y, &mut sink_y);

    let frame_x = ClimateZone::Desert.surface_frame(radius);
    let pal_x = ReliefPalette::for_zone(ClimateZone::Desert);
    let mut sink_x = QuadSink::new();
    facet.emit_facet(GridCell::new(0, 31), &frame_x, &relief, half_blocks, &pal_x, &mut sink_x);

    assert_eq!(sink_y.quads.len(), 1);
    assert_eq!(sink_x.quads.len(), 1);

    let y_verts = sink_y.quads[0].transfer(|v0, v1, v2, v3, _, _| [v0, v1, v2, v3]);
    let x_verts = sink_x.quads[0].transfer(|v0, v1, v2, v3, _, _| [v0, v1, v2, v3]);
    for v in y_verts.into_iter().chain(x_verts) {
        assert!((v.length() - radius).abs() < 1e-4, "Vertex must lie on sphere r={radius}, got {}", v.length());
    }

    let expected_seam_a = frame_y.point_at(radius, 0.0, 0.0);
    let expected_seam_b = frame_y.point_at(radius, 0.5, 0.0);
    // On Desert (+X): tangent = -Z, bitangent = +Y, so u in [-0.5, 0.0] maps to Z in [0.0, +0.5]!
    let mut sink_x_mirror = QuadSink::new();
    facet.emit_facet(GridCell::new(-1, 31), &frame_x, &relief, half_blocks, &pal_x, &mut sink_x_mirror);
    let xm_verts = sink_x_mirror.quads[0].transfer(|v0, v1, v2, v3, _, _| [v0, v1, v2, v3]);
    assert!(xm_verts.iter().any(|v| (*v - expected_seam_a).length() < 1e-4));
    assert!(xm_verts.iter().any(|v| (*v - expected_seam_b).length() < 1e-4));
}

#[test]
fn test_three_way_quad_sphere_corner_cells_converge_at_spherical_apex() {
    let facet = ReliefFacet::new();
    let relief = FlatRelief;
    let half_blocks = 32;
    let radius = (half_blocks as f32) * 0.5;

    let frame_y = ClimateZone::Temperate.surface_frame(radius);
    let pal_y = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut sink_y = QuadSink::new();
    facet.emit_facet(GridCell::new(31, 31), &frame_y, &relief, half_blocks, &pal_y, &mut sink_y);

    let frame_x = ClimateZone::Desert.surface_frame(radius);
    let pal_x = ReliefPalette::for_zone(ClimateZone::Desert);
    let mut sink_x = QuadSink::new();
    facet.emit_facet(GridCell::new(-32, 31), &frame_x, &relief, half_blocks, &pal_x, &mut sink_x);

    let frame_z = ClimateZone::Jungle.surface_frame(radius);
    let pal_z = ReliefPalette::for_zone(ClimateZone::Jungle);
    let mut sink_z = QuadSink::new();
    facet.emit_facet(GridCell::new(31, 31), &frame_z, &relief, half_blocks, &pal_z, &mut sink_z);

    let target_apex = Vec3::ONE.normalize() * radius;
    for (name, sink) in [("Y", &sink_y), ("X", &sink_x), ("Z", &sink_z)] {
        let touches = sink.quads.iter().any(|q| {
            q.transfer(|v0, v1, v2, v3, _, _| {
                [v0, v1, v2, v3].iter().any(|v| (*v - target_apex).length() < 1e-4)
            })
        });
        assert!(touches, "Shell {name} corner must converge at exact spherical apex {target_apex:?}");
    }
}

#[test]
fn test_meridian_border_cells_carry_distinct_climate_colors() {
    let facet = ReliefFacet::new();
    let relief = FlatRelief;
    let half_blocks = 32;
    let radius = (half_blocks as f32) * 0.5;

    let frame_y = ClimateZone::Temperate.surface_frame(radius);
    let pal_y = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut sink_y = QuadSink::new();
    facet.emit_facet(GridCell::new(31, 0), &frame_y, &relief, half_blocks, &pal_y, &mut sink_y);

    let frame_x = ClimateZone::Desert.surface_frame(radius);
    let pal_x = ReliefPalette::for_zone(ClimateZone::Desert);
    let mut sink_x = QuadSink::new();
    facet.emit_facet(GridCell::new(0, 31), &frame_x, &relief, half_blocks, &pal_x, &mut sink_x);

    let col_y = sink_y.quads[0].transfer(|_, _, _, _, _, c| c);
    let col_x = sink_x.quads[0].transfer(|_, _, _, _, _, c| c);
    assert_ne!(col_y, col_x);
}
