use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::{EarthClods, GridCell};
use crate::topology::ClimateZone;
use super::{ReliefElevation, ReliefPalette, ReliefStrata, TerracedRelief};

struct FlatMeadow;

impl ReliefElevation for FlatMeadow {
    fn natural_elevation_at(&self, _u: i32, _v: i32) -> i32 { 0 }
    fn elevation_at(&self, _u: i32, _v: i32) -> i32 { 0 }
    fn surface_height_at(&self, _u: f32, _v: f32) -> f32 { 0.0 }
}

struct ToneLedger {
    tones: Vec<Vec3>,
    down_pebbles: usize,
}

impl MeshTarget for ToneLedger {
    fn push_quad(&mut self, quad: Quad) {
        quad.transfer(|_, _, _, _, norm, color| {
            self.tones.push(color);
            if norm.y < -0.9 && color == Vec3::new(0.64, 0.61, 0.55) {
                self.down_pebbles += 1;
            }
        });
    }
}

fn hand_block_tones() -> Vec<Vec3> {
    let mut ledger = ToneLedger { tones: Vec::new(), down_pebbles: 0 };
    EarthClods::new(GridCell::new(0, 0), 0.0)
        .emit_terraced(Vec3::new(0.45, 0.30, 0.17), true, &ClimateZone::Temperate.surface_frame(32.0), &mut ledger);
    ledger.tones
}

#[test]
fn test_excavated_terrain_soil_wears_the_hand_held_earth_block_clods() {
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);
    for _ in 0..3 { assert!(relief.excavate_cell(0, 0).1); }
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let (mut pit, mut rim) = (ToneLedger { tones: Vec::new(), down_pebbles: 0 }, ToneLedger { tones: Vec::new(), down_pebbles: 0 });
    ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &relief, &super::NullCavity, &palette, 64, &mut pit);
    ReliefStrata::new().emit_cell(GridCell::new(1, 0), &frame, &relief, &super::NullCavity, &palette, 64, &mut rim);
    let hand = hand_block_tones();
    let pebble = Vec3::new(0.64, 0.61, 0.55);
    assert!(hand.contains(&pebble) && pit.tones.contains(&pebble), "Bare excavated pit floor must show the hand block's clods and pebbles");
    assert!(pit.tones.contains(&Vec3::new(0.45, 0.30, 0.17)), "Bare pit floor must share the hand block's loam top tone");
    let rim_clods = rim.tones.iter().filter(|&&c| c == pebble).count();
    assert!(rim_clods >= 3, "Each exposed 0.5 m soil layer of the pit wall must carry hand block flank clods, got {rim_clods}");
}

#[test]
fn test_sideways_excavated_dirt_preserves_overhang_clod_structure() {
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);
    for _ in 0..7 { assert!(relief.excavate_cell(0, 0).1); }
    assert!(relief.excavate_layer(1, 0, -1.25).1, "Must excavate side block below column top");
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut overhung = ToneLedger { tones: Vec::new(), down_pebbles: 0 };
    ReliefStrata::new().emit_cell(GridCell::new(1, 0), &frame, &relief, &super::NullCavity, &palette, 64, &mut overhung);
    let pebble = Vec3::new(0.64, 0.61, 0.55);
    let overhang_pebbles = overhung.tones.iter().filter(|&&c| c == pebble).count();
    assert!(overhang_pebbles >= 50, "Blocks above a sideways-dug dirt block must retain their full 3D clod structure, got {overhang_pebbles}");
    assert!(overhung.down_pebbles >= 2, "Underside of overhanging dirt block must have downward-facing clod/pebble structure, got {}", overhung.down_pebbles);
}

#[test]
fn test_sight_ray_targets_all_four_side_wall_layers_below_head_height() {
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);
    for _ in 0..7 { assert!(relief.excavate_cell(0, 0).1); }
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let eye = frame.point_at(0.35, 0.25, -3.5 + 1.80);
    for (pitch_deg, expected_layer) in [(-10.0f32, -1.75f32), (-38.0, -2.25), (-58.0, -2.75), (-72.0, -3.25)] {
        let rad = pitch_deg.to_radians();
        let dir = frame.tangent() * rad.cos() + frame.normal() * rad.sin();
        let hit = super::SurfaceSightRay::new(eye, dir).target_layer(&frame, &relief, 64, 32.0);
        assert_eq!(hit, Some((1, 0, expected_layer)), "Pitch {pitch_deg} deg must target side wall block {expected_layer} instead of floor, got {hit:?}");
        assert!(relief.excavate_layer(1, 0, expected_layer).1);
    }
}

#[test]
fn test_earth_crust_side_walls_never_emit_dirt_clods() {
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);
    while !relief.breached_at(0, 0) {
        assert!(relief.excavate_cell(0, 0).1);
    }
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let vault = super::CrustVault::beneath(64);
    let crust_top_y = ((vault.floor_steps_at(1, 0) + 4) as f32) * 0.5;
    struct CrustWallProbe { crust_pebbles: usize, quartz_veins: usize, upper_pebbles: usize, top_y: f32 }
    impl MeshTarget for CrustWallProbe {
        fn push_quad(&mut self, quad: Quad) {
            let glow = quad.radiate(|g| g);
            quad.transfer(|v0, v1, v2, v3, _, color| {
                let cy = (v0.y + v1.y + v2.y + v3.y) * 0.25 - 32.0;
                if cy < self.top_y - 0.02 {
                    if color == Vec3::new(0.64, 0.61, 0.55) || color == Vec3::new(0.41, 0.27, 0.15) {
                        self.crust_pebbles += 1;
                    } else if glow >= 0.54 && color.x > 0.60 && color.y > 0.35 && color.z < 0.45 {
                        self.quartz_veins += 1;
                    }
                } else if color == Vec3::new(0.64, 0.61, 0.55) {
                    self.upper_pebbles += 1;
                }
            });
        }
    }
    let mut probe = CrustWallProbe { crust_pebbles: 0, quartz_veins: 0, upper_pebbles: 0, top_y: crust_top_y };
    ReliefStrata::new().emit_cell(GridCell::new(1, 0), &frame, &relief, &super::NullCavity, &palette, 64, &mut probe);
    assert_eq!(probe.crust_pebbles, 0, "4-layer planetary crust walls must never carry dirt color or soil clods");
    assert!(probe.quartz_veins >= 60, "4-layer planetary crust walls must emit Variant 2A 3D glowing quartz/amber mineral veins, got {}", probe.quartz_veins);
    assert!(probe.upper_pebbles > 0, "Soil layers above the 4-layer crust must retain soil clods");
}
