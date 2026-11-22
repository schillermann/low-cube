use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CardinalTurn, CelestialSystem, ClimateZone,
    CraftBlueprint, CraftPose, LunarWorld, MoldArchetype, NozzleAxis, PilotOrder,
    SurfaceFrame, VehicleAssembly, VehicleAxes, VehiclePart, WedgeHeading,
};

#[test]
fn test_nozzle_axis_toggles_between_downward_main_bell_and_attitude_pod() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let down = NozzleAxis::from_mold(MoldArchetype::from_code(0), WedgeHeading::North);
    assert_eq!(down, NozzleAxis::Downward);
    assert!(!down.attitude_pod());
    assert!((down.thrust_vector(&frame) - Vec3::Y).length() < 1e-5);

    let rcs = NozzleAxis::from_mold(MoldArchetype::from_code(1), WedgeHeading::North);
    assert_eq!(rcs, NozzleAxis::Attitude(WedgeHeading::North));
    assert!(rcs.attitude_pod());
    assert!((rcs.exhaust_vector(&frame) - Vec3::NEG_Z).length() < 1e-5);
    assert!((rcs.thrust_vector(&frame) - Vec3::Z).length() < 1e-5);

    let mold = BlockMold::cube().in_medium(BlockMedium::RocketNozzle);
    let scrolled = mold.next_form();
    let scrolled_back = scrolled.next_form();
    assert_eq!(mold.chronicle_bytes()[0], 0);
    assert_eq!(scrolled.chronicle_bytes()[0], 1);
    assert_eq!(scrolled_back.chronicle_bytes()[0], 0);
}

#[test]
fn test_attitude_nozzle_steers_spacecraft_and_fires_retro_descent_jet() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let axes = VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y);
    let seat_mold = BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::CabinSeat);
    let console_mold = BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::FlightConsole);
    let rcs_mold = BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle);

    let rcs_bp = CraftBlueprint::commission(
        Vec3::ZERO,
        vec![
            VehiclePart::new((0, 0), 0.0, seat_mold.shape_for(Vec3::Z, &frame)),
            VehiclePart::new((0, 1), 0.0, console_mold.shape_for(Vec3::Z, &frame)),
            VehiclePart::new((0, -1), 0.0, rcs_mold.shape_for(Vec3::Z, &frame)),
        ],
    );
    let pose = CraftPose::parked(Vec3::ZERO, axes, Vec3::Z);
    let pitch_forward = PilotOrder::new(Vec3::Z);
    let pitch_imp = rcs_bp.sample_impulse(&pitch_forward, pose);
    assert!(pitch_imp.velocity_delta(1.0).z > 8.0, "Aft Steuerdüse must push craft forward");
    let tilted_up = pitch_imp.attitude_step(0.2) * Vec3::Y;
    assert!(tilted_up.z > 0.05, "Steuerdüse must actively pitch spacecraft nose forward on W");

    let yaw_right = PilotOrder::new(Vec3::X);
    let yaw_imp = rcs_bp.sample_impulse(&yaw_right, pose);
    let turned_fwd = yaw_imp.attitude_step(0.2) * Vec3::Z;
    assert!(turned_fwd.x < -0.05, "Steuerdüse must actively yaw spacecraft heading on D");

    let retro_down = PilotOrder::new(Vec3::NEG_Y);
    let retro_imp = rcs_bp.sample_impulse(&retro_down, pose);
    assert!(retro_imp.velocity_delta(1.0).y < -8.0, "Steuerdüse dorsal horn must brake downward on Shift");
}

#[test]
fn test_console_gyro_absorbs_compact_trim_but_yields_to_extreme_outrigger_nozzles() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let axes = VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y);
    let seat_mold = BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::CabinSeat);
    let console_mold = BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::FlightConsole);
    let nozzle_mold = BlockMold::with_medium(MoldArchetype::from_code(0), CardinalTurn::Forward, BlockMedium::RocketNozzle);

    let compact_bp = CraftBlueprint::commission(
        Vec3::ZERO,
        vec![
            VehiclePart::new((0, 0), 0.0, seat_mold.shape_for(Vec3::Z, &frame)),
            VehiclePart::new((0, 1), 0.0, console_mold.shape_for(Vec3::Z, &frame)),
            VehiclePart::new((0, -1), 0.0, nozzle_mold.shape_for(Vec3::Z, &frame)),
        ],
    );
    assert!(compact_bp.spacefaring());
    let pose = CraftPose::parked(Vec3::ZERO, axes, Vec3::Z);
    let up_burn = PilotOrder::new(Vec3::Y);
    let compact_imp = compact_bp.sample_impulse(&up_burn, pose);
    assert!(compact_imp.velocity_delta(1.0).y > 20.0);
    let compact_rot = compact_imp.attitude_step(0.1);
    assert!((compact_rot.w - 1.0).abs() < 1e-5, "Compact 1-cell pod must stay gyro-stabilized");

    let forward_only = PilotOrder::new(Vec3::Z);
    let silent_imp = compact_bp.sample_impulse(&forward_only, pose);
    assert!(silent_imp.velocity_delta(1.0).length() < 1e-5, "Downward nozzle must not fire on forward burn");

    let asymmetric_bp = CraftBlueprint::commission(
        Vec3::ZERO,
        vec![
            VehiclePart::new((0, 0), 0.0, seat_mold.shape_for(Vec3::Z, &frame)),
            VehiclePart::new((0, 1), 0.0, console_mold.shape_for(Vec3::Z, &frame)),
            VehiclePart::new((5, 0), 0.0, nozzle_mold.shape_for(Vec3::Z, &frame)),
            VehiclePart::new((6, 0), 0.0, nozzle_mold.shape_for(Vec3::Z, &frame)),
        ],
    );
    let asym_imp = asymmetric_bp.sample_impulse(&up_burn, pose);
    let asym_rot = asym_imp.attitude_step(0.2);
    assert!(asym_rot.angle_between(glam::Quat::IDENTITY) > 0.05, "Extreme outrigger nozzles must tip craft");
}

#[test]
fn test_binary_gravity_and_orbital_fleet_across_homeworld_vacuum_and_moon() {
    let mut cosmos = CelestialSystem::barren_authoritative(32, 1.0);
    let moon_pos = CelestialSystem::moon_center(32);

    let earth_pull = cosmos.apply_pull(Vec3::new(0.25, 17.0, 0.25), 1.0);
    assert!((earth_pull.y + 9.81).abs() < 0.05);

    let deep_space = Vec3::new(0.0, 160.0, -48.0);
    let void_pull = cosmos.apply_pull(deep_space, 1.0);
    assert!(void_pull.length() < 1e-5, "Deep space between Earth and Moon must be 0.0g, got {void_pull:?}");

    let lunar_top = moon_pos + Vec3::new(0.25, LunarWorld::HALF_SIZE + 1.0, 0.25);
    let moon_pull = cosmos.apply_pull(lunar_top, 1.0);
    assert!((moon_pull.y + 2.16).abs() < 0.05, "Moon surface must pull at ~2.16 m/s^2, got {moon_pull:?}");

    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let axes = VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y);
    let pod = VehicleAssembly::new(
        deep_space,
        axes,
        Vec3::Z,
        vec![
            VehiclePart::new(
                (0, 0),
                0.0,
                BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::CabinSeat)
                    .shape_for(Vec3::Z, &frame),
            ),
            VehiclePart::new(
                (0, 1),
                0.0,
                BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, BlockMedium::FlightConsole)
                    .shape_for(Vec3::Z, &frame),
            ),
        ],
    );
    assert!(cosmos.park_vehicle(pod));
    let reboarded = cosmos
        .board_vehicle(deep_space - Vec3::Z * 2.0, Vec3::Z, true)
        .expect("Astronaut must re-board weightless orbital craft in deep space");
    assert!((reboarded.origin() - deep_space).length() < 1e-4);

    let moon_south = moon_pos + Vec3::new(0.25, -LunarWorld::HALF_SIZE - 1.0, 0.25);
    assert_eq!(cosmos.surface_up_at(moon_south), ClimateZone::Volcanic.normal());
}
