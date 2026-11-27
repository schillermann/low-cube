use std::fs;
use std::sync::Mutex;
use std::time::Instant;
use crate::argument_celestial::ArgumentCelestial;
use crate::aurora_cadence::NightlyAuroraCadence;
use crate::celestial::{CelestialCadence, TerrestrialCadence, TunedCadence};
use crate::clock::FrozenClock;
use crate::diurnal_pipeline::{DiurnalPipeline, MeasuredEpoch};
use crate::environment_celestial::EnvironmentCelestial;
use crate::file_celestial::FileCelestial;
use crate::firmament::{VaultEphemeris, VoidFirmament, VoidMesh};
use crate::server_celestial::ServerCelestial;

#[test]
fn test_terrestrial_cadence_cycle_and_progress() {
    let cadence = TerrestrialCadence::new();
    assert_eq!(cadence.duration_seconds(), 1200.0);

    assert!((cadence.day_progress(0.0) - 0.0).abs() < 1e-4);
    assert!((cadence.day_progress(300.0) - 0.25).abs() < 1e-4);
    assert!((cadence.day_progress(600.0) - 0.50).abs() < 1e-4);
    assert!((cadence.day_progress(900.0) - 0.75).abs() < 1e-4);
    assert!((cadence.day_progress(1200.0) - 0.0).abs() < 1e-4);
}

#[test]
fn test_terrestrial_cadence_sun_direction() {
    let cadence = TerrestrialCadence::new();
    let noon_sun = cadence.sun_direction(0.25);
    assert!(noon_sun.y > 0.9, "Noon sun must be high in sky");

    let midnight_sun = cadence.sun_direction(0.75);
    assert!(midnight_sun.y < -0.9, "Midnight sun must be below the horizon");

    let sunset_sun = cadence.sun_direction(0.5);
    assert!(sunset_sun.y.abs() < 0.05, "Sunset sun must be near horizon");
    assert!(sunset_sun.x < 0.0, "Sunset sun must be on western horizon");
}

#[test]
fn test_terrestrial_atmospheric_shell_sky_color_transitions() {
    use glam::Vec3;
    use crate::rayleigh_shell::{AtmosphericEnvelope, AtmosphericShell};

    let cadence = TerrestrialCadence::new();
    let shell = AtmosphericShell::terrestrial_at(Vec3::new(0.0, 17.0, 0.0));
    let (_, noon_sky, noon_night) = shell.sky_illumination(cadence.sun_direction(0.25));
    assert!(noon_sky.z > noon_sky.x && noon_night == 0.0, "Noon sky must be dominantly blue");

    let (_, sunset_sky, _) = shell.sky_illumination(cadence.sun_direction(0.5));
    assert!(sunset_sky.x > 0.7, "Sunset sky must have golden/crimson red tint");

    let (_, midnight_sky, mid_night) = shell.sky_illumination(cadence.sun_direction(0.75));
    assert!(
        midnight_sky.x < 0.03 && midnight_sky.y < 0.03 && midnight_sky.z < 0.05 && mid_night > 0.99,
        "Midnight sky must be space black",
    );
}

#[test]
fn test_tuned_cadence_custom_duration_and_frozen_time() {
    let custom = TunedCadence::terrestrial(600.0);
    assert_eq!(custom.duration_seconds(), 600.0);
    assert!((custom.day_progress(150.0) - 0.25).abs() < 1e-4);

    let frozen = TunedCadence::terrestrial(0.0);
    assert_eq!(frozen.duration_seconds(), 0.0);
    assert_eq!(frozen.day_progress(100.0), 0.25, "Frozen cadence must maintain stationary noon");
}

#[test]
fn test_file_celestial_reads_duration_and_handles_missing_file() {
    let temp_path = format!("{}/test_server_celestial.toml", std::env::temp_dir().display());
    fs::write(&temp_path, "[world]\ncycle = 10\n").expect("write temporary celestial toml");

    let file_cadence = FileCelestial::from_path(
        &temp_path,
        Box::new(TerrestrialCadence::new()),
    );
    assert_eq!(file_cadence.duration_seconds(), 600.0);
    fs::remove_file(&temp_path).expect("remove temporary celestial toml");

    let missing = FileCelestial::from_path(
        "non_existent_file.toml",
        Box::new(TerrestrialCadence::new()),
    );
    assert_eq!(missing.duration_seconds(), 1200.0);
}

#[test]
fn test_argument_celestial_parses_cli_flags() {
    let args = vec![
        "low-cube".to_string(),
        "--cycle".to_string(),
        "5".to_string(),
    ];
    let arg_cadence = ArgumentCelestial::from_arguments(&args, Box::new(TerrestrialCadence::new()));
    assert_eq!(arg_cadence.duration_seconds(), 300.0);

    let equals_args = vec!["low-cube".to_string(), "--cycle=10".to_string()];
    let equals_cadence =
        ArgumentCelestial::from_arguments(&equals_args, Box::new(TerrestrialCadence::new()));
    assert_eq!(equals_cadence.duration_seconds(), 600.0);

    let minutes_args = vec!["low-cube".to_string(), "--cycle=20".to_string()];
    let min_cadence =
        ArgumentCelestial::from_arguments(&minutes_args, Box::new(TerrestrialCadence::new()));
    assert_eq!(min_cadence.duration_seconds(), 1200.0);

    let empty_args = vec!["low-cube".to_string()];
    let empty_cadence =
        ArgumentCelestial::from_arguments(&empty_args, Box::new(TerrestrialCadence::new()));
    assert_eq!(empty_cadence.duration_seconds(), 1200.0);
}

#[test]
fn test_cycle_duration_parsing() {
    use crate::cycle_duration::CycleDuration;
    assert_eq!(CycleDuration::parse("20").unwrap().seconds(), 1200.0);
    assert_eq!(CycleDuration::parse("10").unwrap().seconds(), 600.0);
    assert_eq!(CycleDuration::parse("5").unwrap().seconds(), 300.0);
    assert_eq!(CycleDuration::parse("0").unwrap().seconds(), 0.0);
    assert!(CycleDuration::parse("invalid").is_none());
}

#[test]
fn test_environment_celestial_fallback_when_unset() {
    let env_cadence = EnvironmentCelestial::process(Box::new(TerrestrialCadence::new()));
    if std::env::var("LOW_CUBE_CYCLE").is_err() {
        assert_eq!(env_cadence.duration_seconds(), 1200.0);
    }
}

#[test]
fn test_server_celestial_assembly_and_cloning() {
    let assembled = ServerCelestial::assembled();
    assert!(assembled.duration_seconds() > 0.0);
    let cloned = assembled.clone_cadence();
    assert_eq!(cloned.duration_seconds(), assembled.duration_seconds());
}

#[test]
fn test_diurnal_pipeline_construction() {
    let cadence = Box::new(TerrestrialCadence::new());
    let now = Instant::now();
    let clock = Box::new(FrozenClock::new(now));
    let epoch = Box::new(MeasuredEpoch::new(
        cadence,
        Box::new(NightlyAuroraCadence::new()),
        clock,
        now,
    ));
    let pipeline = DiurnalPipeline::new(
        epoch,
        Box::new(VoidFirmament::new()),
        Mutex::new(VaultEphemeris::dormant()),
    );
    assert!(std::mem::size_of_val(&pipeline) > 0);
}

#[test]
fn test_void_firmament_construction() {
    let void_sky = VoidFirmament::new();
    let mesh = VoidMesh::new();
    assert_eq!(std::mem::size_of_val(&void_sky), 0);
    assert_eq!(std::mem::size_of_val(&mesh), 0);
}

#[test]
fn test_firmament_shader_contains_aurora_and_time() {
    let wgsl = engine::FIRMAMENT_LUT_WGSL;
    assert!(wgsl.contains("pole_time"));
    assert!(wgsl.contains("u_aurora_factor"));
    assert!(wgsl.contains("t_sky_lut"));
    assert!(wgsl.contains("planet_cubes"));
}

#[test]
fn test_apparent_solar_cube_swells_between_aphelion_and_perihelion() {
    use std::f64::consts::TAU;
    use glam::Vec3;
    use world::{CelestialAtlas, SolarEphemeris, SolarProportions};
    use crate::rayleigh_shell::{AtlasSkyProjection, SkyProjection};

    let prop = SolarProportions::true_scale(64);
    let au = prop.astronomical_unit();
    let eph0 = SolarEphemeris::new(prop, 0.0);
    let sun_gm = eph0.snapshot_all(0.0)[0].gm();
    let year_secs = TAU * (au * au * au / sun_gm).sqrt();

    let (mut min_ang, mut max_ang) = (f32::MAX, 0.0_f32);
    for step in 0..48 {
        let eph = eph0.advanced(year_secs * (f64::from(step) / 48.0));
        let ang = AtlasSkyProjection::solar(CelestialAtlas::heliosphere(eph))
            .project_from(Vec3::new(0.0, 33.0, 0.0))
            .transfer(|cubes, _| cubes[7][3]);
        min_ang = min_ang.min(ang);
        max_ang = max_ang.max(ang);
    }
    assert!(max_ang > min_ang * 1.45);
}

#[test]
fn test_perihelion_solar_flux_brightens_daytime_sky_scatter() {
    use glam::Vec3;
    use crate::rayleigh_shell::{AtmosphericEnvelope, AtmosphericShell};

    let shell = AtmosphericShell::terrestrial_at(Vec3::new(17.0, 0.0, 0.0));
    let (_, peri_sky, _) = shell.sky_illumination_with_flux(Vec3::X, 0.0058);
    let (_, aphel_sky, _) = shell.sky_illumination_with_flux(Vec3::X, 0.0038);
    assert!(peri_sky.length() > aphel_sky.length());
}

#[test]
fn test_half_diurnal_spin_swings_firmament_sun_across_sky_vault() {
    use glam::Vec3;
    use world::{ApexSpin, CelestialAtlas, PlanetarySpin, SolarEphemeris, SolarProportions};
    use crate::rayleigh_shell::{AtlasSkyProjection, SkyProjection};

    let eph0 = SolarEphemeris::new(SolarProportions::true_scale(64), 0.0);
    let spin = ApexSpin::from_ephemeris(eph0);
    let sun_t0 = AtlasSkyProjection::solar(CelestialAtlas::heliosphere(eph0))
        .project_from(Vec3::new(0.0, 33.0, 0.0))
        .transfer(|cubes, _| Vec3::new(cubes[7][0], cubes[7][1], cubes[7][2]));
    let sun_half_day = AtlasSkyProjection::solar(CelestialAtlas::heliosphere(
        eph0.advanced(spin.epoch_after_turns(0.5)),
    ))
    .project_from(Vec3::new(0.0, 33.0, 0.0))
    .transfer(|cubes, _| Vec3::new(cubes[7][0], cubes[7][1], cubes[7][2]));
    assert!((sun_t0 - sun_half_day).length() > 1.1);
}

#[test]
fn test_earth_keplerian_locus_traces_focal_displaced_ellipse() {
    use glam::DVec3;
    use world::{BodyId, OrbitalLocus, SolarCatalog};

    let earth_orbit = SolarCatalog::new().spec_for(BodyId::EARTH).expect("Earth spec").orbit();
    let (mut min_focus_d, mut max_focus_d) = (f64::MAX, 0.0_f64);
    earth_orbit.trace_ellipse(DVec3::ZERO, 100.0, 48, &mut |a, b| {
        min_focus_d = min_focus_d.min(a.length()).min(b.length());
        max_focus_d = max_focus_d.max(a.length()).max(b.length());
    });
    assert!((min_focus_d - 80.0).abs() < 0.5 && (max_focus_d - 120.0).abs() < 0.5);
}
