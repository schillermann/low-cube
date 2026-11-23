use glam::Vec3;
use super::{
    AirStratum, AtmosphericEnvelope, AtmosphericShell, BarometricStratum, CrimsonDusk,
    CubicAirVolume, CubicRayleighShell, HorizonHazeMedium, OpticalMedium, SolarFluxMedium,
    TropopauseRim, VacuumMedium,
};

#[test]
fn test_climate_zone_seam_crossing_preserves_continuous_day_night_sky() {
    let sun_dir = Vec3::new(-0.7, 0.7, 0.0).normalize();
    let mut prev = AtmosphericShell::terrestrial_at(Vec3::new(14.5, 17.0, 0.0))
        .sky_illumination(sun_dir);
    for step in 1..=24 {
        let t = (step as f32) / 24.0;
        let cam = if t <= 0.5 {
            Vec3::new(14.5 + t * 5.0, 17.0, 0.0)
        } else {
            Vec3::new(17.0, 17.0 - (t - 0.5) * 5.0, 0.0)
        };
        let cur = AtmosphericShell::terrestrial_at(cam).sky_illumination(sun_dir);
        assert!(
            cur.0.dot(prev.0) > 0.995,
            "Local sky zenith must rotate smoothly across climate seam at step {step}"
        );
        assert!(
            (cur.1 - prev.1).length() < 0.12 && (cur.2 - prev.2).abs() < 0.15,
            "Sky color and night factor must transition smoothly across zone border"
        );
        prev = cur;
    }
}

#[test]
fn test_cubic_atmosphere_fills_square_ninety_degree_corners() {
    let shell = CubicRayleighShell::new(64.0, 19.2, Vec3::new(0.18, 0.56, 1.00), 1.0);
    let sun_dir = Vec3::new(1.0, 1.0, 1.0).normalize();
    let (corner_rgb, corner_alpha) =
        shell.scatter_along(Vec3::new(76.0, 260.0, 76.0), Vec3::NEG_Y, sun_dir);
    assert!(
        corner_rgb.z > 0.15 && corner_alpha > 0.10,
        "Cubic atmosphere must fill its square 90° corners up to body_half + thickness"
    );
}

#[test]
fn test_rocket_ascent_penetrates_flat_cubic_atmosphere_ceiling_into_vacuum() {
    let shell = CubicRayleighShell::new(64.0, 19.2, Vec3::new(0.18, 0.56, 1.00), 1.0);
    let (pad_up, pad_a) = shell.scatter_along(Vec3::new(0.0, 65.5, 0.0), Vec3::Y, Vec3::Y);
    let (strato_up, strato_a) = shell.scatter_along(Vec3::new(0.0, 79.0, 0.0), Vec3::Y, Vec3::Y);
    let (space_up, space_a) = shell.scatter_along(Vec3::new(0.0, 84.5, 0.0), Vec3::Y, Vec3::Y);
    let (space_down, down_a) =
        shell.scatter_along(Vec3::new(0.0, 84.5, 0.0), Vec3::NEG_Y, Vec3::Y);

    assert!(pad_up.z > 0.75 && pad_a > 0.55);
    assert!(strato_up.z < pad_up.z && strato_a < pad_a && strato_a > 0.05);
    assert_eq!((space_up, space_a), (Vec3::ZERO, 0.0));
    assert!(space_down.z > 0.70 && down_a > 0.55);
}

#[test]
fn test_vacuum_medium_returns_zero_scatter() {
    let sample = VacuumMedium::new().scatter_along(Vec3::Y * 20.0, Vec3::Y, Vec3::Y);
    assert_eq!(sample, (Vec3::ZERO, 0.0));
}

#[test]
fn test_crimson_dusk_decorator_enriches_terminator_red_scatter() {
    let day_only = BarometricStratum::new(Vec3::new(0.18, 0.56, 1.00));
    let with_dusk = CrimsonDusk::terrestrial(day_only);
    let (_, raw_rgb, _) = day_only.sample_point(Vec3::Y * 70.0, 0.2, Vec3::X);
    let (_, dusk_rgb, _) = with_dusk.sample_point(Vec3::Y * 70.0, 0.2, Vec3::X);
    assert!(dusk_rgb.x > raw_rgb.x + 0.5);
}

#[test]
fn test_tropopause_rim_decorator_intensifies_cubic_ceiling_sheen() {
    let lower = CrimsonDusk::terrestrial(BarometricStratum::new(Vec3::new(0.18, 0.56, 1.00)));
    let rimmed = TropopauseRim::cubic_ceiling(lower);
    let (raw_rho, _, _) = lower.sample_point(Vec3::Y * 80.0, 0.88, Vec3::Y);
    let (rim_rho, _, _) = rimmed.sample_point(Vec3::Y * 80.0, 0.88, Vec3::Y);
    assert!(rim_rho > raw_rho + 0.15);
}

#[test]
fn test_solar_flux_and_horizon_haze_decorators_amplify_cubic_volume() {
    let stratum = TropopauseRim::cubic_ceiling(CrimsonDusk::terrestrial(
        BarometricStratum::new(Vec3::new(0.18, 0.56, 1.00)),
    ));
    let volume = CubicAirVolume::new(64.0, 19.2, stratum);
    let boosted = SolarFluxMedium::new(HorizonHazeMedium::pale(volume), 1.2);
    let (raw_rgb, _) = volume.scatter_along(Vec3::new(0.0, 65.5, 0.0), Vec3::Y, Vec3::Y);
    let (boost_rgb, _) = boosted.scatter_along(Vec3::new(0.0, 65.5, 0.0), Vec3::Y, Vec3::Y);
    assert!(boost_rgb.z > raw_rgb.z);
}
