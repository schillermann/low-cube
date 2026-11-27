use glam::Vec3;
use winit::keyboard::KeyCode;
use world::{
    BlockMedium, BlockMold, CardinalTurn, CelestialSystem, CraftBlueprint, CraftPose, MeshTarget,
    MoldArchetype, PilotOrder, PlanetaryWorld, Quad, SurfaceFrame, VehicleAxes, VehiclePart,
};
use crate::attire::EvaAttire;
use crate::avatar::{Avatar as _, AvatarMotor as _, AvatarPresence as _, HandGesture, PlanetaryAvatar};
use crate::black_box::PresenceProbe;
use crate::controls::{ActionBindings, KeyMotion, KeyboardControls};
use crate::flight::SurfaceFlight;
use crate::zero_g_flight::ZeroGFlight;

fn pilot_snapshot(avatar: &PlanetaryAvatar) -> (Vec3, f32) {
    let mut probe = PresenceProbe::at_origin();
    avatar.report_presence(&mut probe);
    let pos = probe.position();
    let yaw = probe
        .presence()
        .transfer(|_, kin, _, _| kin.transfer(|_, rot, _, _, _| rot.to_euler(glam::EulerRot::YXZ).0));
    (pos, yaw)
}

fn board_and_test_yaw_then_liftoff(mut world: Box<dyn PlanetaryWorld>, seat_x: f32, seat_z: f32, g0: f32) {
    let spawn = world.constrain_to_surface(Vec3::new(seat_x + 0.5, 22.0, seat_z), 1.0);
    let mut pilot = PlanetaryAvatar::with_attire_and_bindings(
        spawn,
        Box::new(ZeroGFlight::wrapping(SurfaceFlight::calibrated(0.522, 9.81, 1.0))),
        EvaAttire::astronaut(),
        ActionBindings::seeded(),
    );
    pilot.rotate_degrees(-90.0, 10.0);
    assert!(HandGesture::RightTap.perform(&mut pilot, world.as_mut()));
    assert!(pilot.use_right_hand(world.as_mut()));

    let (_, yaw_initial) = pilot_snapshot(&pilot);
    let mut yaw_left = KeyMotion::idle();
    yaw_left.register_key(KeyCode::KeyA, true);
    for _ in 0..30 {
        pilot.step_motion(1.0 / 120.0, &yaw_left, world.as_ref());
    }
    let (_, yaw_after_a) = pilot_snapshot(&pilot);
    let mut yaw_right = KeyMotion::idle();
    yaw_right.register_key(KeyCode::KeyD, true);
    for _ in 0..60 {
        pilot.step_motion(1.0 / 120.0, &yaw_right, world.as_ref());
    }
    let (pad_pos, yaw_after_d) = pilot_snapshot(&pilot);
    assert!(
        (yaw_after_a - yaw_initial).abs() > 0.10 && (yaw_after_d - yaw_after_a).abs() > 0.20,
        "A and D must yaw rocket left and right on pad ({yaw_initial:.3} -> {yaw_after_a:.3} -> {yaw_after_d:.3})"
    );

    let mut liftoff = KeyMotion::idle();
    liftoff.register_key(KeyCode::Space, true);
    for _ in 0..300 {
        pilot.step_motion(1.0 / 120.0, &liftoff, world.as_ref());
    }
    let (air_pos, _) = pilot_snapshot(&pilot);
    assert!(
        air_pos.y > pad_pos.y + 4.0 && air_pos.y > g0 + 5.0,
        "Holding Space must lift rocket cleanly off the ground (pad_y={:.2}, air_y={:.2})",
        pad_pos.y,
        air_pos.y
    );
}

#[test]
fn test_compact_skeleton_rocket_with_single_nozzle_yaws_on_ad_and_lifts_off_on_space() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let av = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    for med in [BlockMedium::CantileverRig, BlockMedium::RocketNozzle] {
        assert!(world.place_molded_surface_block(
            Vec3::new(0.25, g0 + 1.5, 1.25),
            crane_dir,
            av,
            BlockMold::cube().in_medium(med),
        ));
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
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 1.25),
        Vec3::NEG_Z,
        av,
        BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, -0.25),
        Vec3::Z,
        Vec3::new(1.25, g0, -1.25),
        BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(0.25, g0 + 1.75, 0.75),
        Vec3::X,
        av,
        BlockMold::cube().in_medium(BlockMedium::PlanedPlanks),
    ));
    for top_y in [2.90, 3.40, 3.90, 4.40] {
        assert!(world.place_molded_surface_block(
            Vec3::new(0.75, g0 + top_y, 0.75),
            Vec3::NEG_Y,
            av,
            BlockMold::cube().in_medium(BlockMedium::PlanedPlanks),
        ));
    }
    assert!(world.place_molded_surface_block(
        Vec3::new(0.75, g0 + 4.90, 0.75),
        Vec3::NEG_Y,
        av,
        BlockMold::cube().in_medium(BlockMedium::RocketCamera),
    ));

    board_and_test_yaw_then_liftoff(world, 1.25, 0.75, g0);
}

#[test]
fn test_steuerduese_only_pod_yaws_on_ad_and_lifts_off_on_space() {
    let mut world = CelestialSystem::barren_authoritative(32, 1.0);
    let g0 = world.constrain_to_surface(Vec3::new(0.25, 22.0, 0.25), 0.0).y;
    let av = Vec3::new(0.25, g0, 1.75);
    let crane_dir = Vec3::new(0.0, -0.8, -0.6).normalize();
    for mold in [
        BlockMold::cube().in_medium(BlockMedium::CantileverRig),
        BlockMold::cube().in_medium(BlockMedium::PlanedPlanks),
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
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, 1.25),
        Vec3::NEG_Z,
        av,
        BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle),
    ));
    assert!(world.place_molded_surface_block(
        Vec3::new(1.25, g0 + 1.75, -0.25),
        Vec3::Z,
        Vec3::new(1.25, g0, -1.25),
        BlockMold::cube().next_form().in_medium(BlockMedium::RocketNozzle),
    ));

    board_and_test_yaw_then_liftoff(world, 1.25, 0.75, g0);
}

#[test]
fn test_linear_hull_mass_demands_additional_main_nozzles_and_streams_live_plumes() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
    let axes = VehicleAxes::new(Vec3::X, Vec3::Z, Vec3::Y);
    let pose = CraftPose::parked(Vec3::ZERO, axes, Vec3::Z);
    let up_burn = PilotOrder::new(Vec3::Y);
    let part = |ox: i32, oy: i32, z: f32, med: BlockMedium| {
        let mold = BlockMold::with_medium(MoldArchetype::FullCube, CardinalTurn::Forward, med);
        VehiclePart::new((ox, oy), z, mold.atom_for(Vec3::Z, &frame))
    };

    let mut hull_parts = vec![
        part(0, 0, 0.5, BlockMedium::CabinSeat),
        part(0, 1, 0.5, BlockMedium::FlightConsole),
        part(0, -1, 0.0, BlockMedium::RocketNozzle),
    ];
    for idx in 0..20 {
        hull_parts.push(part((idx % 5) - 2, (idx / 5) - 2, 1.0, BlockMedium::QuarriedStone));
    }
    let heavy_one_bell = CraftBlueprint::commission(Vec3::ZERO, hull_parts.clone());
    let accel_one = heavy_one_bell.sample_impulse(&up_burn, pose).velocity_delta(1.0).y;
    assert!(
        accel_one < 9.81,
        "Heavy 23-block hulled ship must not lift off Earth with 1 main bell (got {accel_one:.2} m/s^2)"
    );

    hull_parts.push(part(1, -1, 0.0, BlockMedium::RocketNozzle));
    let heavy_two_bells = CraftBlueprint::commission(Vec3::ZERO, hull_parts);
    let accel_two = heavy_two_bells.sample_impulse(&up_burn, pose).velocity_delta(1.0).y;
    assert!(
        accel_two > 10.5,
        "Adding a 2nd main bell must lift the heavy hulled ship cleanly (got {accel_two:.2} m/s^2)"
    );

    struct QuadTally(usize);
    impl MeshTarget for QuadTally {
        fn push_quad(&mut self, _quad: Quad) { self.0 += 1; }
    }
    let burn_pose = pose.with_motion(1.0, 0.5);
    let (mut cpu, mut local, mut kin) = (QuadTally(0), QuadTally(0), QuadTally(0));
    heavy_two_bells.emit_at(burn_pose, &mut cpu);
    heavy_two_bells.emit_local_hull(&mut local);
    heavy_two_bells.emit_kinematic_parts(burn_pose, &mut kin);
    assert_eq!(
        local.0 + kin.0,
        cpu.0,
        "GPU VesselStage split must stream live rocket plumes and console levers every frame"
    );
}
