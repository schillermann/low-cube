use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{ClimateZone, CubePlanet, FacetedReliefFace, NullFace, PlanetaryWorld};
use crate::attire::{
    BeltSatchel, BodyPose, HeldRelic, StrideRhythm, YouthAttire, YouthPhysique,
};
use super::super::burden::{SlotBurden, WeightGrade};
use crate::avatar::{Avatar, PlanetaryAvatar};
use crate::controls::{KeyboardControls, LocomotionTarget};
use crate::server_satchel::StartingSatchel;
use super::QuadLedger;

struct BurdenProvision {
    relic: HeldRelic,
    count: u32,
}

impl StartingSatchel for BurdenProvision {
    fn provision(&self, satchel: BeltSatchel) -> BeltSatchel {
        satchel.with_stowed_relic(self.relic, self.count)
    }
}

struct StrideControls;

impl KeyboardControls for StrideControls {
    fn register_key(&mut self, _code: KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, _forward: Vec3, right: Vec3, target: &mut dyn LocomotionTarget) {
        target.travel(right);
    }
}

struct LeapControls {
    triggered: bool,
}

impl KeyboardControls for LeapControls {
    fn register_key(&mut self, _code: KeyCode, _pressed: bool) {}
    fn drive_locomotion(&self, _forward: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        if self.triggered {
            target.jump();
        }
    }
}

fn flat_planet() -> CubePlanet {
    let face = Box::new(FacetedReliefFace::in_zone(ClimateZone::Temperate, 8.0, 16));
    CubePlanet::new(vec![face], Box::new(NullFace))
}

fn measure_stride_distance(relic: HeldRelic, stowed_count: u32) -> f32 {
    let world = flat_planet();
    let start = world.constrain_to_surface(Vec3::new(-2.0, 12.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(start)
        .with_starting_satchel(&BurdenProvision { relic, count: stowed_count });
    let controls = StrideControls;
    for _ in 0..45 {
        avatar.step_motion(1.0 / 60.0, &controls, &world);
    }
    let end_eye = avatar.view_projection_matrix(16.0 / 9.0).inverse().project_point3(Vec3::ZERO);
    end_eye.x - start.x
}

fn measure_peak_leap(relic: HeldRelic, stowed_count: u32) -> f32 {
    let world = flat_planet();
    let start = world.constrain_to_surface(Vec3::new(0.25, 12.0, 0.25), 1.0);
    let mut avatar = PlanetaryAvatar::at(start)
        .with_starting_satchel(&BurdenProvision { relic, count: stowed_count });
    let base_eye_y = avatar.view_projection_matrix(16.0 / 9.0).inverse().project_point3(Vec3::ZERO).y;
    let mut peak_gain = 0.0_f32;
    for step in 0..45 {
        avatar.step_motion(1.0 / 60.0, &LeapControls { triggered: step == 0 }, &world);
        let eye_y = avatar.view_projection_matrix(16.0 / 9.0).inverse().project_point3(Vec3::ZERO).y;
        peak_gain = peak_gain.max(eye_y - base_eye_y);
    }
    peak_gain
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
    empty_satchel.emit_on_waist(&pose, &mut empty_ledger);
    assert!(
        empty_ledger.quads.len() >= 15 * 73,
        "Empty satchel must still render all 15 available slot frames, got {}",
        empty_ledger.quads.len()
    );

    let (moved_right, ok_right) = empty_satchel.navigate(1, 0);
    let (moved_down, ok_down) = moved_right.navigate(0, 1);
    assert!(ok_right && ok_down, "WASD must navigate freely across unoccupied slots");
    let mut moved_ledger = QuadLedger::empty();
    moved_down.emit_on_waist(&pose, &mut moved_ledger);
    let changed_selection = empty_ledger
        .quads
        .iter()
        .zip(moved_ledger.quads.iter())
        .any(|(a, b)| a.transfer(|va, _, _, _, _, _| b.transfer(|vb, _, _, _, _, _| (va - vb).length_squared() > 1e-6)));
    assert!(changed_selection, "Moving cursor onto another empty slot must shift the selected slot frame");
}

#[test]
fn test_english_weight_grades_stack_summation_and_corner_triangle_pigment() {
    let light_one = SlotBurden::new(HeldRelic::PlankBlock, 1);
    let moderate_one = SlotBurden::new(HeldRelic::ExcavatedEarth, 1);
    let heavy_one = SlotBurden::new(HeldRelic::QuarriedStone, 1);

    assert_eq!(light_one.unit_grade(), WeightGrade::Light);
    assert_eq!(moderate_one.unit_grade(), WeightGrade::Moderate);
    assert_eq!(heavy_one.unit_grade(), WeightGrade::Heavy);
    assert_eq!(format!("{}", WeightGrade::Light), "LIGHT");
    assert_eq!(format!("{}", WeightGrade::Moderate), "MODERATE");
    assert_eq!(format!("{}", WeightGrade::Heavy), "HEAVY");

    let light_three = SlotBurden::new(HeldRelic::PlankBlock, 3);
    let light_six = SlotBurden::new(HeldRelic::PlankBlock, 6);
    let heavy_three = SlotBurden::new(HeldRelic::QuarriedStone, 3);
    assert_eq!(light_three.summed_grade(), WeightGrade::Moderate);
    assert_eq!(light_six.summed_grade(), WeightGrade::Heavy);
    assert!(format!("{}", light_six).contains("LIGHT -> HEAVY"));
    assert_ne!(light_one.corner_pigment(), moderate_one.corner_pigment());
    assert_ne!(moderate_one.corner_pigment(), heavy_one.corner_pigment());
    assert_ne!(heavy_one.corner_pigment(), heavy_three.corner_pigment());

    let pose = BodyPose::new(
        Vec3::new(0.0, 9.0, 0.0),
        Quat::IDENTITY,
        Quat::IDENTITY,
        0.8,
        StrideRhythm::new(0.0, 1.0),
        YouthPhysique::SlenderSquire,
    );
    let (mut light_tray, mut heavy_tray) = (QuadLedger::empty(), QuadLedger::empty());
    BeltSatchel::empty().with_stowed_relic(HeldRelic::PlankBlock, 1).with_open(true).emit_on_waist(&pose, &mut light_tray);
    BeltSatchel::empty().with_stowed_relic(HeldRelic::PlankBlock, 6).with_open(true).emit_on_waist(&pose, &mut heavy_tray);
    let color_changed = light_tray
        .quads
        .iter()
        .zip(heavy_tray.quads.iter())
        .any(|(a, b)| a.transfer(|_, _, _, _, _, ca| b.transfer(|_, _, _, _, _, cb| (ca - cb).length_squared() > 1e-4)));
    assert!(color_changed, "Stacking items in a slot must shift the top-left corner triangle weight color");
}

#[test]
fn test_inventory_weight_reduces_player_speed_and_jump_height() {
    let dist_free = measure_stride_distance(HeldRelic::EmptyPalm, 0);
    let dist_light = measure_stride_distance(HeldRelic::PlankBlock, 2);
    let dist_mod = measure_stride_distance(HeldRelic::ExcavatedEarth, 3);
    let dist_heavy = measure_stride_distance(HeldRelic::QuarriedStone, 8);
    assert!(
        dist_free > dist_light && dist_light > dist_mod && dist_mod > dist_heavy,
        "Heavier inventory must progressively slow player movement: free={dist_free}, light={dist_light}, mod={dist_mod}, heavy={dist_heavy}"
    );

    let leap_free = measure_peak_leap(HeldRelic::EmptyPalm, 0);
    let leap_light = measure_peak_leap(HeldRelic::PlankBlock, 2);
    let leap_mod = measure_peak_leap(HeldRelic::ExcavatedEarth, 3);
    let leap_heavy = measure_peak_leap(HeldRelic::QuarriedStone, 8);
    assert!(
        leap_free > leap_light && leap_light > leap_mod && leap_mod > leap_heavy,
        "Heavier inventory must progressively lower peak jump height: free={leap_free}, light={leap_light}, mod={leap_mod}, heavy={leap_heavy}"
    );

    let held_burden = BeltSatchel::empty()
        .carried_burden(YouthAttire::squire().with_relics(HeldRelic::QuarriedStone, HeldRelic::WoodmanAxe));
    assert!(
        held_burden.scale_pace(6.0) < 6.0 && held_burden.scale_leap(1.0) < 1.0,
        "Items carried in hands must also contribute to total encumbrance"
    );
}
