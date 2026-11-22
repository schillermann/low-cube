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

#[test]
fn test_kepler_ellipse_conserves_vis_viva_and_couples_perihelion_to_southern_trio() {
    use std::f64::consts::TAU;
    use crate::{ApexSpin, BodyId, PlanetarySpin, SolarEphemeris, SolarProportions};

    let prop = SolarProportions::true_scale(64);
    let au = prop.astronomical_unit();
    let eph0 = SolarEphemeris::new(prop, 0.0);
    let sun_gm = eph0.snapshot_all(0.0)[0].gm();
    let year_secs = TAU * (au * au * au / sun_gm).sqrt();

    let north_apex = ApexSpin::terrestrial().north_apex();
    let (mut min_r, mut max_r) = (f64::MAX, 0.0_f64);
    let (mut peri_speed, mut aphel_speed) = (0.0_f64, f64::MAX);
    let (mut peri_decl, mut aphel_decl) = (0.0_f32, 0.0_f32);

    for step in 0..240 {
        let t = year_secs * (f64::from(step) / 240.0);
        let snaps = eph0.snapshot_all(t);
        let sun = snaps[0];
        let r = sun.center().length();
        let earth_helio_speed = sun.velocity().length();
        let vis_viva_v = (sun_gm * (2.0 / r - 1.0 / au)).sqrt();
        assert!(
            ((earth_helio_speed - vis_viva_v) / vis_viva_v).abs() < 1e-6,
            "Keplerian velocity must satisfy Vis-Viva energy equation: got {earth_helio_speed}, expected {vis_viva_v}",
        );
        let sun_unit = sun.center().normalize().as_vec3();
        if r < min_r {
            min_r = r;
            peri_speed = earth_helio_speed;
            peri_decl = sun_unit.dot(north_apex);
        }
        if r > max_r {
            max_r = r;
            aphel_speed = earth_helio_speed;
            aphel_decl = sun_unit.dot(north_apex);
        }
        let merc = snaps.into_iter().find(|s| s.id() == BodyId::MERCURY).expect("Mercury");
        let merc_r = (merc.center() - sun.center()).length();
        assert!(merc_r > 0.29 * au && merc_r < 0.48 * au, "Mercury ellipse must span 0.307..0.467 AU");
    }

    assert!((min_r / au - 0.80).abs() < 0.01, "Earth perihelion must reach 0.80 AU, got {}", min_r / au);
    assert!((max_r / au - 1.20).abs() < 0.01, "Earth aphelion must reach 1.20 AU, got {}", max_r / au);
    assert!(peri_speed > aphel_speed * 1.48, "Kepler 2nd law: perihelion speed must exceed aphelion by ~1.5x");
    assert!(peri_decl < -0.55, "Perihelion Sun must tilt into hot Southern Apex (+X/-Y/+Z), got {peri_decl}");
    assert!(aphel_decl > 0.55, "Aphelion Sun must tilt into cool Northern Apex (-X/+Y/-Z), got {aphel_decl}");
}

#[test]
fn test_corner_apex_spin_illuminates_all_six_climate_zones_and_rotates_distant_earth() {
    use crate::{
        ApexSpin, CelestialSystem, MeshTarget, PlanetarySpin, Quad, SightCone, SolarEphemeris,
        SolarProportions,
    };

    let prop = SolarProportions::true_scale(64);
    let eph = SolarEphemeris::new(prop, 0.0);
    let spin = ApexSpin::from_ephemeris(eph);
    let half = prop.earth_half() + 1.0;

    for zone in ClimateZone::all() {
        let surface_spot = (zone.normal() * half).as_dvec3();
        let (mut max_elev, mut min_elev) = (-1.0_f32, 1.0_f32);
        for tick in 0..48 {
            let dt = spin.epoch_after_turns(f64::from(tick) / 48.0);
            let sun_dir = eph.sun_direction_from(surface_spot, dt);
            let elev = sun_dir.dot(zone.normal());
            max_elev = max_elev.max(elev);
            min_elev = min_elev.min(elev);
        }
        assert!(
            max_elev > 0.22 && min_elev < -0.22,
            "{zone:?} must experience full day ({max_elev:.2}) and night ({min_elev:.2}) under corner spin",
        );
    }

    struct SurfaceNormals(Vec<Vec3>);
    impl MeshTarget for SurfaceNormals {
        fn push_quad(&mut self, q: Quad) {
            if q.radiate(|g| (g - 0.24).abs() < 0.01) {
                q.transfer(|_, _, _, _, n, _| self.0.push(n));
            }
        }
    }
    let mut world = CelestialSystem::barren_heliosphere(prop, 1.0);
    let deep_eye = Vec3::new(0.0, 0.0, 65_000.0);
    let mut n0 = SurfaceNormals(Vec::new());
    world.emit_cosmic_field(SightCone::from_view(deep_eye, Vec3::NEG_Z), &mut n0);
    let _ = world.step_surface(spin.epoch_after_turns(0.16) as f32);
    let mut n1 = SurfaceNormals(Vec::new());
    world.emit_cosmic_field(SightCone::from_view(deep_eye, Vec3::NEG_Z), &mut n1);
    assert!(!n0.0.is_empty() && !n1.0.is_empty());
    assert!(
        (n0.0[0] - n1.0[0]).length() > 0.45,
        "Distant orbital Earth cube must visibly spin around its (-1,+1,-1) corner axis",
    );
}
