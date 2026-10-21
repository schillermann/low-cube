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
fn test_two_row_wedge_seam_contacts_flush_without_overlap() {
    let facet = ReliefFacet::new();
    let relief = FlatRelief;
    let half_blocks = 32;
    let half_size = (half_blocks as f32) * 0.5;

    // Face +Y (Temperate) border cell on East edge: u = 31, v = 0
    let frame_y = ClimateZone::Temperate.surface_frame(half_size);
    let pal_y = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut sink_y = QuadSink::new();
    facet.emit_facet(GridCell::new(31, 0), &frame_y, &relief, half_blocks, &pal_y, &mut sink_y);

    // Face +X (Desert) border cell on North edge: touching +Y at v = 31, u = 0
    let frame_x = ClimateZone::Desert.surface_frame(half_size);
    let pal_x = ReliefPalette::for_zone(ClimateZone::Desert);
    let mut sink_x = QuadSink::new();
    facet.emit_facet(GridCell::new(0, 31), &frame_x, &relief, half_blocks, &pal_x, &mut sink_x);

    assert_eq!(sink_y.quads.len(), 2, "Face Y border cell must emit 2 triangular quads");
    assert_eq!(sink_x.quads.len(), 2, "Face X border cell must emit 2 triangular quads");

    // Extract seam line vertices from Face +Y (outermost vertices at x = 15.75)
    let y_outer_verts: Vec<Vec3> = sink_y.quads.iter().flat_map(|q| {
        q.transfer(|v0, v1, v2, v3, _, _| vec![v0, v1, v2, v3])
    }).filter(|v| v.x > 15.6).collect();

    // Extract seam line vertices from Face +X (outermost vertices at y = 15.75)
    let x_outer_verts: Vec<Vec3> = sink_x.quads.iter().flat_map(|q| {
        q.transfer(|v0, v1, v2, v3, _, _| vec![v0, v1, v2, v3])
    }).filter(|v| v.y > 15.6).collect();

    assert!(!y_outer_verts.is_empty(), "Face +Y must produce seam vertices along x = 15.75");
    assert!(!x_outer_verts.is_empty(), "Face +X must produce seam vertices along y = 15.75");

    // All seam vertices on both sides must satisfy x == 15.75 and y == 15.75 (distance zero)
    for vy in &y_outer_verts {
        assert!((vy.x - 15.75).abs() < 1e-4, "Face +Y seam x must be 15.75, got {}", vy.x);
        assert!((vy.y - 15.75).abs() < 1e-4, "Face +Y seam y must be 15.75, got {}", vy.y);
    }
    for vx in &x_outer_verts {
        assert!((vx.x - 15.75).abs() < 1e-4, "Face +X seam x must be 15.75, got {}", vx.x);
        assert!((vx.y - 15.75).abs() < 1e-4, "Face +X seam y must be 15.75, got {}", vx.y);
    }

    // No vertex on Face +Y may penetrate deeper into Face +X (x must not exceed 15.75)
    let y_max_x = sink_y.quads.iter().flat_map(|q| {
        q.transfer(|v0, v1, v2, v3, _, _| vec![v0.x, v1.x, v2.x, v3.x])
    }).fold(f32::MIN, f32::max);
    assert!(y_max_x <= 15.7501, "Face +Y must never penetrate past x = 15.75, observed max x = {y_max_x}");

    // No vertex on Face +X may penetrate deeper into Face +Y (y must not exceed 15.75)
    let x_max_y = sink_x.quads.iter().flat_map(|q| {
        q.transfer(|v0, v1, v2, v3, _, _| vec![v0.y, v1.y, v2.y, v3.y])
    }).fold(f32::MIN, f32::max);
    assert!(x_max_y <= 15.7501, "Face +X must never penetrate past y = 15.75, observed max y = {x_max_y}");
}

#[test]
fn test_three_way_corner_caps_converge_at_exact_point() {
    let facet = ReliefFacet::new();
    let relief = FlatRelief;
    let half_blocks = 32;
    let half_size = (half_blocks as f32) * 0.5;

    // Corner cell (31, 31) on Face +Y (+X and +Z edges meet)
    let frame_y = ClimateZone::Temperate.surface_frame(half_size);
    let pal_y = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut sink_y = QuadSink::new();
    facet.emit_facet(GridCell::new(31, 31), &frame_y, &relief, half_blocks, &pal_y, &mut sink_y);

    // Corner cell (-32, 31) on Face +X (+Y and +Z edges meet: u0=-16 touches +Z, v1=16 touches +Y)
    let frame_x = ClimateZone::Desert.surface_frame(half_size);
    let pal_x = ReliefPalette::for_zone(ClimateZone::Desert);
    let mut sink_x = QuadSink::new();
    facet.emit_facet(GridCell::new(-32, 31), &frame_x, &relief, half_blocks, &pal_x, &mut sink_x);

    // Corner cell (31, 31) on Face +Z (+X and +Y edges meet)
    let frame_z = ClimateZone::Jungle.surface_frame(half_size);
    let pal_z = ReliefPalette::for_zone(ClimateZone::Jungle);
    let mut sink_z = QuadSink::new();
    facet.emit_facet(GridCell::new(31, 31), &frame_z, &relief, half_blocks, &pal_z, &mut sink_z);

    let target_corner = Vec3::new(15.75, 15.75, 15.75);

    let has_corner_y = sink_y.quads.iter().any(|q| {
        q.transfer(|v0, v1, v2, v3, _, _| {
            [v0, v1, v2, v3].iter().any(|v| (*v - target_corner).length() < 1e-4)
        })
    });
    let has_corner_x = sink_x.quads.iter().any(|q| {
        q.transfer(|v0, v1, v2, v3, _, _| {
            [v0, v1, v2, v3].iter().any(|v| (*v - target_corner).length() < 1e-4)
        })
    });
    let has_corner_z = sink_z.quads.iter().any(|q| {
        q.transfer(|v0, v1, v2, v3, _, _| {
            [v0, v1, v2, v3].iter().any(|v| (*v - target_corner).length() < 1e-4)
        })
    });

    assert!(has_corner_y, "Face +Y corner must terminate at exact point (15.75, 15.75, 15.75)");
    assert!(has_corner_x, "Face +X corner must terminate at exact point (15.75, 15.75, 15.75)");
    assert!(has_corner_z, "Face +Z corner must terminate at exact point (15.75, 15.75, 15.75)");
}

#[test]
fn test_seam_wedges_carry_distinct_climate_colors_per_side() {
    let facet = ReliefFacet::new();
    let relief = FlatRelief;
    let half_blocks = 32;
    let half_size = (half_blocks as f32) * 0.5;

    let frame_y = ClimateZone::Temperate.surface_frame(half_size);
    let pal_y = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut sink_y = QuadSink::new();
    facet.emit_facet(GridCell::new(31, 0), &frame_y, &relief, half_blocks, &pal_y, &mut sink_y);

    let frame_x = ClimateZone::Desert.surface_frame(half_size);
    let pal_x = ReliefPalette::for_zone(ClimateZone::Desert);
    let mut sink_x = QuadSink::new();
    facet.emit_facet(GridCell::new(0, 31), &frame_x, &relief, half_blocks, &pal_x, &mut sink_x);

    let col_y = sink_y.quads[0].transfer(|_, _, _, _, _, c| c);
    let col_x = sink_x.quads[0].transfer(|_, _, _, _, _, c| c);

    assert_ne!(col_y, col_x, "Each side of the seam wedge row must carry its own climate color");
    assert!((col_y - pal_y.inspect_surface(|s| s)).length() < 1e-4);
    assert!((col_x - pal_x.inspect_surface(|s| s)).length() < 1e-4);
}
