use glam::Vec3;
use world::BodyId;
use super::{
    AirCatalog, AirSample, AirStratum, AtmosphericEnvelope, AtmosphericShell, BarometricStratum,
    CobaltShadowArch, CrimsonDusk, CubicAirVolume, CubicRayleighShell, HorizonHazeMedium,
    IonosphereMedium, Magnetosphere, OpticalMedium, PlateauZenith, PolarIonosphere,
    SolarAirCatalog, SolarFluxMedium, TropopauseRim, VacuumMedium, ZenithField,
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

#[test]
fn test_arctic_ionosphere_centers_symmetrically_over_arctic_face_not_corner() {
    let iono = PolarIonosphere::planetary(64.0, Magnetosphere::terrestrial_arctic(), 1.0, 0.0);
    let cam = Vec3::new(-65.5, 0.0, 0.0);
    let midnight_sun = Vec3::X;

    let zenith_eye = iono.radiance_along(cam, Vec3::NEG_X, midnight_sun);
    let north = iono.radiance_along(cam, Vec3::new(-1.0, 0.85, 0.0).normalize(), midnight_sun);
    let south = iono.radiance_along(cam, Vec3::new(-1.0, -0.85, 0.0).normalize(), midnight_sun);
    let east = iono.radiance_along(cam, Vec3::new(-1.0, 0.0, 0.85).normalize(), midnight_sun);
    let west = iono.radiance_along(cam, Vec3::new(-1.0, 0.0, -0.85).normalize(), midnight_sun);

    assert_eq!(zenith_eye, Vec3::ZERO, "Central L^4 oval eye directly at (-1,0,0) zenith must stay clear");
    assert!(north.y > 0.12 && south.y > 0.12 && east.y > 0.12 && west.y > 0.12);
    assert!(
        (north.length() - south.length()).abs() < 0.08 && (east.length() - west.length()).abs() < 0.08,
        "Aurora oval must frame all four sides of Arctic symmetrically: N={north:?}, S={south:?}, E={east:?}, W={west:?}"
    );
}

#[test]
fn test_temperate_and_steppe_clear_zenith_with_low_western_arctic_compass_glow() {
    let iono = PolarIonosphere::planetary(64.0, Magnetosphere::terrestrial_arctic(), 1.0, 0.0);
    let midnight_sun = Vec3::X;
    let temp_cam = Vec3::new(0.0, 65.5, 0.0);
    let steppe_cam = Vec3::new(0.0, 0.0, -65.5);

    assert_eq!(iono.radiance_along(temp_cam, Vec3::Y, midnight_sun), Vec3::ZERO);
    assert_eq!(iono.radiance_along(steppe_cam, Vec3::NEG_Z, midnight_sun), Vec3::ZERO);
    assert_eq!(
        iono.radiance_along(temp_cam, Vec3::new(-1.0, 1.0, -1.0).normalize(), midnight_sun),
        Vec3::ZERO,
        "Northern tri-corner must no longer host overhead Temperate aurora"
    );

    let west_compass =
        iono.radiance_along(temp_cam, Vec3::new(-1.0, 0.12, 0.0).normalize(), midnight_sun);
    assert!(
        west_compass.length() > 0.01 && west_compass.length() < 0.45,
        "Low -X western horizon from Temperate should show gentle distant Arctic compass glow, got {west_compass:?}"
    );
}

#[test]
fn test_rocket_penetrates_3d_curtains_and_orbit_views_nightside_crown() {
    let iono = PolarIonosphere::planetary(64.0, Magnetosphere::terrestrial_arctic(), 1.0, 4.0);
    let midnight_sun = Vec3::X;

    let inside_curtain = iono.radiance_along(
        Vec3::new(-108.0, 36.0, 0.0),
        Vec3::new(-0.35, 0.90, 0.25).normalize(),
        midnight_sun,
    );
    let orbital_down = iono.radiance_along(
        Vec3::new(-220.0, 38.0, 0.0),
        Vec3::X,
        midnight_sun,
    );
    let desert_blocked = iono.radiance_along(Vec3::new(65.5, 0.0, 0.0), Vec3::NEG_X, midnight_sun);
    let arctic_daylight =
        iono.radiance_along(Vec3::new(-65.5, 0.0, 0.0), Vec3::new(-1.0, 0.85, 0.0).normalize(), Vec3::NEG_X);

    assert!(inside_curtain.length() > 0.05, "Spacecraft inside ionosphere must fly through glowing 3D curtains");
    assert!(orbital_down.length() > 0.05, "Orbital observer must see nightside Arctic aurora crown");
    assert_eq!(desert_blocked, Vec3::ZERO, "Solid planet core must occlude Arctic aurora from Desert (+X)");
    assert_eq!(arctic_daylight, Vec3::ZERO, "Sunlit Arctic noon ionosphere must remain dark");
}

#[test]
fn test_large_continental_world_and_alien_magnetospheres_scale_cleanly() {
    let continental = PolarIonosphere::planetary(512.0, Magnetosphere::terrestrial_arctic(), 1.0, 2.0);
    let glow = continental.radiance_along(
        Vec3::new(-513.5, 0.0, 0.0),
        Vec3::new(-1.0, 0.85, 0.0).normalize(),
        Vec3::X,
    );
    assert!(glow.y > 0.10, "512m continental Arctic world must scale ionosphere without clipping underground");

    let cat = SolarAirCatalog::new();
    assert!(cat.shell_for(BodyId::EARTH, Vec3::ZERO, 64.0).magnetized());
    assert!(cat.shell_for(BodyId::TITAN, Vec3::ZERO, 26.0).magnetized());
    assert!(cat.shell_for(BodyId::MARS, Vec3::ZERO, 34.0).magnetized());
    assert!(!cat.shell_for(BodyId::VENUS, Vec3::ZERO, 60.0).magnetized());
    assert!(!cat.shell_for(BodyId::LUNA, Vec3::ZERO, 18.0).magnetized());

    let titan = PolarIonosphere::planetary(26.0, Magnetosphere::saturnian_methane(), 1.0, 1.0)
        .radiance_along(Vec3::new(-27.5, 0.0, 0.0), Vec3::new(-1.0, 0.85, 0.0).normalize(), Vec3::X);
    assert!(titan.z > titan.y, "Titan methane magnetosphere must emit deep indigo-amber spectrum: {titan:?}");
}

#[test]
fn test_rainstorm_overcast_darkens_daytime_sky_without_triggering_false_night_aurora() {
    let cat = SolarAirCatalog::new();
    let cam = Vec3::new(0.0, 65.5, 0.0);
    let noon_sun = Vec3::Y;
    let sunny = cat.shell_for(BodyId::EARTH, cam, 64.0);
    let stormy = sunny.darkened(0.90);

    let (_, sunny_sky, sunny_night) = sunny.sky_illumination(noon_sun);
    let (_, storm_sky, storm_night) = stormy.sky_illumination(noon_sun);

    let sunny_lum = sunny_sky.x + sunny_sky.y + sunny_sky.z;
    let storm_lum = storm_sky.x + storm_sky.y + storm_sky.z;
    assert!(
        storm_lum < sunny_lum * 0.52,
        "Rainstorm overcast must darken daytime sky strongly: sunny={sunny_sky:?}, storm={storm_sky:?}"
    );
    assert_eq!(sunny_night, 0.0);
    assert_eq!(
        storm_night, 0.0,
        "Daytime rainstorm darkening must never trigger nocturnal stars or aurora"
    );
}

#[test]
fn test_overcast_canopy_epoch_dims_ground_sunbeam_and_rain_ear_motor_emits_shower_cues() {
    use glam::{Mat4, Vec4};
    use crate::avatar::{AvatarMotor as _, PlanetaryAvatar};
    use crate::controls::{ActionBindings, KeyMotion};
    use crate::diurnal_pipeline::{DiurnalEpoch, OvercastCanopyEpoch};
    use crate::firmament::{VaultEphemeris, VaultGlow, VaultView};

    struct NoonEpoch;
    impl DiurnalEpoch for NoonEpoch {
        fn evaluate_pass(&self, ivp: Mat4, _cam: Vec3, _eph: &VaultEphemeris) -> (Vec3, VaultView, VaultGlow) {
            let sun = Vec3::new(0.0, 1.0, 0.0);
            (sun, VaultView::new(ivp, sun, Vec3::Y), VaultGlow::new(Vec4::ONE, 0.0, 0.0))
        }
    }

    let cat = SolarAirCatalog::new();
    let cam = Vec3::new(0.0, 65.5, 0.0);
    let shell = cat.shell_for(BodyId::EARTH, cam, 64.0);
    let mut clear_cubes = [[0.0_f32; 4]; 8];
    clear_cubes[7] = [0.0, 1.0, 0.0, 0.00465];
    let clear_eph = VaultEphemeris::active(clear_cubes, [0.0; 4], shell);
    let mut storm_cubes = clear_cubes;
    storm_cubes[7] = [0.0, 0.42, 0.0, 0.00465];
    let storm_eph = VaultEphemeris::active(storm_cubes, [0.0; 4], shell.darkened(0.90));

    let canopy = OvercastCanopyEpoch::new(NoonEpoch);
    let (clear_sun, _, _) = canopy.evaluate_pass(Mat4::IDENTITY, cam, &clear_eph);
    let (storm_sun, _, _) = canopy.evaluate_pass(Mat4::IDENTITY, cam, &storm_eph);
    assert!((clear_sun.length() - 1.0).abs() < 1e-4);
    assert!(storm_sun.length() < 0.48 && storm_sun.normalize().dot(Vec3::Y) > 0.999);

    let prop = world::SolarProportions::compressed(32, 0.01);
    let controls = KeyMotion::new(std::collections::HashSet::new(), ActionBindings::seeded());
    let mut shower_cues = Vec::new();
    for step_t in 0..48 {
        let helio = world::HeliosphereWorld::assembled(
            world::BiomedCubeWorld::barren_beveled(32, 1.0),
            world::SolarEphemeris::new(prop, f64::from(step_t) * 24.0),
            true,
        );
        let probe = Vec3::new(0.0, 2.0, 17.5);
        let mut walker = PlanetaryAvatar::at(probe);
        let captured = acoustics::RainWaveTrace::new(0).capture(|| {
            walker.step_motion(1.0 / 60.0, &controls, &helio);
        });
        shower_cues.extend(captured);
    }
    assert!(
        shower_cues.contains(&acoustics::PrecipitationCue::WoodlandRain),
        "RainEarMotor decorator must emit WoodlandRain cues when stepping under active Jungle rain"
    );
}
