use crate::argument_aurora::ArgumentAurora;
use crate::aurora_cadence::{
    AuroraCadence, DormantAuroraCadence, NightlyAuroraCadence, StochasticAuroraCadence,
};
use crate::aurora_canopy::{CalibratedCanopy, VoidCanopy};
use crate::file_aurora::FileAurora;
use crate::firmament_glsl::{FIRMAMENT_FRAGMENT_SRC, FIRMAMENT_VERTEX_SRC};

#[test]
fn test_calibrated_canopy_small_world_never_exceeds_arctic_face() {
    let small_world_blocks = 16;
    let canopy = CalibratedCanopy::planetary(small_world_blocks);
    let world_half = 8.0;

    canopy.transfer(|half, radius, ceiling| {
        assert_eq!(half, world_half);
        assert!(
            radius <= world_half * 0.85,
            "Aurora radius ({radius}) must not exceed Arctic face half-size ({world_half})"
        );
        assert_eq!(radius, 6.4);
        assert_eq!(ceiling, 8.0);
    });
}

#[test]
fn test_calibrated_canopy_large_world_not_too_small_and_capped_at_max() {
    let large_world_blocks = 1024;
    let canopy = CalibratedCanopy::planetary(large_world_blocks);

    canopy.transfer(|half, radius, ceiling| {
        assert_eq!(half, 512.0);
        assert!(radius >= 14.0, "Aurora radius must not be too small");
        assert!(radius <= 96.0, "Aurora radius must not exceed max limit");
        assert_eq!(radius, 96.0);
        assert_eq!(ceiling, 48.0);
    });
}

#[test]
fn test_calibrated_canopy_medium_world_scales_between_bounds() {
    let medium_world_blocks = 128;
    let canopy = CalibratedCanopy::planetary(medium_world_blocks);

    canopy.transfer(|half, radius, ceiling| {
        assert_eq!(half, 64.0);
        assert!(radius >= 14.0 && radius <= 96.0);
        assert!((radius - 38.4).abs() < 1e-4);
        assert_eq!(ceiling, 25.6);
    });
}

#[test]
fn test_void_canopy_null_object_pattern() {
    let void_canopy = VoidCanopy::new();
    assert_eq!(std::mem::size_of_val(&void_canopy), 0);
}

#[test]
fn test_firmament_glsl_contains_camera_pos_and_aurora_uniforms() {
    assert!(FIRMAMENT_VERTEX_SRC.contains("v_cam_pos"));
    assert!(FIRMAMENT_FRAGMENT_SRC.contains("v_cam_pos"));
    assert!(FIRMAMENT_FRAGMENT_SRC.contains("u_face_half"));
    assert!(FIRMAMENT_FRAGMENT_SRC.contains("u_aurora_radius"));
    assert!(FIRMAMENT_FRAGMENT_SRC.contains("u_ceiling_altitude"));
    assert!(FIRMAMENT_FRAGMENT_SRC.contains("u_aurora_factor"));
}

#[test]
fn test_dormant_aurora_cadence_always_zero() {
    let dormant = DormantAuroraCadence::new();
    assert_eq!(dormant.activity(0.75, 500.0, 1.0), 0.0);
    assert_eq!(dormant.activity(0.20, 100.0, 0.0), 0.0);
}

#[test]
fn test_nightly_aurora_cadence_scales_with_night_factor() {
    let nightly = NightlyAuroraCadence::new();
    assert_eq!(nightly.activity(0.75, 500.0, 0.85), 0.85);
    assert_eq!(nightly.activity(0.20, 100.0, 0.0), 0.0);
}

#[test]
fn test_stochastic_aurora_cadence_daylight_and_bounds() {
    let stochastic = StochasticAuroraCadence::new(1.0, 1200.0, Box::new(NightlyAuroraCadence::new()));
    // In full daylight (night_factor = 0), activity is always 0.0
    assert_eq!(stochastic.activity(0.25, 300.0, 0.0), 0.0);
    // Outside the nocturnal window (e.g., at dawn progress 0.99), activity is 0.0
    assert_eq!(stochastic.activity(0.99, 1188.0, 1.0), 0.0);
    // At zero chance, activity is always 0.0
    let never = StochasticAuroraCadence::new(0.0, 1200.0, Box::new(NightlyAuroraCadence::new()));
    assert_eq!(never.activity(0.75, 900.0, 1.0), 0.0);
}

#[test]
fn test_argument_aurora_parsing_and_fallback() {
    let fallback = Box::new(NightlyAuroraCadence::new());
    let off_args = vec!["app".into(), "--aurora".into(), "off".into()];
    let off = ArgumentAurora::from_arguments(&off_args, 1200.0, fallback.clone_cadence());
    assert_eq!(off.activity(0.75, 900.0, 1.0), 0.0);

    let nightly_args = vec!["app".into(), "--aurora=nightly".into()];
    let nightly = ArgumentAurora::from_arguments(&nightly_args, 1200.0, Box::new(DormantAuroraCadence::new()));
    assert_eq!(nightly.activity(0.75, 900.0, 0.7), 0.7);

    let empty_args = vec!["app".into()];
    let passthrough = ArgumentAurora::from_arguments(&empty_args, 1200.0, Box::new(NightlyAuroraCadence::new()));
    assert_eq!(passthrough.activity(0.75, 900.0, 0.5), 0.5);
}

#[test]
fn test_file_aurora_from_path() {
    let fallback = Box::new(DormantAuroraCadence::new());
    // Missing file safely falls back
    let missing = FileAurora::from_path("non_existent_file.toml", 1200.0, fallback.clone_cadence());
    assert_eq!(missing.activity(0.75, 900.0, 0.9), 0.0);

    let temp_path = std::env::temp_dir().join("test_aurora_config.toml");
    std::fs::write(&temp_path, "[aurora]\nmode = \"nightly\"\n").expect("write temp config");

    let file = FileAurora::from_path(temp_path.to_str().unwrap(), 1200.0, fallback);
    let _ = std::fs::remove_file(temp_path);
    assert_eq!(file.activity(0.75, 900.0, 0.9), 0.9);
}

#[test]
fn test_stochastic_aurora_cadence_presets() {
    let rare = StochasticAuroraCadence::rare(1200.0, Box::new(NightlyAuroraCadence::new()));
    assert_eq!(rare.activity(0.20, 100.0, 0.0), 0.0);
    let normal = StochasticAuroraCadence::normal(1200.0, Box::new(NightlyAuroraCadence::new()));
    assert_eq!(normal.activity(0.20, 100.0, 0.0), 0.0);
    let frequent = StochasticAuroraCadence::frequent(1200.0, Box::new(NightlyAuroraCadence::new()));
    assert_eq!(frequent.activity(0.20, 100.0, 0.0), 0.0);
}

#[test]
fn test_keyword_aurora_modes_in_cli_and_file() {
    let fallback = Box::new(DormantAuroraCadence::new());

    // CLI --aurora rare
    let args = vec!["app".into(), "--aurora".into(), "rare".into()];
    let rare = ArgumentAurora::from_arguments(&args, 1200.0, fallback.clone_cadence());
    assert_eq!(rare.activity(0.20, 100.0, 0.0), 0.0);

    // CLI --aurora frequent
    let args_freq = vec!["app".into(), "--aurora=frequent".into()];
    let freq = ArgumentAurora::from_arguments(&args_freq, 1200.0, fallback.clone_cadence());
    assert_eq!(freq.activity(0.20, 100.0, 0.0), 0.0);

    // File with mode = "normal"
    let temp_path = std::env::temp_dir().join("test_aurora_normal.toml");
    std::fs::write(&temp_path, "[aurora]\nmode = \"normal\"\n").expect("write temp config");
    let file = FileAurora::from_path(temp_path.to_str().unwrap(), 1200.0, fallback);
    let _ = std::fs::remove_file(temp_path);
    assert_eq!(file.activity(0.20, 100.0, 0.0), 0.0);
}
