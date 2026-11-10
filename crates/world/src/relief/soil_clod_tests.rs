use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::{BlockMedium, BlockMold, CardinalTurn, EarthClods, GridCell, MoldArchetype};
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
    for step in 0..3 { assert!(relief.excavate_layer(0, 0, -((step as f32) * 0.5) - 0.25).1); }
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let (mut pit, mut rim) = (ToneLedger { tones: Vec::new(), down_pebbles: 0 }, ToneLedger { tones: Vec::new(), down_pebbles: 0 });
    ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &relief, &palette, 64, &mut pit);
    ReliefStrata::new().emit_cell(GridCell::new(1, 0), &frame, &relief, &palette, 64, &mut rim);
    let hand = hand_block_tones();
    let pebble = Vec3::new(0.64, 0.61, 0.55);
    assert!(hand.contains(&pebble) && pit.tones.contains(&pebble), "Bare excavated pit floor must show the hand block's clods and pebbles");
    assert!(pit.tones.contains(&Vec3::new(0.45, 0.30, 0.17)), "Bare pit floor must share the hand block's loam top tone");
    let rim_clods = rim.tones.iter().filter(|&&c| c == pebble).count();
    assert!(rim_clods >= 3, "Each exposed 0.5 m soil layer of the pit wall must carry hand block flank clods, got {rim_clods}");
    for code in 0..16 {
        let mut form_ledger = ToneLedger { tones: Vec::new(), down_pebbles: 0 };
        BlockMold::with_medium(MoldArchetype::from_code(code), CardinalTurn::Forward, BlockMedium::TurfEarth)
            .emit_miniature(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z, 1.0, Vec3::new(0.45, 0.30, 0.17), Vec3::new(0.45, 0.30, 0.17), &mut form_ledger);
        assert!(form_ledger.tones.contains(&pebble), "Mold form {code} must carry full-cube dirt clods and pebbles");
        assert!(form_ledger.tones.contains(&Vec3::new(0.41, 0.27, 0.15)), "Mold form {code} must carry full-cube dirt loam walls");
        assert!(!form_ledger.tones.iter().any(|c| c.y > c.x + 0.05), "Handheld dirt form {code} must have no green grass");
    }
}

#[test]
fn test_sideways_excavated_dirt_preserves_overhang_clod_structure() {
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);
    for step in 0..7 { assert!(relief.excavate_layer(0, 0, -((step as f32) * 0.5) - 0.25).1); }
    assert!(relief.excavate_layer(1, 0, -1.25).1, "Must excavate side block below column top");
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut overhung = ToneLedger { tones: Vec::new(), down_pebbles: 0 };
    ReliefStrata::new().emit_cell(GridCell::new(1, 0), &frame, &relief, &palette, 64, &mut overhung);
    let pebble = Vec3::new(0.64, 0.61, 0.55);
    let overhang_pebbles = overhung.tones.iter().filter(|&&c| c == pebble).count();
    assert!(overhang_pebbles >= 50, "Blocks above a sideways-dug dirt block must retain their full 3D clod structure, got {overhang_pebbles}");
    assert!(overhung.down_pebbles >= 2, "Underside of overhanging dirt block must have downward-facing clod/pebble structure, got {}", overhung.down_pebbles);
}

#[test]
fn test_sight_ray_targets_all_four_side_wall_layers_below_head_height() {
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);
    for step in 0..7 { assert!(relief.excavate_layer(0, 0, -((step as f32) * 0.5) - 0.25).1); }
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
    while relief.elevation_at(0, 0) > relief.crust_floor_at(0, 0) {
        let top_layer = (relief.elevation_at(0, 0) as f32) * 0.5 - 0.25;
        assert!(relief.excavate_layer(0, 0, top_layer).1);
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
    ReliefStrata::new().emit_cell(GridCell::new(1, 0), &frame, &relief, &palette, 64, &mut probe);
    assert_eq!(probe.crust_pebbles, 0, "4-layer planetary crust walls must never carry dirt color or soil clods");
    assert!(probe.quartz_veins >= 60, "4-layer planetary crust walls must emit Variant 2A 3D glowing quartz/amber mineral veins, got {}", probe.quartz_veins);
    assert!(probe.upper_pebbles > 0, "Soil layers above the 4-layer crust must retain soil clods");

    let crust_top_step = vault.floor_steps_at(1, 0) + 4;
    while relief.elevation_at(1, 0) > crust_top_step {
        let top_layer = (relief.elevation_at(1, 0) as f32) * 0.5 - 0.25;
        assert!(relief.excavate_layer(1, 0, top_layer).1);
    }
    let mut top_idle = ToneLedger { tones: Vec::new(), down_pebbles: 0 };
    ReliefStrata::new().emit_cell(GridCell::new(1, 0), &frame, &relief, &palette, 64, &mut top_idle);
    assert!(
        !top_idle.tones.contains(&Vec3::new(0.45, 0.30, 0.17)) && !top_idle.tones.contains(&Vec3::new(0.64, 0.61, 0.55)),
        "Excavating the dirt block atop the highest crust block must leave no dirt layer or soil clods on top"
    );
    assert!(top_idle.tones.contains(&Vec3::new(0.35, 0.31, 0.28)), "Top of exposed crust block must show crust rock tone");

    let crust_layer_elev = (crust_top_step as f32) * 0.5 - 0.25;
    assert_eq!(
        relief.mark_dig_layer(1, 0, crust_layer_elev, 2).1,
        Some(crate::block::BlockMedium::CrustRock),
        "Exposed crust block must report CrustRock medium for rock debris particles"
    );
    let mut top_dug = ToneLedger { tones: Vec::new(), down_pebbles: 0 };
    ReliefStrata::new().emit_cell(GridCell::new(1, 0), &frame, &relief, &palette, 64, &mut top_dug);
    assert!(
        !top_dug.tones.contains(&Vec3::new(0.28, 0.19, 0.11)) && !top_dug.tones.contains(&Vec3::new(0.42, 0.29, 0.17)),
        "Digging a crust block from above must not emit brown dirt crater tones"
    );
    assert!(
        top_dug.tones.iter().any(|c| (*c - Vec3::new(0.35, 0.31, 0.28) * 0.52).length() < 1e-3),
        "Digging a crust block from above must recess into dark crust rock tones matching side excavation"
    );
    assert_eq!(relief.harvest_layer(1, 0, crust_layer_elev).1, Some(crate::block::BlockMedium::CrustRock));
    let placed_base = (relief.elevation_at(1, 0) as f32) * 0.5;
    assert!(relief.stack_molded_cell(1, 0, placed_base, Box::new(crate::block::CuboidShape::new(0.5))));
    assert_eq!(
        relief.mark_dig_layer(1, 0, crust_layer_elev, 1).1,
        Some(crate::block::BlockMedium::TurfEarth),
        "Dirt block placed inside crust cavity must preserve TurfEarth identity instead of mutating into CrustRock"
    );
}

#[test]
fn test_mining_blocks_never_expands_geometry_outward() {
    struct SpanBounds { min: Vec3, max: Vec3, recessed_sides: [usize; 4] }
    impl SpanBounds {
        fn empty() -> Self { Self { min: Vec3::splat(f32::MAX), max: Vec3::splat(f32::MIN), recessed_sides: [0; 4] } }
    }
    impl MeshTarget for SpanBounds {
        fn push_quad(&mut self, quad: Quad) {
            quad.transfer(|v0, v1, v2, v3, norm, _| {
                for v in [v0, v1, v2, v3] {
                    self.min = self.min.min(v);
                    self.max = self.max.max(v);
                }
                let c = (v0 + v1 + v2 + v3) * 0.25;
                if c.y > 32.05 && c.y < 32.45 {
                    if norm.x > 0.9 && c.x < 0.48 { self.recessed_sides[0] += 1; }
                    if norm.x < -0.9 && c.x > 0.02 { self.recessed_sides[1] += 1; }
                    if norm.z > 0.9 && c.z < 0.48 { self.recessed_sides[2] += 1; }
                    if norm.z < -0.9 && c.z > 0.02 { self.recessed_sides[3] += 1; }
                }
            });
        }
    }
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);
    assert!(relief.stack_molded_cell(0, 0, 0.0, Box::new(crate::block::CuboidShape::new(0.5))));
    let mut idle = SpanBounds::empty();
    ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &relief, &palette, 64, &mut idle);
    for stage in 1..=3 {
        relief.mark_dig_layer(0, 0, 0.25, stage);
        let mut dug = SpanBounds::empty();
        ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &relief, &palette, 64, &mut dug);
        assert!(
            !dug.min.cmplt(idle.min - Vec3::splat(1e-4)).any() && !dug.max.cmpgt(idle.max + Vec3::splat(1e-4)).any(),
            "Stage {stage} digging must never expand block bounds outward: idle=({:?}..{:?}), dug=({:?}..{:?})",
            idle.min, idle.max, dug.min, dug.max
        );
        assert!(
            dug.recessed_sides.iter().all(|&n| n >= 1),
            "Stage {stage} digging must emit recessed crater floors on all 4 side walls, got {:?}",
            dug.recessed_sides
        );
    }
    assert!(relief.excavate_layer(0, 0, 0.25).1 && relief.excavate_layer(1, 0, -0.25).1);
    relief.mark_dig_layer(0, 0, -0.25, 2);
    let mut wall_dug = SpanBounds::empty();
    ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &relief, &palette, 64, &mut wall_dug);
    let has_recessed_east_wall = wall_dug.min.x < 0.45;
    assert!(has_recessed_east_wall, "Terrain cell side wall must also recess inward when mined");
}

#[test]
fn test_polyhedral_cantilever_rig_deploys_retracts_telescopic_gripper_and_hoists_vehicle() {
    let frame = ClimateZone::Temperate.surface_frame(32.0);
    let palette = ReliefPalette::for_zone(ClimateZone::Temperate);
    let mut relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);

    let rig_shape = Box::new(crate::block::MediumShape::new(Box::new(crate::block::CuboidShape::new(0.5)), crate::block::BlockMedium::CantileverRig, true, false));
    assert!(relief.stack_molded_cell(0, 0, 0.0, rig_shape), "Cantilever rig must deploy on ground cell (0, 0)");
    assert!(!relief.layer_occupied_at(2, 0, 1.75), "Deploying rig must NOT spawn an engine block automatically");
    let engine_shape = Box::new(crate::block::MediumShape::new(Box::new(crate::block::CuboidShape::new(0.5)), crate::block::BlockMedium::SteeringWheel, true, false));
    let (claw_eye, claw_dir) = (frame.point_at(1.25, -1.25, 1.75), frame.bitangent());
    assert_eq!(
        super::SurfaceSightRay::new(claw_eye, claw_dir).target_layer(&frame, &relief, 64, 32.0),
        Some((2, 0, 1.75)),
        "Aiming directly at the suspended crane claw in mid-air must intercept the crane boom at (2, 0, 1.75)"
    );
    assert!(!relief.stack_molded_cell(0, 0, 0.0, Box::new(crate::block::CuboidShape::new(0.5))), "Non-SteeringWheel blocks must be rejected on an empty crane");
    assert!(relief.stack_molded_cell(0, 0, 0.0, engine_shape), "Mounting SteeringWheel onto crane must succeed");
    assert!(relief.layer_occupied_at(2, 0, 1.75), "Mounted SteeringWheel must be suspended at (2, 0) height 1.5m..2.0m");
    let mut rig_ledger = ToneLedger { tones: Vec::new(), down_pebbles: 0 };
    ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &relief, &palette, 64, &mut rig_ledger);
    assert!(rig_ledger.tones.len() >= 40, "Polyhedral cantilever rig must emit sculpted 3D mast, rack teeth, collar, and crossbar quads");
    assert!(
        relief.stack_molded_cell(1, 0, 1.5, Box::new(crate::block::CuboidShape::new(0.5))),
        "Player must be able to build onto the gripped side face at (1, 0) height 1.5m"
    );
    let wheel_shape = crate::block::BlockMold::cube()
        .in_medium(crate::block::BlockMedium::CarWheel)
        .shape_for(frame.tangent(), &frame);
    assert!(relief.stack_molded_cell(2, 1, 1.5, wheel_shape), "Mounting CarWheel next to engine must succeed");
    let mut wheel_s0 = ToneLedger { tones: Vec::new(), down_pebbles: 0 };
    ReliefStrata::new().emit_cell(GridCell::new(2, 1), &frame, &relief, &palette, 64, &mut wheel_s0);
    for stage in 1..=3 {
        assert_eq!(relief.mark_dig_layer(2, 1, 1.75, stage).1, Some(crate::block::BlockMedium::CarWheel));
        let mut wheel_dug = ToneLedger { tones: Vec::new(), down_pebbles: 0 };
        ReliefStrata::new().emit_cell(GridCell::new(2, 1), &frame, &relief, &palette, 64, &mut wheel_dug);
        assert_eq!(
            wheel_dug.tones.len(),
            wheel_s0.tones.len(),
            "Stage {stage} digging on a CarWheel must keep 100% of wheel quads visible instead of vanishing"
        );
    }
    assert_eq!(relief.harvest_layer(2, 1, 1.75).1, Some(crate::block::BlockMedium::CarWheel));
    assert!(!relief.layer_occupied_at(2, 1, 1.75), "Harvested CarWheel must be removed from cell (2, 1)");
    assert!(relief.adjust_rig_at(0, 0, 1, false).0, "Hoisting rig by +1 step must raise the connected vehicle assembly");
    assert!(
        relief.layer_occupied_at(1, 0, 2.25) && relief.layer_occupied_at(2, 0, 2.25),
        "Both the newly gripped block at (1, 0) and EngineCore at (2, 0) must rise to 2.0m..2.5m"
    );
    assert!(relief.adjust_rig_at(2, 0, 0, true).0, "Rotating rig assembly by 90 degrees must succeed");
    assert_eq!(
        relief.harvest_layer(0, 0, 0.25).1,
        Some(crate::block::BlockMedium::CantileverRig),
        "Harvesting the crane mast must fold it back into CantileverRig medium and dismantle the mounted engine"
    );
    assert!(!relief.layer_occupied_at(1, 1, 0.25), "Rotated engine at (1, 1) must be dismantled together with the crane");
    assert!(relief.layer_occupied_at(1, 0, 0.25), "Attached non-engine vehicle blocks must settle onto ground layer 0.0m..0.5m");

    let mut cam_relief = TerracedRelief::in_span(Box::new(FlatMeadow), 64);
    for h in [0.0, 0.5, 1.0] {
        assert!(cam_relief.stack_molded_cell(0, 0, h, Box::new(crate::block::CuboidShape::new(0.5))));
    }
    let mut three_dirt = ToneLedger { tones: Vec::new(), down_pebbles: 0 };
    ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &cam_relief, &palette, 64, &mut three_dirt);
    let cam_shape = crate::block::BlockMold::cube()
        .in_medium(crate::block::BlockMedium::FieldCamera)
        .shape_for(frame.tangent(), &frame);
    assert!(cam_relief.stack_molded_cell(0, 0, 1.5, cam_shape));
    let mut with_cam = ToneLedger { tones: Vec::new(), down_pebbles: 0 };
    ReliefStrata::new().emit_cell(GridCell::new(0, 0), &frame, &cam_relief, &palette, 64, &mut with_cam);
    assert!(
        with_cam.tones.len() >= three_dirt.tones.len() + 30,
        "Placing FieldCamera on 3 stacked dirt blocks must preserve the 3rd dirt block's top quads and add tripod leg quads"
    );
    let mut car_rel = TerracedRelief::in_span(Box::new(FlatMeadow), 64);
    assert!(car_rel.stack_molded_cell(0, 0, 0.0, Box::new(crate::block::MediumShape::new(Box::new(crate::block::CuboidShape::new(0.5)), crate::block::BlockMedium::CantileverRig, true, false))));
    assert!(car_rel.stack_molded_cell(0, 0, 0.0, crate::block::BlockMold::cube().in_medium(crate::block::BlockMedium::SteeringWheel).shape_for(frame.bitangent(), &frame)));
    assert!(car_rel.stack_molded_cell(2, -1, 1.5, crate::block::BlockMold::cube().in_medium(crate::block::BlockMedium::CabinSeat).shape_for(frame.bitangent(), &frame)));
    let s_ray = super::SurfaceSightRay::new(frame.point_at(1.25, -0.25, 2.12), frame.bitangent());
    assert!(car_rel.board_vehicle_at(&frame, 2, 0, 1.75, s_ray, true).is_none());
}
