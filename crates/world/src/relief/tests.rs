use super::*;
use glam::Vec3;
use crate::topology::ClimateZone;
use crate::{CountedMeshTarget, Face};

#[test]
fn test_zoned_relief_bounds_and_continuity() {
    let face = FacetedReliefFace::in_zone(ClimateZone::Temperate, 8.0, 16);
    let sample_elev = face.constrain(Vec3::new(0.0, 9.0, 0.0), 1.0);
    assert!(sample_elev.y >= 9.0, "Elevation must be at or above base radius");
}

#[test]
fn test_relief_palette_biome_differentiation() {
    let temp_palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let volc_palette = ReliefPalette::for_zone(ClimateZone::Volcanic);

    let temp_surf = temp_palette.inspect_surface(|s| s);
    let volc_surf = volc_palette.inspect_surface(|s| s);

    assert_ne!(temp_surf, volc_surf, "Different biomes must have distinct palettes");
}

#[test]
fn test_tron_palette_distinct_colors_for_facets_and_mismatches() {
    let tron = ReliefPalette::tron();
    let (surf, ramp, ridge, valley, chamfer, cliff) =
        tron.inspect_facets(|s, r, rd, v, ch, cl| (s, r, rd, v, ch, cl));

    assert_ne!(surf, ramp, "Cube flat and ramp must have distinct colors");
    assert_ne!(ramp, ridge, "Ramp and ridge corner must have distinct colors");
    assert_ne!(ridge, valley, "Ridge and valley corner must have distinct colors");
    assert_ne!(cliff, surf, "Vertical cliff drop must have warning color distinct from surface");
    assert_eq!(cliff, Vec3::new(1.0, 0.1, 0.1), "Mismatched cliff walls must glow warning red");
    assert_eq!(chamfer, Vec3::new(1.0, 1.0, 1.0), "Planet seam chamfers must be white");
}

struct QuadCollector;

impl crate::MeshTarget for QuadCollector {
    fn push_quad(&mut self, _quad: crate::Quad) {}
}

#[test]
fn test_faceted_relief_face_quad_emission() {
    let face = FacetedReliefFace::in_zone(ClimateZone::Arctic, 8.0, 8);
    let mut collector = QuadCollector;
    let mut counter = CountedMeshTarget::new(&mut collector);
    face.emit_surface(&mut counter);

    assert!(counter.count() > 0, "Relief face must emit faceted surface and cave quads");
}

#[test]
fn test_subterranean_cavity_depth_constraint() {
    let face = FacetedReliefFace::in_zone(ClimateZone::Jungle, 8.0, 16);
    let deep_point = Vec3::new(0.0, 7.0, 0.0);
    let constrained = face.constrain(deep_point, 1.0);
    assert!(constrained.y.is_finite());
}

#[test]
fn test_zoned_cavity_and_relief_direct_sampling() {
    use crate::noise::CoherentNoise;
    let frame = ClimateZone::Temperate.surface_frame(8.0);
    let relief = ZonedRelief::new(Box::new(CoherentNoise::new(123, 0.1)), frame.normal(), frame.tangent(), frame.bitangent(), 8.0, 0, 3);
    assert!((0..=3).contains(&relief.elevation_at(2, 2)));
    let cavity = ZonedCavity::new(Box::new(CoherentNoise::new(456, 0.1)), frame.normal(), frame.tangent(), frame.bitangent(), 8.0, 0.3);
    let depth = cavity.constrain_depth(2.0, 2.0, -1.0, 1.0);
    assert!(depth == -2.0 || depth == 1.0);
    let mut collector = QuadCollector;
    ReliefStrata::new().emit_cell(crate::block::GridCell::new(2, 2), &frame, &relief, &cavity, &ReliefPalette::for_zone(ClimateZone::Temperate), 8, &mut collector);
}

#[test]
fn test_deep_crust_excavation_hollow_world_breach_and_indestructible_magma_core() {
    let mut deep = TerracedRelief::in_span(Box::new(TestRelief(|_, _| 0)), 64);
    for step in 1..=15 {
        assert!(deep.excavate_cell(0, 0).1, "Must excavate through mantle and first 3 crust layers (step {step})");
    }
    assert_eq!(deep.elevation_at(0, 0), -15);
    assert!(!deep.breached_at(0, 0));
    assert!(deep.excavate_cell(0, 0).1, "Excavating 4th crust layer above the vault must breach into hollow world");
    assert!(deep.breached_at(0, 0) && deep.elevation_at(0, 0) == -32);
    assert!(!deep.excavate_cell(0, 0).1 && !deep.mark_dig_stage(0, 0, 1).1, "Magma core summit at -32 must be indestructible");
    assert!(deep.weightless_at(0.0, 0.0, -13.0) && !deep.lethal_at(0.0, 0.0, -13.0), "Hollow world cavity must be weightless");
    assert!(deep.lethal_at(0.0, 0.0, -15.5) && !deep.weightless_at(0.0, 0.0, -15.5), "Magma core surface must be lethal");
    assert!(deep.weightless_at(14.0, 0.0, -16.0) && !deep.weightless_at(24.0, 24.0, -9.0), "Hollow beside the core floats while the rounded crust beyond the vault stays solid");

    let frame = ClimateZone::Temperate.surface_frame(32.0);
    for _ in 0..5 { assert!(deep.excavate_layer(2, 2, -9.0).1); }
    assert_eq!(deep.crust_floor_at(2, 2), -11, "Digging upward from core must raise upward shaft ceiling");
    let up_eye = frame.point_at(1.25, 1.25, -6.0);
    let side_dir = (frame.tangent() + frame.normal() * 0.6).normalize();
    let (cu, cv, layer) = SurfaceSightRay::new(up_eye, side_dir).target_layer(&frame, &deep, 64, 32.0).expect("Must hit side cell from upward shaft");
    assert_eq!((cu, cv), (3, 2));
    assert!(deep.excavate_layer(cu, cv, layer).1 && deep.crust_floor_at(3, 2) > -16, "Digging sideways from upward shaft must open adjacent cell upward");
    assert!(deep.excavate_layer(1, 0, -2.25).1 && deep.elevation_at(1, 0) == -5 && deep.hit_top_at(0.75, 0.25, -2.25).is_none());
    assert!(deep.excavate_layer(1, 0, -1.75).1 && deep.hit_top_at(0.75, 0.25, -1.75).is_none(), "Mining next side block at -2.0..-1.5 must open only that layer");
    for kept in [-1.25, -0.75, -0.25] { assert!(deep.hit_top_at(0.75, 0.25, kept).is_some(), "Block at {kept} above mined side blocks must remain intact"); }
}


#[test]
fn test_scale_64_mountain_elevation_reaches_authentic_height() {
    let face = FacetedReliefFace::in_zone(ClimateZone::Volcanic, 32.0, 64);
    let mut max_observed_h: f32 = 0.0;
    for u in -15..15 {
        for v in -15..15 {
            let sample = face.constrain(Vec3::new(u as f32, 33.0, v as f32), 1.0);
            let elev = (sample.y.abs() - 32.0 - 1.0).max(0.0);
            if elev > max_observed_h {
                max_observed_h = elev;
            }
        }
    }
    assert!(
        max_observed_h >= 3.0,
        "Scale 64 volcanic face must generate mountain heights of at least 6 half-meter blocks (3.0m), observed {max_observed_h}"
    );
}

#[test]
fn test_watertight_surface_emission_always_emits_top_quad() {
    let face = FacetedReliefFace::in_zone(ClimateZone::Arctic, 32.0, 16);
    let mut collector = QuadCollector;
    let mut counter = CountedMeshTarget::new(&mut collector);
    face.emit_surface(&mut counter);
    // 16x16 grid must emit at least 256 top quads plus side cliff walls
    assert!(
        counter.count() >= 256,
        "Every cell must emit surface quads without holes, got {}",
        counter.count()
    );
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
fn test_beveled_slope_and_corner_emission() {
    let frame = ClimateZone::Temperate.surface_frame(16.0);
    let facet = ReliefFacet::new();
    let mut collector = QuadCollector;
    let mut counter = CountedMeshTarget::new(&mut collector);

    let palette = ReliefPalette::uniform(Vec3::ONE, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO);

    // 1-block drop to North: emits 45° ramp quad
    let cell_ramp = crate::block::GridCell::new(0, 0);
    let ramp_relief = TestRelief(|u, v| if (u, v) == (0, 1) { 1 } else { 2 });
    facet.emit_facet(cell_ramp, &frame, &ramp_relief, 16, &palette, &mut counter);
    assert_eq!(counter.count(), 1);

    // 1-block drop to North and East: emits 2 triangular corner quads
    let cell_corner = crate::block::GridCell::new(1, 0);
    let corner_relief = TestRelief(|u, v| if (u, v) == (1, 1) || (u, v) == (2, 0) { 1 } else { 2 });
    facet.emit_facet(cell_corner, &frame, &corner_relief, 16, &palette, &mut counter);
    assert_eq!(counter.count(), 3);

    // 1-block lower diagonal at South-East: replaces flat block with 2 inward slope quads
    let cell_valley = crate::block::GridCell::new(2, 0);
    let valley_relief = TestRelief(|u, v| if (u, v) == (3, -1) { 1 } else { 2 });
    facet.emit_facet(cell_valley, &frame, &valley_relief, 16, &palette, &mut counter);
    assert_eq!(counter.count(), 5);
}

#[test]
fn test_seam_chamfer_emission_at_world_border() {
    let frame = ClimateZone::Temperate.surface_frame(16.0);
    let facet = ReliefFacet::new();
    let mut collector = QuadCollector;
    let mut counter = CountedMeshTarget::new(&mut collector);
    let palette = ReliefPalette::uniform(Vec3::ONE, Vec3::ZERO, Vec3::ZERO, Vec3::ZERO);

    // Border cell touching North seam (v1 = 16)
    let border_cell = crate::block::GridCell::new(0, 15);
    let border_relief = TestRelief(|_, _| 0);
    facet.emit_facet(border_cell, &frame, &border_relief, 16, &palette, &mut counter);
    assert_eq!(counter.count(), 2);

    // Grass regrowth: scales with adjacent grass count (8 neighbors ~40-95s, 1 neighbor 8x slower, 2-block-high pillar never regrows)
    let mut terraced = TerracedRelief::in_span(Box::new(TestRelief(|_, _| 0)), 8);
    assert!(terraced.stack_cell(0, 0) && terraced.bare_dirt_at(0, 0));
    assert!(terraced.stack_cell(3, 3) && terraced.stack_cell(3, 3) && terraced.bare_dirt_at(3, 3));
    assert!(terraced.step_grass(15.0).is_empty() && terraced.bare_dirt_at(0, 0));
    assert!(!terraced.step_grass(30.0).is_empty() && !terraced.bare_dirt_at(0, 0), "Dirt block surrounded by 8 grass blocks within 1 block height must regrow grass in ~40-95s");
    assert!(terraced.step_grass(500.0).is_empty() && terraced.bare_dirt_at(3, 3), "Dirt block 2 blocks higher than neighboring grass must remain bare dirt");
}


