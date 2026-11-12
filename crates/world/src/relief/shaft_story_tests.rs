use super::*;
use crate::topology::ClimateZone;
use crate::CountedMeshTarget;

struct QuadLedger;

impl crate::MeshTarget for QuadLedger {
    fn push_quad(&mut self, _quad: crate::Quad) {}
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
fn test_downward_shaft_excavation_through_mantle_and_crust_layers() {
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut downward = TerracedRelief::in_span(Box::new(FlatRelief), 64);

    for step in (0..16).rev() {
        let expected_mid = ((step - 16) as f32) * 0.5 + 0.25;
        let expected_med = if step < 4 {
            crate::block::BlockMedium::CrustRock
        } else {
            crate::block::BlockMedium::TurfEarth
        };
        let eye = frame.point_at(0.25, 0.25, expected_mid + 0.75);
        let (cu, cv, layer) = SurfaceSightRay::new(eye, -frame.normal())
            .target_layer(&frame, &downward, 64, 32.0)
            .expect("Downward ray must hit top block");
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
    let mut hole = TerracedRelief::in_span(Box::new(FlatRelief), 64);
    for step in (12..16).rev() {
        let mid = ((step - 16) as f32) * 0.5 + 0.25;
        assert!(hole.excavate_layer(0, 0, mid).1);
    }
    assert_eq!(hole.elevation_at(0, 0), -4);
    assert_eq!(hole.elevation_at(1, 0), 0);

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
    let mut house = TerracedRelief::in_span(Box::new(FlatRelief), 64);
    for step in [4, 5, 10, 11] {
        let base = (step as f32) * 0.5;
        assert!(house.stack_molded_cell(0, 0, base, Box::new(crate::block::CuboidShape::new(0.5))));
    }
    for step in 0..4 {
        assert!(house.stack_molded_cell(1, 0, (step as f32) * 0.5, Box::new(crate::block::CuboidShape::new(0.5))));
    }

    assert!((house.support_height_at(0.25, 0.25, 0.0, 1.80) - 0.0).abs() < 1e-4);
    assert!((house.support_height_at(0.25, 0.25, 3.0, 1.80) - 3.0).abs() < 1e-4);
    assert!((house.support_height_at(0.25, 0.25, 6.0, 1.80) - 6.0).abs() < 1e-4);

    let (clamped_u, _, clamped_z) = house.confine(0.45, 0.25, 1.95);
    assert!(clamped_u <= 0.30, "Multi-story surface wall at (1, 0) must clamp horizontal movement, got u={clamped_u}");
    assert!(clamped_z <= 1.86, "Ground story ceiling at 2.0m must clamp vertical head elevation, got z={clamped_z}");

    let sand_shape = Box::new(crate::block::MediumShape::new(
        Box::new(crate::block::CuboidShape::new(0.5)),
        crate::block::BlockMedium::Sand,
        true,
        false,
    ));
    assert!(house.stack_molded_cell(2, 0, 0.0, Box::new(crate::block::CuboidShape::new(0.5))));
    assert!(house.stack_molded_cell(2, 0, 0.5, sand_shape));
    assert!(house.excavate_layer(2, 0, 0.25).1);
    assert!(house.layer_occupied_at(2, 0, 0.75) && !house.layer_occupied_at(2, 0, 0.25));
    let settled = house.step_grass(0.05);
    assert!(settled.contains(&(2, 0)));
    assert!(house.layer_occupied_at(2, 0, 0.25) && !house.layer_occupied_at(2, 0, 0.75));
}
