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

struct QuadLedger;

impl crate::MeshTarget for QuadLedger {
    fn push_quad(&mut self, _quad: crate::Quad) {}
}

#[test]
fn test_faceted_relief_face_quad_emission() {
    let face = FacetedReliefFace::in_zone(ClimateZone::Arctic, 8.0, 8);
    let mut ledger = QuadLedger;
    let mut counter = CountedMeshTarget::new(&mut ledger);
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
fn test_zoned_relief_direct_sampling() {
    use crate::noise::CoherentNoise;
    let frame = ClimateZone::Temperate.surface_frame(8.0);
    let relief = ZonedRelief::new(Box::new(CoherentNoise::new(123, 0.1)), frame.normal(), frame.tangent(), frame.bitangent(), 8.0, 0, 3);
    assert!((0..=3).contains(&relief.elevation_at(2, 2)));
    let mut collector = QuadLedger;
    ReliefStrata::new().emit_cell(crate::block::GridCell::new(2, 2), &frame, &relief, &ReliefPalette::for_zone(ClimateZone::Temperate), 8, &mut collector);
}

#[test]
fn test_deep_crust_excavation_and_indestructible_lethal_core_floor() {
    let mut deep = TerracedRelief::in_span(Box::new(TestRelief(|_, _| 0)), 64);
    for step in 1..=15 {
        let layer = -((step as f32) * 0.5) + 0.25;
        assert!(deep.excavate_layer(0, 0, layer).1, "Must excavate through mantle and first 3 crust layers (step {step})");
    }
    assert_eq!(deep.elevation_at(0, 0), -15);
    assert!(!deep.lethal_at(0.0, 0.0, -6.5), "4th crust layer at -15 (-7.5m) must shield avatar from lethal core floor");
    assert!(deep.excavate_layer(0, 0, -7.75).1, "Excavating 4th crust layer must expose the indestructible core floor at -16 (-8.0m)");
    assert_eq!(deep.elevation_at(0, 0), -16, "Column floor must rest on the indestructible core floor at -16");
    assert!(!deep.excavate_layer(0, 0, -7.75).1 && !deep.excavate_layer(0, 0, -8.25).1 && deep.mark_dig_layer(0, 0, -8.25, 1).1.is_none(), "Indestructible core floor at -16 must reject all further excavation");
    assert!(deep.lethal_at(0.0, 0.0, -7.0) && !deep.lethal_at(0.0, 0.0, -6.5), "Contact with the exposed core floor at -8.0m (waist at -7.0m) must be lethal");

    let frame = ClimateZone::Temperate.surface_frame(32.0);
    for step in 1..=12 {
        let layer = -((step as f32) * 0.5) + 0.25;
        assert!(deep.excavate_layer(2, 2, layer).1);
    }
    let up_eye = frame.point_at(1.25, 1.25, -5.5);
    let side_dir = (frame.tangent() + frame.normal() * 0.6).normalize();
    let (cu, cv, layer) = SurfaceSightRay::new(up_eye, side_dir).target_layer(&frame, &deep, 64, 32.0).expect("Must hit side cell from deep shaft");
    assert_eq!((cu, cv), (3, 2));
    assert!(deep.excavate_layer(cu, cv, layer).1 && deep.hit_top_at(1.75, 1.25, layer).is_none() && deep.hit_top_at(1.75, 1.25, layer + 0.5).is_some() && deep.hit_top_at(1.75, 1.25, layer - 0.5).is_some(), "Digging sideways from shaft must excavate only that single block layer");
    let (_, _, confined_z) = deep.confine(1.65, 1.25, layer);
    assert!((confined_z - layer).abs() < 0.35, "Lateral niche must keep avatar at niche elevation, got {confined_z}");
    assert!(deep.support_height_at(1.65, 1.25, layer - 1.0, 1.0) > -8.0, "Lateral niche must provide solid floor support");
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
    let mut collector = QuadLedger;
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
    let mut collector = QuadLedger;
    let mut counter = CountedMeshTarget::new(&mut collector);

    let palette = ReliefPalette::uniform(Vec3::ONE, Vec3::ZERO, Vec3::ZERO);

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
    let mut collector = QuadLedger;
    let mut counter = CountedMeshTarget::new(&mut collector);
    let palette = ReliefPalette::uniform(Vec3::ONE, Vec3::ZERO, Vec3::ZERO);

    // Border cell touching North seam (v1 = 16)
    let border_cell = crate::block::GridCell::new(0, 15);
    let border_relief = TestRelief(|_, _| 0);
    facet.emit_facet(border_cell, &frame, &border_relief, 16, &palette, &mut counter);
    assert_eq!(counter.count(), 2);

    // Grass regrowth: scales with adjacent grass count (8 neighbors ~40-95s, 1 neighbor 8x slower, 2-block-high pillar never regrows)
    let mut terraced = TerracedRelief::in_span(Box::new(TestRelief(|_, _| 0)), 8);
    assert!(terraced.stack_molded_cell(0, 0, 0.0, Box::new(crate::block::CuboidShape::new(0.5))) && terraced.bare_dirt_at(0, 0));
    assert!(
        terraced.stack_molded_cell(3, 3, 0.0, Box::new(crate::block::CuboidShape::new(0.5)))
            && terraced.stack_molded_cell(3, 3, 0.5, Box::new(crate::block::CuboidShape::new(0.5)))
            && terraced.bare_dirt_at(3, 3)
    );
    assert!(terraced.step_grass(15.0).is_empty() && terraced.bare_dirt_at(0, 0));
    assert!(!terraced.step_grass(30.0).is_empty() && !terraced.bare_dirt_at(0, 0), "Dirt block surrounded by 8 grass blocks within 1 block height must regrow grass in ~40-95s");
    assert!(terraced.step_grass(500.0).is_empty() && terraced.bare_dirt_at(3, 3), "Dirt block 2 blocks higher than neighboring grass must remain bare dirt");
}

#[test]
fn test_downward_shaft_excavation_through_mantle_and_crust_layers() {
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut downward = TerracedRelief::in_span(Box::new(TestRelief(|_, _| 0)), 64);

    for step in (0..16).rev() {
        let expected_mid = ((step - 16) as f32) * 0.5 + 0.25;
        let expected_med = if step < 4 { crate::block::BlockMedium::CrustRock } else { crate::block::BlockMedium::TurfEarth };
        let eye = frame.point_at(0.25, 0.25, expected_mid + 0.75);
        let (cu, cv, layer) = SurfaceSightRay::new(eye, -frame.normal()).target_layer(&frame, &downward, 64, 32.0).expect("Downward ray must hit top block");
        assert_eq!((cu, cv, layer), (0, 0, expected_mid));
        assert_eq!(downward.mark_dig_layer(0, 0, layer, 2).1, Some(expected_med));
        assert_eq!(downward.layer_dig_stage_at(0, 0, layer), 2);
        let mut ledger = QuadLedger;
        let mut counter = CountedMeshTarget::new(&mut ledger);
        ReliefStrata::new().emit_cell(crate::block::GridCell::new(0, 0), &frame, &downward, &palette, 64, &mut counter);
        assert!(counter.count() >= 9, "Excavated shaft floor at stage 2 must emit recessed 3D crater quads");
        assert_eq!(downward.harvest_layer(0, 0, layer).1, Some(expected_med));
    }
    assert_eq!(downward.elevation_at(0, 0), -16, "Excavating all 16 layers stops on the indestructible core floor at -16");
    assert!(downward.harvest_layer(0, 0, -8.25).1.is_none());
}

#[test]
fn test_close_wall_crosshair_targets_head_height_block_until_sight_line_crosses_lower_seam() {
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let mut hole = TerracedRelief::in_span(Box::new(TestRelief(|_, _| 0)), 64);
    for step in (12..16).rev() {
        let mid = ((step - 16) as f32) * 0.5 + 0.25;
        assert!(hole.excavate_layer(0, 0, mid).1);
    }
    assert_eq!(hole.elevation_at(0, 0), -4);
    assert_eq!(hole.elevation_at(1, 0), 0);

    // Standing on floor -2.0m with eye at -0.20m (head-height wall block spans [-0.50m, 0.00m], mid = -0.25m)
    let eye = frame.point_at(0.44, 0.25, -0.20);
    for target_z in [-0.20, -0.30, -0.42, -0.48] {
        let wall_pt = frame.point_at(0.50, 0.25, target_z);
        let ray = SurfaceSightRay::new(eye, (wall_pt - eye).normalize());
        assert_eq!(
            ray.target_layer(&frame, &hole, 64, 32.0),
            Some((1, 0, -0.25)),
            "Crosshair aimed at z={target_z} on the head-height block face [-0.50, 0.00] must target head-height layer -0.25"
        );
    }
    let lower_wall_pt = frame.point_at(0.50, 0.25, -0.58);
    let lower_ray = SurfaceSightRay::new(eye, (lower_wall_pt - eye).normalize());
    assert_eq!(
        lower_ray.target_layer(&frame, &hole, 64, 32.0),
        Some((1, 0, -0.75)),
        "Crosshair aimed below -0.50m onto the lower block face [-1.00, -0.50] must target lower layer -0.75"
    );
}

#[test]
fn test_unified_3d_block_model_supports_multi_story_surface_houses_and_settling_sand() {
    let mut house = TerracedRelief::in_span(Box::new(TestRelief(|_, _| 0)), 64);
    for step in [4, 5, 10, 11] {
        let base = (step as f32) * 0.5;
        assert!(house.stack_molded_cell(0, 0, base, Box::new(crate::block::CuboidShape::new(0.5))));
    }
    for step in 0..4 {
        assert!(house.stack_molded_cell(1, 0, (step as f32) * 0.5, Box::new(crate::block::CuboidShape::new(0.5))));
    }

    assert!((house.support_height_at(0.25, 0.25, 0.0, 1.80) - 0.0).abs() < 1e-4, "Ground story floor at 0.0m must remain passable under 2.0m ceiling");
    assert!((house.support_height_at(0.25, 0.25, 3.0, 1.80) - 3.0).abs() < 1e-4, "Upper story floor at 3.0m must support avatar under 5.0m roof");
    assert!((house.support_height_at(0.25, 0.25, 6.0, 1.80) - 6.0).abs() < 1e-4, "Roof terrace at 6.0m must support avatar above second story");

    let (clamped_u, _, clamped_z) = house.confine(0.45, 0.25, 1.95);
    assert!(clamped_u <= 0.30, "Multi-story surface wall at (1, 0) must clamp horizontal movement inside ground story, got u={clamped_u}");
    assert!(clamped_z <= 1.86, "Ground story ceiling at 2.0m must clamp vertical head elevation inside room, got z={clamped_z}");

    let sand_shape = Box::new(crate::block::MediumShape::new(
        Box::new(crate::block::CuboidShape::new(0.5)),
        crate::block::BlockMedium::Sand,
        true,
        false,
    ));
    assert!(house.stack_molded_cell(2, 0, 0.0, Box::new(crate::block::CuboidShape::new(0.5))));
    assert!(house.stack_molded_cell(2, 0, 0.5, sand_shape));
    assert!(house.excavate_layer(2, 0, 0.25).1, "Supporting dirt block under sand must be excavated");
    assert!(house.layer_occupied_at(2, 0, 0.75) && !house.layer_occupied_at(2, 0, 0.25));
    let settled = house.step_grass(0.05);
    assert!(settled.contains(&(2, 0)), "Unsupported sand block must settle downward on step_grass");
    assert!(house.layer_occupied_at(2, 0, 0.25) && !house.layer_occupied_at(2, 0, 0.75), "Sand block must rest on the 0.0..0.5m ground layer after settling");
}




