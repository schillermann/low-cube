use glam::Vec3;

pub(crate) const LOAM_CROWN: Vec3 = Vec3::new(0.45, 0.30, 0.17);
pub(crate) const LOAM_WALL: Vec3 = Vec3::new(0.41, 0.27, 0.15);
const CLOD_A: Vec3 = Vec3::new(0.52, 0.36, 0.21);
const CLOD_B: Vec3 = Vec3::new(0.46, 0.31, 0.18);
const CLOD_C: Vec3 = Vec3::new(0.34, 0.22, 0.12);
const CLOD_D: Vec3 = Vec3::new(0.57, 0.40, 0.24);
const PEBBLE_LIGHT: Vec3 = Vec3::new(0.64, 0.61, 0.55);
const PEBBLE_DARK: Vec3 = Vec3::new(0.49, 0.46, 0.42);
pub(crate) const TURF_BASE: Vec3 = Vec3::new(0.28, 0.62, 0.24);
const SOD_A: Vec3 = Vec3::new(0.35, 0.72, 0.29);
const SOD_B: Vec3 = Vec3::new(0.30, 0.66, 0.25);
pub(crate) const SOD_C: Vec3 = Vec3::new(0.24, 0.56, 0.21);

pub(crate) const CROWN_PATTERN: [(f32, f32, f32, f32, f32, Vec3); 7] = [
    (0.03, 0.22, 0.03, 0.23, 0.018, CLOD_A),
    (0.25, 0.47, 0.04, 0.21, 0.013, CLOD_B),
    (0.05, 0.26, 0.26, 0.46, 0.015, CLOD_C),
    (0.28, 0.46, 0.24, 0.47, 0.020, CLOD_D),
    (0.14, 0.34, 0.15, 0.35, 0.026, CLOD_A),
    (0.35, 0.44, 0.07, 0.16, 0.028, PEBBLE_LIGHT),
    (0.07, 0.15, 0.34, 0.42, 0.024, PEBBLE_DARK),
];

pub(crate) const TURF_FRINGE_PATTERN: [(f32, f32, f32, f32, f32, Vec3); 3] = [
    (0.04, 0.17, 0.35, 0.44, 0.016, SOD_B),
    (0.20, 0.34, 0.37, 0.44, 0.018, SOD_A),
    (0.37, 0.47, 0.34, 0.44, 0.016, SOD_C),
];

pub(crate) const SOLE_PATTERN: [(f32, f32, f32, f32, f32, Vec3); 7] = [
    (0.04, 0.23, 0.04, 0.22, 0.017, CLOD_B),
    (0.26, 0.46, 0.03, 0.22, 0.014, CLOD_C),
    (0.04, 0.25, 0.25, 0.45, 0.016, CLOD_A),
    (0.27, 0.47, 0.25, 0.46, 0.019, CLOD_D),
    (0.15, 0.35, 0.14, 0.34, 0.024, CLOD_B),
    (0.08, 0.17, 0.07, 0.16, 0.026, PEBBLE_LIGHT),
    (0.33, 0.42, 0.33, 0.41, 0.023, PEBBLE_DARK),
];

pub(crate) const FLANK_PATTERN: [(f32, f32, f32, f32, f32, Vec3); 7] = [
    (0.04, 0.24, 0.04, 0.21, 0.016, CLOD_B),
    (0.27, 0.46, 0.04, 0.19, 0.013, CLOD_C),
    (0.05, 0.28, 0.25, 0.44, 0.018, CLOD_A),
    (0.31, 0.46, 0.23, 0.45, 0.015, CLOD_D),
    (0.15, 0.33, 0.14, 0.31, 0.022, CLOD_B),
    (0.34, 0.43, 0.09, 0.18, 0.025, PEBBLE_LIGHT),
    (0.07, 0.16, 0.30, 0.39, 0.021, PEBBLE_DARK),
];
