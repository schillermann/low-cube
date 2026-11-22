use std::fs;
use std::time::Instant;
use crate::argument_celestial::ArgumentCelestial;
use crate::celestial::{CelestialCadence, TerrestrialCadence, TunedCadence};
use crate::clock::FrozenClock;
use crate::diurnal_pipeline::DiurnalPipeline;
use crate::environment_celestial::EnvironmentCelestial;
use crate::file_celestial::FileCelestial;
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
    use crate::rayleigh_shell::AtmosphericShell;
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
    fs::write(&temp_path, "[world]\ncycle = 10\n").unwrap();

    let file_cadence = FileCelestial::from_path(
        &temp_path,
        Box::new(TerrestrialCadence::new()),
    );
    assert_eq!(file_cadence.duration_seconds(), 600.0);
    let _ = fs::remove_file(temp_path);

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

    let equals_args = vec![
        "low-cube".to_string(),
        "--cycle=10".to_string(),
    ];
    let equals_cadence = ArgumentCelestial::from_arguments(&equals_args, Box::new(TerrestrialCadence::new()));
    assert_eq!(equals_cadence.duration_seconds(), 600.0);

    let minutes_args = vec![
        "low-cube".to_string(),
        "--cycle=20".to_string(),
    ];
    let min_cadence = ArgumentCelestial::from_arguments(&minutes_args, Box::new(TerrestrialCadence::new()));
    assert_eq!(min_cadence.duration_seconds(), 1200.0);

    let empty_args = vec!["low-cube".to_string()];
    let empty_cadence = ArgumentCelestial::from_arguments(&empty_args, Box::new(TerrestrialCadence::new()));
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
    let pipeline = DiurnalPipeline::headless(cadence, clock, now);
    assert!(std::mem::size_of_val(&pipeline) > 0);
}

#[test]
fn test_void_firmament_construction() {
    use crate::firmament::{VoidFirmament, VoidMesh};
    let void_sky = VoidFirmament::new();
    let mesh = VoidMesh::new();
    assert_eq!(std::mem::size_of_val(&void_sky), 0);
    assert_eq!(std::mem::size_of_val(&mesh), 0);
}

#[test]
fn test_firmament_shader_contains_aurora_and_time() {
    use crate::firmament_glsl::FIRMAMENT_FRAGMENT_SRC;
    assert!(FIRMAMENT_FRAGMENT_SRC.contains("u_time"));
    assert!(FIRMAMENT_FRAGMENT_SRC.contains("polar_aurora"));
    assert!(FIRMAMENT_FRAGMENT_SRC.contains("u_night_factor"));
}

#[test]
fn test_heliosphere_corner_spin_perihelion_flux_and_focal_ellipse_locus() {
    use std::f64::consts::TAU;
    use glam::{DVec3, Vec3};
    use world::{
        ApexSpin, BodyId, CelestialAtlas, OrbitalLocus, PlanetarySpin, SolarCatalog,
        SolarEphemeris, SolarProportions,
    };
    use crate::rayleigh_shell::{AtmosphericShell, FirmamentPlanets};

    let prop = SolarProportions::true_scale(64);
    let au = prop.astronomical_unit();
    let eph0 = SolarEphemeris::new(prop, 0.0);
    let sun_gm = eph0.snapshot_all(0.0)[0].gm();
    let year_secs = TAU * (au * au * au / sun_gm).sqrt();

    let (mut min_ang, mut max_ang) = (f32::MAX, 0.0_f32);
    for step in 0..48 {
        let eph = eph0.advanced(year_secs * (f64::from(step) / 48.0));
        let ang = FirmamentPlanets::from_atlas(CelestialAtlas::heliosphere(eph), Vec3::new(0.0, 33.0, 0.0))
            .transfer(|cubes, _| cubes[7][3]);
        min_ang = min_ang.min(ang);
        max_ang = max_ang.max(ang);
    }
    assert!(
        max_ang > min_ang * 1.45,
        "Apparent cubic Sun must swell ~1.5x between Aphelion ({min_ang}) and Perihelion ({max_ang})",
    );

    let shell = AtmosphericShell::terrestrial_at(Vec3::new(17.0, 0.0, 0.0));
    let (_, peri_sky, _) = shell.sky_illumination_with_flux(Vec3::X, max_ang);
    let (_, aphel_sky, _) = shell.sky_illumination_with_flux(Vec3::X, min_ang);
    assert!(peri_sky.length() > aphel_sky.length(), "Perihelion solar flux must brighten daytime sky scatter");

    let spin = ApexSpin::from_ephemeris(eph0);
    let sun_t0 = FirmamentPlanets::from_atlas(CelestialAtlas::heliosphere(eph0), Vec3::new(0.0, 33.0, 0.0))
        .transfer(|cubes, _| Vec3::new(cubes[7][0], cubes[7][1], cubes[7][2]));
    let sun_half_day = FirmamentPlanets::from_atlas(
        CelestialAtlas::heliosphere(eph0.advanced(spin.epoch_after_turns(0.5))),
        Vec3::new(0.0, 33.0, 0.0),
    )
    .transfer(|cubes, _| Vec3::new(cubes[7][0], cubes[7][1], cubes[7][2]));
    assert!(
        (sun_t0 - sun_half_day).length() > 1.1,
        "Half a diurnal spin must swing the Firmament Sun across the sky vault",
    );

    let earth_orbit = SolarCatalog::new().spec_for(BodyId::EARTH).expect("Earth spec").orbit();
    let (mut min_focus_d, mut max_focus_d) = (f64::MAX, 0.0_f64);
    earth_orbit.trace_ellipse(DVec3::ZERO, 100.0, 48, &mut |a, b| {
        min_focus_d = min_focus_d.min(a.length()).min(b.length());
        max_focus_d = max_focus_d.max(a.length()).max(b.length());
    });
    assert!((min_focus_d - 80.0).abs() < 0.5 && (max_focus_d - 120.0).abs() < 0.5);
}
