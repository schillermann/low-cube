use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CardinalTurn, CelestialSystem, ClimateZone, CountedMeshTarget,
    CraftBlueprint, CraftPose, LunarWorld, MeshTarget, MoldArchetype, NozzleAxis, PilotOrder,
    Quad, StarVault, SurfaceFrame, VehicleAssembly, VehicleAxes, VehiclePart, WedgeHeading,
};

#[test]
fn test_nozzle_axis_reaction_thrust_opposes_bell_direction() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let down = NozzleAxis::from_mold(MoldArchetype::from_code(0), WedgeHeading::North);
    assert_eq!(down, NozzleAxis::Downward);
    assert!((down.thrust_vector(&frame) - Vec3::Y).length() < 1e-5);

    let aft = NozzleAxis::from_mold(MoldArchetype::from_code(1), WedgeHeading::South);
    assert_eq!(aft, NozzleAxis::Lateral(WedgeHeading::South));
    assert!((aft.thrust_vector(&frame) - Vec3::Z).length() < 1e-5);

    let retro_up = NozzleAxis::from_mold(MoldArchetype::from_code(2), WedgeHeading::North);
    assert_eq!(retro_up, NozzleAxis::Upward);
    assert!((retro_up.thrust_vector(&frame) - Vec3::NEG_Y).length() < 1e-5);
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

    struct NullSink;
    impl MeshTarget for NullSink {
        fn push_quad(&mut self, _quad: Quad) {}
    }
    let mut sink = NullSink;
    let mut stars = CountedMeshTarget::new(&mut sink);
    StarVault::cosmic().emit(&mut stars);
    assert_eq!(stars.count(), 360);

    let moon_south = moon_pos + Vec3::new(0.25, -LunarWorld::HALF_SIZE - 1.0, 0.25);
    assert_eq!(cosmos.surface_up_at(moon_south), ClimateZone::Volcanic.normal());
}
