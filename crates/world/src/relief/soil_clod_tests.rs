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
}

impl MeshTarget for ToneLedger {
    fn push_quad(&mut self, quad: Quad) { quad.transfer(|_, _, _, _, _, color| self.tones.push(color)); }
}

fn hand_block_tones() -> Vec<Vec3> {
    let mut ledger = ToneLedger { tones: Vec::new() };
    EarthClods::new(GridCell::new(0, 0), 0.0).emit_block(&ClimateZone::Temperate.surface_frame(32.0), &mut ledger);
    ledger.tones
}

#[test]
fn test_excavated_terrain_soil_wears_the_hand_held_earth_block_clods() {
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);
    for _ in 0..3 { assert!(relief.excavate_cell(0, 0).1); }
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let (mut pit, mut rim) = (ToneLedger { tones: Vec::new() }, ToneLedger { tones: Vec::new() });
    ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &relief, &super::NullCavity, &palette, 64, &mut pit);
    ReliefStrata::new().emit_cell(GridCell::new(1, 0), &frame, &relief, &super::NullCavity, &palette, 64, &mut rim);
    let hand = hand_block_tones();
    let pebble = Vec3::new(0.64, 0.61, 0.55);
    assert!(hand.contains(&pebble) && pit.tones.contains(&pebble), "Bare excavated pit floor must show the hand block's clods and pebbles");
    assert!(pit.tones.contains(&Vec3::new(0.45, 0.30, 0.17)), "Bare pit floor must share the hand block's loam top tone");
    let rim_clods = rim.tones.iter().filter(|&&c| c == pebble).count();
    assert!(rim_clods >= 3, "Each exposed 0.5 m soil layer of the pit wall must carry hand block flank clods, got {rim_clods}");
}
