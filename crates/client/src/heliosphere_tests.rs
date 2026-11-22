use glam::{DVec3, Vec3};
use world::{
    AsteroidBelt, BlockMedium, BlockMold, BodyId, CelestialSystem, SolarEphemeris, SolarProportions,
    MAJOR_BODY_COUNT,
};
use crate::rayleigh_shell::{CubicRayleighShell, CubicSolarBody, FirmamentPlanets};

#[test]
fn test_orbital_earth_rayleigh_shell_shows_azure_day_limb_crimson_terminator_and_cubic_sun() {
    let shell = CubicRayleighShell::around_earth(64.0);
    let sun_dir = Vec3::X;

    let cam_day = Vec3::new(68.0, 260.0, 0.0);
    let (day_rgb, day_alpha) = shell.sample_scatter(cam_day, Vec3::NEG_Y, sun_dir);
    assert!(
        day_rgb.z > day_rgb.x * 2.0 && day_alpha > 0.15,
        "Daylight Earth limb from space must glow azure-blue, got rgb={day_rgb:?}, a={day_alpha}"
    );

    let cam_term = Vec3::new(2.0, 260.0, 68.0);
    let (term_rgb, _) = shell.sample_scatter(cam_term, Vec3::NEG_Y, sun_dir);
    assert!(
        term_rgb.x > term_rgb.z,
        "Terminator horizon from space must scatter warm orange-red sunset tones, got {term_rgb:?}"
    );

    let cam_night = Vec3::new(-68.0, 260.0, 0.0);
    let (night_rgb, _) = shell.sample_scatter(cam_night, Vec3::NEG_Y, sun_dir);
    assert!(
        night_rgb.length() < day_rgb.length() * 0.15,
        "Unlit night-side limb must fade into dark space, got {night_rgb:?}"
    );

    let solar_cube = CubicSolarBody::new(Vec3::Z, Vec3::Y, 0.04);
    let corner_ray = Vec3::new(0.036, 0.036, 1.0).normalize();
    assert!(
        solar_cube.strikes_cube(corner_ray),
        "Cubic Sun must include square corner diagonal rays beyond an inscribed circle"
    );

    // Square eclipse occluder directly in front of the cubic Sun cuts a crisp square umbra
    let square_moon_occ = [0.0, 0.0, 1.0, 0.025];
    assert!(
        solar_cube.eclipse_transmittance(Vec3::Z, square_moon_occ) < 0.03,
        "Foreground Moon/Planet transit must cast a dark umbra across the solar core"
    );
    let square_occ_corner = Vec3::new(0.021, 0.021, 1.0).normalize();
    assert!(
        solar_cube.eclipse_transmittance(square_occ_corner, square_moon_occ) < 0.05,
        "Cubic eclipsing Moon must occlude its square corner diagonal"
    );
    let unoccluded_rim = Vec3::new(0.033, 0.0, 1.0).normalize();
    assert!(
        solar_cube.eclipse_transmittance(unoccluded_rim, square_moon_occ) > 0.95,
        "Outer solar rim beyond the eclipsing square must remain bright"
    );

    // Real-time FirmamentPlanets directions match SolarEphemeris planet positions
    let eph = SolarEphemeris::new(SolarProportions::true_scale(64), 420.0);
    let atlas = world::CelestialAtlas::heliosphere(eph);
    let snaps = eph.snapshot_all(0.0);
    let jupiter = snaps.into_iter().find(|s| s.id() == BodyId::JUPITER).expect("Jupiter");
    let expected_jup_dir = jupiter.center().normalize().as_vec3();
    FirmamentPlanets::from_atlas(atlas, Vec3::ZERO).transfer(|planets, _occ| {
        let gpu_jup_dir = Vec3::new(planets[3][0], planets[3][1], planets[3][2]);
        assert!(
            gpu_jup_dir.dot(expected_jup_dir) > 0.9999 && planets[3][3] > 0.001,
            "Firmament uniform slot 3 (Jupiter) must match live SolarEphemeris direction"
        );
    });
}

#[test]
fn test_unpowered_jupiter_slingshot_curves_trajectory_and_boosts_heliocentric_speed() {
    let prop = SolarProportions::true_scale(64);
    let eph = SolarEphemeris::new(prop, 0.0);
    let snaps0 = eph.snapshot_all(0.0);
    assert_eq!(snaps0.len(), MAJOR_BODY_COUNT);
    let sun0 = snaps0[0];
    let jup0 = snaps0.into_iter().find(|s| s.id() == BodyId::JUPITER).expect("Jupiter");

    let jup_helio_vel = jup0.velocity() - snaps0[0].velocity();
    let prograde = jup_helio_vel.normalize();
    let radial_out = (jup0.center() - sun0.center()).normalize();

    let approach_speed = jup_helio_vel.length() * 0.85;
    let lead_time = 16.0_f64;
    let intercept_jup = jup0.center() + jup0.velocity() * lead_time;
    let aim_point = intercept_jup - prograde * (jup0.half_size() * 1.15);
    let approach_helio_vel = radial_out * approach_speed;
    let mut ship_vel = snaps0[0].velocity() + approach_helio_vel;
    let mut ship_pos = aim_point - ship_vel * lead_time;

    let initial_helio_speed = (ship_vel - snaps0[0].velocity()).length();
    let initial_dir = (ship_vel - snaps0[0].velocity()).normalize();

    let dt = 0.10_f64;
    let mut t = 0.0_f64;
    for _ in 0..450 {
        let a1 = eph.gravity_accel_at(ship_pos, t);
        let next_pos = ship_pos + ship_vel * dt + a1 * (0.5 * dt * dt);
        let a2 = eph.gravity_accel_at(next_pos, t + dt);
        ship_vel += (a1 + a2) * (0.5 * dt);
        ship_pos = next_pos;
        t += dt;
    }

    let snaps_end = eph.snapshot_all(t);
    let final_helio_vel = ship_vel - snaps_end[0].velocity();
    let final_helio_speed = final_helio_vel.length();
    let deflection_deg = initial_dir.dot(final_helio_vel.normalize()).clamp(-1.0, 1.0).acos().to_degrees();

    assert!(
        deflection_deg > 18.0,
        "Unpowered flyby must curve into a hyperbolic slingshot arc, got {deflection_deg:.2} deg"
    );
    assert!(
        final_helio_speed > initial_helio_speed + 12.0,
        "Trailing Jupiter flyby must transfer orbital momentum (slingshot boost): before={initial_helio_speed:.2}, after={final_helio_speed:.2}"
    );
}

#[test]
fn test_on_demand_mars_and_asteroid_belt_landing_persists_placed_blocks_across_planet_hops() {
    let prop = SolarProportions::compressed(32, 0.02);
    let mut world = CelestialSystem::barren_heliosphere(prop, 1.0);
    let atlas = world.celestial_atlas();
    let eph = atlas.ephemeris().expect("heliosphere ephemeris");
    let snaps = eph.snapshot_all(0.0);
    let mars = snaps.into_iter().find(|s| s.id() == BodyId::MARS).expect("Mars");
    let europa = snaps.into_iter().find(|s| s.id() == BodyId::EUROPA).expect("Europa");

    let mars_probe = mars.center().as_vec3() + Vec3::new(0.25, (mars.half_size() as f32) + 8.0, 0.25);
    let mars_ground = world.constrain_to_surface(mars_probe, 0.0);
    assert!((mars_ground.y - mars.center().y as f32) <= (mars.half_size() as f32) + 2.5);

    let eye = mars_ground + Vec3::new(0.0, 1.5, 1.0);
    let dir = (mars_ground - eye).normalize();
    assert!(world.place_molded_surface_block(
        eye,
        dir,
        mars_ground + Vec3::Z * 1.5,
        BlockMold::cube().in_medium(BlockMedium::QuarriedStone),
    ));
    let built_height = world.constrain_to_surface(mars_ground + Vec3::Y * 4.0, 0.0).y;
    assert!(built_height >= mars_ground.y + 0.45, "Placed block on Mars must raise local surface");

    let europa_probe = europa.center().as_vec3() + Vec3::new(0.25, (europa.half_size() as f32) + 8.0, 0.25);
    let europa_ground = world.constrain_to_surface(europa_probe, 0.0);
    assert!((europa_ground - europa.center().as_vec3()).length() < (europa.half_size() as f32) + 4.0);

    let restored_height = world.constrain_to_surface(mars_ground + Vec3::Y * 4.0, 0.0).y;
    assert!(
        (restored_height - built_height).abs() < 0.05,
        "Returning from Europa to Mars must replay Mars chronicle and preserve built blocks"
    );

    let sun = snaps[0];
    let belt = AsteroidBelt::new(prop, 0.0);
    let sample_r = prop.astronomical_unit() * 2.65;
    let mut found_rock = None;
    for step in 0..64 {
        let ang = f64::from(step) * 0.098;
        let probe = sun.center() + DVec3::new(sample_r * ang.cos(), 0.0, sample_r * ang.sin());
        if let Some(r) = belt.nearest_rock(probe, sun.center(), sun.gm(), 600.0) {
            found_rock = Some(r);
            break;
        }
    }
    let rock = found_rock.expect("Procedural Asteroid Belt must yield deterministic belt rocks");
    let rock_ground = world.constrain_to_surface(
        rock.center().as_vec3() + Vec3::Y * (rock.half_size() + 6.0),
        0.0,
    );
    assert!(
        (rock_ground - rock.center().as_vec3()).abs().max_element() <= rock.half_size() + 3.0,
        "Belt asteroid must materialize on-demand as a landable CubePlanet"
    );
}
