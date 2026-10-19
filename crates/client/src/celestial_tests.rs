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
fn test_terrestrial_cadence_sky_color_transitions() {
    let cadence = TerrestrialCadence::new();
    let noon_sky = cadence.sky_color(0.25);
    assert!(noon_sky.z > noon_sky.x, "Noon sky must be dominantly blue");

    let sunset_sky = cadence.sky_color(0.5);
    assert!(sunset_sky.x > 0.7, "Sunset sky must have golden/crimson red tint");

    let midnight_sky = cadence.sky_color(0.75);
    assert!(midnight_sky.x < 0.03 && midnight_sky.y < 0.03 && midnight_sky.z < 0.05, "Midnight sky must be space black");
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
