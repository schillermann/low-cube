use glam::{Quat, Vec3};
use crate::attire::{
    BeltSatchel, BodyPose, HeldRelic, LimbFrame, SatchelBurden, StrideRhythm, YouthAttire,
    YouthPhysique,
};
use super::super::burden::{CarryLoad, SlotBurden};
use super::super::compartment::{CompartmentBackplate, SlotPlaque, WeightedCompartment};
use super::QuadLedger;

fn plaque_corner_color(slot: SlotBurden) -> Vec3 {
    let frame = LimbFrame::new(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z);
    let backplate = CompartmentBackplate::new(false, true, Vec3::new(0.42, 0.27, 0.16));
    let mut ledger = QuadLedger::empty();
    WeightedCompartment::new(backplate, slot).emit_slot(&frame, &mut ledger);
    ledger
        .quads
        .last()
        .expect("occupied WeightedCompartment must emit corner triangle quad")
        .transfer(|_, _, _, _, _, color| color)
}

#[test]
fn test_satchel_renders_all_fifteen_slots_and_navigates_empty_and_occupied_compartments() {
    let pose = BodyPose::new(
        Vec3::new(0.0, 9.0, 0.0),
        Quat::IDENTITY,
        Quat::IDENTITY,
        0.8,
        StrideRhythm::new(0.0, 1.0),
        YouthPhysique::SlenderSquire,
    );
    let empty_satchel = BeltSatchel::empty().with_open(true);
    let mut empty_ledger = QuadLedger::empty();
    empty_satchel.emit_for_attire(&pose, YouthAttire::squire(), &mut empty_ledger);
    assert!(
        empty_ledger.quads.len() >= 15 * 73,
        "Empty satchel must still render all 15 available slot frames, got {}",
        empty_ledger.quads.len()
    );

    let (moved_right, ok_right) = empty_satchel.navigate(1, 0);
    let (moved_down, ok_down) = moved_right.navigate(0, 1);
    assert!(ok_right && ok_down, "WASD must navigate freely across unoccupied slots");
    let mut moved_ledger = QuadLedger::empty();
    moved_down.emit_for_attire(&pose, YouthAttire::squire(), &mut moved_ledger);
    let changed_selection = empty_ledger
        .quads
        .iter()
        .zip(moved_ledger.quads.iter())
        .any(|(a, b)| a.transfer(|va, _, _, _, _, _| b.transfer(|vb, _, _, _, _, _| (va - vb).length_squared() > 1e-6)));
    assert!(changed_selection, "Moving cursor onto another empty slot must shift the selected slot frame");
}

#[test]
fn test_english_weight_grades_stack_summation_and_corner_triangle_pigment() {
    let empty_slot = SlotBurden::new(HeldRelic::EmptyPalm, 0);
    let light_one = SlotBurden::new(HeldRelic::PlankBlock, 1);
    let moderate_one = SlotBurden::new(HeldRelic::ExcavatedEarth, 1);
    let weighty_one = SlotBurden::new(HeldRelic::QuarriedStone, 1);
    let heavy_one = SlotBurden::new(HeldRelic::CrustBlock, 1);

    assert_eq!(format!("{empty_slot}"), "EMPTY SLOT");
    assert!(format!("{light_one}").ends_with(" - LIGHT"));
    assert!(format!("{moderate_one}").ends_with(" - MODERATE"));
    assert!(format!("{weighty_one}").ends_with(" - WEIGHTY"));
    assert!(format!("{heavy_one}").ends_with(" - HEAVY"));

    let fold_pace = |slot: SlotBurden| slot.fold_burden(SatchelBurden::weightless()).scale_pace(6.0);
    assert!(
        fold_pace(light_one) > fold_pace(moderate_one)
            && fold_pace(moderate_one) > fold_pace(weighty_one)
            && fold_pace(weighty_one) > fold_pace(heavy_one),
        "CarryLoad fold_burden must rank unit mass monotonically across Light, Moderate, Weighty, and Heavy"
    );

    // 10-step escalation rule via Display: 1..=9 stays at unit grade, +1 tier at 10, +2 at 20, +3 at 30+
    assert!(format!("{}", SlotBurden::new(HeldRelic::PlankBlock, 9)).ends_with("X9 - LIGHT"));
    assert!(format!("{}", SlotBurden::new(HeldRelic::PlankBlock, 10)).ends_with("LIGHT -> MODERATE"));
    assert!(format!("{}", SlotBurden::new(HeldRelic::PlankBlock, 19)).ends_with("LIGHT -> MODERATE"));
    assert!(format!("{}", SlotBurden::new(HeldRelic::PlankBlock, 20)).ends_with("LIGHT -> WEIGHTY"));
    assert!(format!("{}", SlotBurden::new(HeldRelic::PlankBlock, 29)).ends_with("LIGHT -> WEIGHTY"));
    let light_thirty = SlotBurden::new(HeldRelic::PlankBlock, 30);
    let mod_nine = SlotBurden::new(HeldRelic::ExcavatedEarth, 9);
    let mod_ten = SlotBurden::new(HeldRelic::ExcavatedEarth, 10);
    let mod_twenty = SlotBurden::new(HeldRelic::ExcavatedEarth, 20);
    let weighty_ten = SlotBurden::new(HeldRelic::QuarriedStone, 10);
    let heavy_ten = SlotBurden::new(HeldRelic::CrustBlock, 10);
    assert!(format!("{light_thirty}").ends_with("LIGHT -> HEAVY"));
    assert!(format!("{mod_nine}").ends_with("X9 - MODERATE"));
    assert!(format!("{mod_ten}").ends_with("MODERATE -> WEIGHTY"));
    assert!(format!("{mod_twenty}").ends_with("MODERATE -> HEAVY"));
    assert!(format!("{weighty_ten}").ends_with("WEIGHTY -> HEAVY"));

    // Verify SlotPlaque corner triangle pigments across tiers via WeightedCompartment
    assert_ne!(plaque_corner_color(light_one), plaque_corner_color(moderate_one));
    assert_ne!(plaque_corner_color(moderate_one), plaque_corner_color(weighty_one));
    assert_ne!(plaque_corner_color(weighty_one), plaque_corner_color(heavy_one));
    assert_ne!(plaque_corner_color(heavy_one), plaque_corner_color(heavy_ten));

    let squire = YouthAttire::squire();
    let pose = BodyPose::new(
        Vec3::new(0.0, 9.0, 0.0),
        Quat::IDENTITY,
        Quat::IDENTITY,
        0.8,
        StrideRhythm::new(0.0, 1.0),
        YouthPhysique::SlenderSquire,
    );
    let (mut light_tray, mut heavy_tray) = (QuadLedger::empty(), QuadLedger::empty());
    BeltSatchel::empty().with_stowed_relic(HeldRelic::PlankBlock, 1).with_open(true).emit_for_attire(&pose, squire, &mut light_tray);
    BeltSatchel::empty().with_stowed_relic(HeldRelic::PlankBlock, 30).with_open(true).emit_for_attire(&pose, squire, &mut heavy_tray);
    let color_changed = light_tray
        .quads
        .iter()
        .zip(heavy_tray.quads.iter())
        .any(|(a, b)| a.transfer(|_, _, _, _, _, ca| b.transfer(|_, _, _, _, _, cb| (ca - cb).length_squared() > 1e-4)));
    assert!(color_changed, "Stacking items in 10-steps in a slot must shift the top-left corner triangle weight color");
}

#[test]
fn test_handheld_crane_baseplate_and_pedestal_ring_emit_orthogonal_square_plinths() {
    let hand = LimbFrame::new(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::NEG_Z);
    let mut ledger = QuadLedger::empty();
    HeldRelic::CantileverCrane.figure(world::BlockMold::cube()).emit_in_palm(1.0, &hand, &mut ledger);
    let (stone, iron_steel, timber_dark) = (Vec3::new(0.52, 0.54, 0.58), Vec3::new(0.42, 0.46, 0.52), Vec3::new(0.54, 0.38, 0.20));
    let mut stone_normals = Vec::new();
    let (mut stone_span, mut steel_quads, mut brace_flanks) = (0.0_f32, 0_usize, 0_usize);
    let mut ring_square_top = false;
    for q in &ledger.quads {
        q.transfer(|v0, v1, v2, v3, n, col| {
            if (col - stone).length_squared() < 1e-6 {
                assert!((v2 - v3).length_squared() > 1e-5, "Stone baseplate must emit full rectangular quads");
                stone_span = stone_span.max((v1 - v0).length());
                stone_normals.push(n);
            } else if (col - iron_steel).length_squared() < 1e-6 && (v2 - v3).length_squared() > 1e-5 {
                let (ea, eb) = (v1 - v0, v3 - v0);
                steel_quads += 1;
                ring_square_top |= (ea.length() - 0.030).abs() < 1e-4 && (ea.length() - eb.length()).abs() < 1e-5 && ea.dot(eb).abs() < 1e-5;
            } else if (col - timber_dark).length_squared() < 1e-6 && (v2 - v3).length_squared() > 1e-5 {
                brace_flanks += 1;
            }
        });
    }
    assert_eq!(stone_normals.len(), 6, "Square crane baseplate must emit 6 orthogonal box faces");
    assert!((stone_span - 0.054).abs() < 1e-4, "Stone baseplate must be compact and proportional to the crane mast, got {stone_span}");
    assert!(
        stone_normals.iter().all(|a| stone_normals.iter().all(|b| a.dot(*b).abs() < 1e-4 || (a.dot(*b).abs() - 1.0).abs() < 1e-4)),
        "Stone baseplate faces must meet at 90-degree angles"
    );
    assert!(ring_square_top && steel_quads == 4 && brace_flanks == 3, "Pedestal ring, mast/crossbar iron shoes, and timber-iron sloped knee brace must match placed crane");
}
