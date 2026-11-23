use glam::Vec3;
use super::{
    AirSample, AirStratum, AtmosphericEnvelope, AtmosphericShell, BarometricStratum,
    CobaltShadowArch, CrimsonDusk, CubicAirVolume, CubicRayleighShell, HorizonHazeMedium,
    OpticalMedium, PlateauZenith, SolarFluxMedium, TropopauseRim, VacuumMedium, ZenithField,
};

#[test]
fn test_climate_zone_seam_crossing_preserves_continuous_day_night_sky() {
    let sun_dir = Vec3::new(-0.7, 0.7, 0.0).normalize();
    let mut prev = AtmosphericShell::terrestrial_at(Vec3::new(14.5, 17.0, 0.0))
        .sky_illumination(sun_dir);
    for step in 1..=64 {
        let t = (step as f32) / 64.0;
        let cam = if t <= 0.5 {
            Vec3::new(14.5 + t * 5.0, 17.0, 0.0)
        } else {
            Vec3::new(17.0, 17.0 - (t - 0.5) * 5.0, 0.0)
        };
        let cur = AtmosphericShell::terrestrial_at(cam).sky_illumination(sun_dir);
        assert!(
            cur.0.dot(prev.0) > 0.99,
            "Plateau sky zenith must rotate smoothly across climate seam at step {step}"
        );
        assert!(
            (cur.1 - prev.1).length() < 0.16 && (cur.2 - prev.2).abs() < 0.28,
            "Sky color and night factor must transition smoothly at step {step}: prev={prev:?}, cur={cur:?}"
        );
        prev = cur;
    }
}

#[test]
fn test_sunlit_cube_edge_preserves_azure_blue_sky_without_false_red_rim() {
    let shell = CubicRayleighShell::new(64.0, 19.2, Vec3::new(0.18, 0.56, 1.00), 1.0);
    let afternoon_sun = Vec3::new(-0.45, 0.893, 0.0).normalize();
    let edge_cam = Vec3::new(58.0, 65.5, 0.0);
    let (zenith_rgb, _) = shell.scatter_along(edge_cam, Vec3::Y, afternoon_sun);
    let (horizon_rgb, _) =
        shell.scatter_along(edge_cam, Vec3::new(0.95, 0.31, 0.0).normalize(), afternoon_sun);

    assert!(
        zenith_rgb.z > zenith_rgb.x * 2.5 && horizon_rgb.z > horizon_rgb.x * 2.2,
        "Daytime continent edge must remain azure blue instead of turning red: zenith={zenith_rgb:?}, horizon={horizon_rgb:?}"
    );
}

#[test]
fn test_directional_sunset_glows_crimson_sunward_and_cobalt_antisolar() {
    let shell = CubicRayleighShell::new(64.0, 19.2, Vec3::new(0.18, 0.56, 1.00), 1.0);
    let setting_sun = Vec3::NEG_X;
    let cam = Vec3::new(0.0, 65.5, 0.0);
    let (west_rgb, _) =
        shell.scatter_along(cam, Vec3::new(-0.94, 0.34, 0.0).normalize(), setting_sun);
    let (east_rgb, _) =
        shell.scatter_along(cam, Vec3::new(0.94, 0.34, 0.0).normalize(), setting_sun);

    assert!(west_rgb.x > west_rgb.z, "Sunward western horizon must glow crimson-gold, got {west_rgb:?}");
    assert!(east_rgb.z > east_rgb.x * 2.0, "Anti-solar eastern horizon must show cool cobalt shadow arch, got {east_rgb:?}");
}

#[test]
fn test_plateau_zenith_stays_flat_across_continent_and_bisects_seam() {
    let field = PlateauZenith::new();
    let interior = field.zenith_at(Vec3::new(45.0, 65.0, 0.0));
    let seam = field.zenith_at(Vec3::new(65.0, 65.0, 0.0));
    assert!(interior.y > 0.985, "PlateauZenith must remain flat across 70% of continent, got {interior:?}");
    assert!((seam.x - seam.y).abs() < 1e-4, "PlateauZenith must bisect 45° cube edge symmetrically");
}

#[test]
fn test_cubic_atmosphere_fills_square_ninety_degree_corners() {
    let shell = CubicRayleighShell::new(64.0, 19.2, Vec3::new(0.18, 0.56, 1.00), 1.0);
    let sun_dir = Vec3::new(1.0, 1.0, 1.0).normalize();
    let (corner_rgb, corner_alpha) =
        shell.scatter_along(Vec3::new(76.0, 260.0, 76.0), Vec3::NEG_Y, sun_dir);
    assert!(corner_rgb.z > 0.15 && corner_alpha > 0.10);
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
fn test_vacuum_and_stratum_decorators_compose_cobalt_dusk_rim_and_flux() {
    assert_eq!(VacuumMedium::new().scatter_along(Vec3::Y * 20.0, Vec3::Y, Vec3::Y), (Vec3::ZERO, 0.0));
    let sunward_pt = AirSample::new(Vec3::Y * 70.0, 0.2, Vec3::X, Vec3::X);
    let antisolar_pt = AirSample::new(Vec3::Y * 70.0, 0.2, Vec3::NEG_X, Vec3::X);
    let cobalt = CobaltShadowArch::terrestrial(BarometricStratum::new(Vec3::new(0.18, 0.56, 1.00)));
    let dusk = CrimsonDusk::directional(cobalt, 1.0);
    assert!(dusk.sample_point(&sunward_pt).1.x > cobalt.sample_point(&sunward_pt).1.x + 0.5);
    assert_eq!(dusk.sample_point(&antisolar_pt).1, cobalt.sample_point(&antisolar_pt).1);
    let ceiling_pt = AirSample::new(Vec3::Y * 80.0, 0.88, Vec3::Y, Vec3::Y);
    let rimmed = TropopauseRim::cubic_ceiling(dusk);
    assert!(rimmed.sample_point(&ceiling_pt).0 > dusk.sample_point(&ceiling_pt).0 + 0.15);
    let vol = CubicAirVolume::new(64.0, 19.2, rimmed);
    let boosted = SolarFluxMedium::new(HorizonHazeMedium::pale(vol), 1.2);
    assert!(boosted.scatter_along(Vec3::new(0.0, 65.5, 0.0), Vec3::Y, Vec3::Y).0.z > vol.scatter_along(Vec3::new(0.0, 65.5, 0.0), Vec3::Y, Vec3::Y).0.z);
}
