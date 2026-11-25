use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{
    AttitudeGuidance, BiomedCubeWorld, BlockMedium, BlockMold, CardinalTurn, ClimateZone,
    MeshTarget, MoldArchetype, PilotOrder, PlanetaryWorld, Quad, VehicleAxes, VehiclePart,
    WorldEarthworks,
};
use crate::attire::{ArtisanTome, BeltSatchel, HeldRelic, YouthAttire};
use crate::avatar::{Avatar, AvatarMotor, BlackBoxDeck, HandGesture, PlanetaryAvatar};
use crate::black_box::{BlackBoxAvatar, LiveTape};
use crate::controls::{ActionBindings, ControlAction, InputTrigger, KeyboardControls, LocomotionTarget};
use crate::drive::{CourseProfile, FlipBurnReadout, NavigationCourse, TransferArc, ARC_NODES};
use crate::flight::SurfaceFlight;
use crate::zero_g_flight::ZeroGFlight;

struct RestControls;
impl KeyboardControls for RestControls {
    fn drive_locomotion(&self, _forward: Vec3, _right: Vec3, _target: &mut dyn LocomotionTarget) {}
}

struct FullVerticalBurn;
impl KeyboardControls for FullVerticalBurn {
    fn drive_locomotion(&self, _forward: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        target.jump();
    }
}

struct FullDiveBurn;
impl KeyboardControls for FullDiveBurn {
    fn drive_locomotion(&self, _forward: Vec3, _right: Vec3, target: &mut dyn LocomotionTarget) {
        target.sneak();
    }
}

struct QuadCounter {
    count: usize,
}
impl MeshTarget for QuadCounter {
    fn push_quad(&mut self, _q: Quad) {
        self.count += 1;
    }
}

fn build_prospector_parts(ore_medium: Option<(BlockMedium, i32)>) -> Vec<VehiclePart> {
    let mut parts = vec![
        VehiclePart::new((0, 0), 0.0, BlockMold::shape_from_stamp([0, 0, BlockMedium::CabinSeat.code()])),
        VehiclePart::new((0, 1), 0.0, BlockMold::shape_from_stamp([0, 0, BlockMedium::FlightConsole.code()])),
        VehiclePart::new((0, -1), 0.0, BlockMold::shape_from_stamp([0, 0, BlockMedium::FusionTorch.code()])),
        VehiclePart::new(
            (1, 0),
            0.0,
            BlockMold::shape_from_stamp([2, 1, BlockMedium::RocketNozzle.code()]),
        ),
        VehiclePart::new(
            (-1, 0),
            0.0,
            BlockMold::shape_from_stamp([2, 3, BlockMedium::RocketNozzle.code()]),
        ),
    ];
    if let Some((med, count)) = ore_medium {
        for i in 0..count {
            parts.push(VehiclePart::new(
                ((i % 3) - 1, -2 - (i / 3)),
                0.0,
                BlockMold::shape_from_stamp([0, 0, med.code()]),
            ));
        }
    }
    parts
}

#[test]
fn test_cargo_ballast_scales_acceleration_by_ore_density_and_shifts_flip_burn_point_earlier() {
    let axes = VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y);
    let full = PilotOrder::from_channels(Vec3::Y, Vec3::ZERO, Vec3::ZERO, AttitudeGuidance::Engaged);

    let empty_ship = world::VehicleAssembly::new(Vec3::ZERO, axes, Vec3::Z, build_prospector_parts(None));
    let plank_ship = world::VehicleAssembly::new(Vec3::ZERO, axes, Vec3::Z, build_prospector_parts(Some((BlockMedium::PlanedPlanks, 6))));
    let copper_ship = world::VehicleAssembly::new(Vec3::ZERO, axes, Vec3::Z, build_prospector_parts(Some((BlockMedium::CopperOre, 6))));
    let iron_ship = world::VehicleAssembly::new(Vec3::ZERO, axes, Vec3::Z, build_prospector_parts(Some((BlockMedium::IronOre, 6))));
    let gold_ship = world::VehicleAssembly::new(Vec3::ZERO, axes, Vec3::Z, build_prospector_parts(Some((BlockMedium::GoldOre, 6))));

    let accel_of = |ship: &world::VehicleAssembly, satchel_cargo: f32| {
        ship.sample_loaded_impulse(&full, satchel_cargo).fusion_velocity_delta(1.0).length()
    };
    let a_empty = accel_of(&empty_ship, 0.0);
    let a_plank = accel_of(&plank_ship, 0.0);
    let a_copper = accel_of(&copper_ship, 0.0);
    let a_iron = accel_of(&iron_ship, 0.0);
    let a_gold = accel_of(&gold_ship, 0.0);

    assert!((a_empty - 78.5).abs() < 1.0, "Empty Epstein prospector must hit ~8G cap (78.5 m/s^2), got {a_empty}");
    assert!(
        a_plank > a_copper && a_copper > a_iron && a_iron > a_gold,
        "Physical hull ore density must enforce Plank({a_plank:.1}) > Copper({a_copper:.1}) > Iron({a_iron:.1}) > Gold({a_gold:.1})"
    );
    assert!(a_gold < 2.0 * 9.81, "6 GoldOre cargo blocks must drop acceleration below 2G, got {a_gold:.2} m/s^2");

    // Pilot satchel burden (`SatchelBurden::ballast_tonnage`) also weighs down the ship via `CargoBallast`:
    let satchel = BeltSatchel::empty().with_stowed_relic(HeldRelic::GoldOre, 20);
    let cargo_tonnage = satchel.carried_burden(YouthAttire::squire()).ballast_tonnage();
    let a_satchel_laden = accel_of(&empty_ship, cargo_tonnage);
    assert!(a_satchel_laden < 1.8 * 9.81, "20 GoldOre in pilot satchel must weigh down empty ship below 1.8G, got {a_satchel_laden:.2}");

    // `FlipBurnReadout`: at 30% route progress and 320 m/s closing speed with 3,000 m remaining:
    // Empty ship (78.5 m/s^2 -> stop_dist = 652 m) still coasts/accelerates (`TimeToFlip`),
    // while GoldOre-laden ship (14.0 m/s^2 -> stop_dist = 3,657 m > 3,000 m) immediately warns `FlipNow`!
    let nodes: [Vec3; ARC_NODES] = std::array::from_fn(|i| Vec3::new(0.0, 0.0, (i as f32) * (4_285.7 / ((ARC_NODES - 1) as f32))));
    let arc = TransferArc::with_profile(
        nodes,
        Vec3::NEG_Z,
        24.0,
        CourseProfile::Brachistochrone {
            flip_point: Vec3::new(0.0, 0.0, 2_142.8),
            total_secs: 24.0,
            burn_accel: a_empty,
        },
    );
    let pos_at_30_pct = Vec3::new(0.0, 0.0, 1_285.7);
    let vel = Vec3::new(0.0, 0.0, 320.0);
    let readout_empty = arc.sample_flip_burn_with_accel(pos_at_30_pct, vel, Vec3::Z, a_empty);
    let readout_laden = arc.sample_flip_burn_with_accel(pos_at_30_pct, vel, Vec3::Z, a_satchel_laden);
    assert!(matches!(readout_empty, FlipBurnReadout::TimeToFlip(_)), "Empty 8G ship still has braking margin: {readout_empty:?}");
    assert_eq!(readout_laden, FlipBurnReadout::FlipNow, "Heavy ore-laden ship must command FlipNow earlier so it does not overshoot!");
}

#[test]
fn test_fusion_torch_vitrifies_launchpad_crash_deposits_blackbox_pod_and_hand_readout_opens_log() {
    let mut world = BiomedCubeWorld::barren(32);
    let pad = world.constrain_to_surface(Vec3::new(0.25, 24.0, 0.25), 0.0);

    // 1. Craft `BlackBoxPod` via `ArtisanTome` recipe #23 (`2 IronOre + 1 CopperOre`):
    let stocked = BeltSatchel::empty()
        .with_stowed_relic(HeldRelic::IronOre, 2)
        .with_stowed_relic(HeldRelic::CopperOre, 1);
    let tome = ArtisanTome::new(23, true);
    assert_eq!(tome.selected_entry().product(), HeldRelic::BlackBoxPod);
    let (_, _, crafted_attire, ok) = tome.craft_selected(1.0, stocked, YouthAttire::squire());
    assert!(ok && crafted_attire.held_pair().0 == HeldRelic::BlackBoxPod, "ArtisanTome #23 must craft BlackBoxPod");

    // 2. Commission FusionTorch ship on Temperate pad, ignite upward -> leaves local `VitrifiedBasalt` scorch slab!
    let g0 = pad.y;
    let av = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    for mold in [
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
        BlockMold::cube().in_medium(BlockMedium::FusionTorch),
    ] {
        assert!(world.place_molded_surface_block(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, av, mold));
    }
    assert!(world.adjust_rig(Vec3::new(0.25, g0 + 1.5, 1.25), crane_dir, -1, false));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 2.40, 0.25),
        Vec3::NEG_Y,
        av,
        BlockMold::cube().turn_clockwise().in_medium(BlockMedium::FlightConsole),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 0.75),
        Vec3::NEG_Z,
        av,
        BlockMold::cube().in_medium(BlockMedium::CabinSeat),
    ));

    let stand = world.constrain_to_surface(Vec3::new(1.75, 24.0, 0.75), 1.0);
    let inner = PlanetaryAvatar::with_attire_and_bindings(
        stand,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        YouthAttire::squire(),
        ActionBindings::seeded(),
    );
    let mut pilot = BlackBoxAvatar::wrapping(Box::new(inner), Box::new(LiveTape::fresh()));
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, &mut world));
    assert!(pilot.use_right_hand(&mut world));

    let mut vitrified = false;
    for _ in 0..22 {
        pilot.step_motion(1.0 / 60.0, &FullVerticalBurn, &world);
        vitrified |= pilot.step_digging(&mut world);
    }
    assert!(vitrified, "Firing FusionTorch over planetary crust must vitrify launchpad");

    // Verify other ClimateZones remained 100% untouched (strict local sector isolation):
    let mut snapshots = Vec::new();
    world.collect_sector_snapshots(&mut snapshots);
    assert!(!snapshots.is_empty());
    assert!(
        snapshots.iter().all(|s| s.face() == ClimateZone::Temperate.index() as u8),
        "VitrifiedBasalt scorching must stay strictly isolated to the local launchpad face/sector"
    );

    // 3. Ballistic apex (~38 m above pad) then free-fall slam (`v_norm < -26 m/s`) -> deposits `BlackBoxPod`!
    pilot.install_navigation_course(NavigationCourse::Uncharted);
    let mut crash_altered = false;
    for _ in 0..380 {
        pilot.step_motion(1.0 / 60.0, &FullDiveBurn, &world);
        if pilot.step_digging(&mut world) {
            let mut snaps = Vec::new();
            world.collect_sector_snapshots(&mut snaps);
            let mut raw = Vec::new();
            for s in &snaps {
                s.inscribe(&mut raw);
            }
            if raw.contains(&BlockMedium::BlackBoxPod.code()) {
                crash_altered = true;
                break;
            }
        }
    }
    assert!(crash_altered, "Severe spacecraft impact (> 26 m/s) must deposit a salvageable BlackBoxPod at the crash site");

    // 4. Instant single-click salvage + HUD `IncidentSheet` readout in hand:
    let mut salvage_world = BiomedCubeWorld::barren(16);
    let g = salvage_world.constrain_to_surface(Vec3::new(0.25, 16.0, -0.75), 0.0);
    let pod_mold = BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::BlackBoxPod);
    assert!(salvage_world.place_molded_surface_block(g + Vec3::Y * 1.2, Vec3::NEG_Y, g + Vec3::Z * 2.5, pod_mold));

    let explorer_pos = salvage_world.constrain_to_surface(Vec3::new(0.25, 16.0, 0.25), 1.0);
    let exp_inner = PlanetaryAvatar::with_attire_and_bindings(
        explorer_pos,
        Box::new(SurfaceFlight::calibrated(0.522, 9.81, 1.0)),
        YouthAttire::squire(),
        ActionBindings::seeded(),
    );
    let mut explorer = BlackBoxAvatar::wrapping(Box::new(exp_inner), Box::new(LiveTape::fresh()));
    explorer.step_motion(1.0 / 60.0, &RestControls, &salvage_world);
    explorer.rotate_degrees(0.0, 53.0);

    // Single click (`instant_pickup() == true`) salvages the BlackBoxPod straight into hand!
    assert!(explorer.use_right_hand(&mut salvage_world), "Single click must salvage BlackBoxPod instantly");
    let _ = explorer.release_hand(&mut salvage_world);

    let mut before_hud = QuadCounter { count: 0 };
    explorer.emit_reticle_for_aspect(16.0 / 9.0, &mut before_hud);
    assert!(explorer.dispatch_trigger(InputTrigger::WheelClick), "WheelClick with BlackBoxPod in hand must open IncidentSheet");
    assert!(explorer.black_box_populated());
    let mut after_hud = QuadCounter { count: 0 };
    explorer.emit_reticle_for_aspect(16.0 / 9.0, &mut after_hud);
    assert!(
        after_hud.count > before_hud.count + 40,
        "Reading salvaged BlackBoxPod in hand must emit tilted ruled IncidentSheet on HUD ({} -> {})",
        before_hud.count,
        after_hud.count
    );

    // Verify `ControlAction::CraftingBook` still functions cleanly alongside 20 recipes:
    assert!(explorer.dispatch_trigger(InputTrigger::Key(KeyCode::KeyC)));
    let _ = ControlAction::CraftingBook;
}
