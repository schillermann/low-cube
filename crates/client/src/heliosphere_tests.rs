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
            gpu_jup_dir.dot(expected_jup_dir) > 0.9999 && planets[3][3] > 0.00005 && planets[7][3] > 0.004,
            "Firmament slots must match live SolarEphemeris Jupiter direction and solar angular half-size"
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

#[test]
fn test_nose_cam_sees_3d_moon_across_entire_flight_and_holo_orrery_matches_lunar_surface() {
    use glam::Mat3;
    use world::{MeshTarget, Quad, SightCone};
    use crate::drive::{ARC_NODES, NavigationCourse};
    use crate::lens::{PerspectiveLens, ProjectionLens};
    use crate::menu::OrreryFrame;

    let prop = SolarProportions::true_scale(256);
    let mut world = CelestialSystem::barren_heliosphere(prop, 1.0);
    let atlas0 = world.celestial_atlas();
    let moon_center = world.body_center(BodyId::LUNA).expect("Luna center");
    let moon_half = atlas0.transfer(|_, _, mh| mh);
    assert!(moon_center.length() > 7_000.0, "256-block true-scale Luna orbits beyond 4096m far plane");

    // 1. OrreryFrame must be right-handed SO(3) (det = +1) and strictly exterior to vis_moon outside moon_half
    let frame = OrreryFrame::from_atlas(atlas0);
    let det = Mat3::from_cols(
        frame.rotate_dir(Vec3::X),
        frame.rotate_dir(Vec3::Y),
        frame.rotate_dir(Vec3::Z),
    )
    .determinant();
    assert!((det - 1.0).abs() < 1e-4, "OrreryFrame rotation must be right-handed SO(3), got det={det}");
    let (_, vis_moon) = frame.visual_radii();
    let moon_holo = frame.map_point(moon_center);
    let axis_to_earth = (Vec3::ZERO - moon_center).normalize();
    for stand_off in [800.0_f32, 414.0, 200.0, 90.0] {
        let ship_holo = frame.map_point(moon_center + axis_to_earth * stand_off);
        let holo_cheb = (ship_holo - moon_holo).abs().max_element();
        assert!(
            holo_cheb > vis_moon + 0.002,
            "Rocket {stand_off}m outside Moon must remain outside Holo Moon-Cube ({holo_cheb} <= {vis_moon})"
        );
    }
    let lateral_miss = frame.map_point(moon_center + Vec3::Y * 220.0);
    assert!((lateral_miss - moon_holo).abs().max_element() > vis_moon + 0.004);
    let on_crust = frame.map_point(moon_center + Vec3::Z * moon_half);
    assert!(((on_crust - moon_holo).length() - vis_moon).abs() < 1e-4);
    let approach_frame = OrreryFrame::for_ship(atlas0, moon_center + axis_to_earth * (moon_half * 6.0));
    assert!(approach_frame.approach_zoom() && !frame.approach_zoom());

    // 2. Nose-Cam projection sees 3D Moon-Cube from launchpad (7,705m > 4,096m far clip) through mid-course
    let lens = PerspectiveLens::human_eye().projection_matrix(16.0 / 9.0);
    struct BoresightMoonProbe {
        vp: glam::Mat4,
        hits: usize,
    }
    impl MeshTarget for BoresightMoonProbe {
        fn push_quad(&mut self, q: Quad) {
            q.transfer(|v0, v1, v2, v3, _, _| {
                let c = (v0 + v1 + v2 + v3) * 0.25;
                let clip = self.vp * c.extend(1.0);
                if clip.w > 0.1 {
                    let ndc = clip.truncate() / clip.w;
                    if ndc.x.abs() < 0.08 && ndc.y.abs() < 0.08 && (0.0..1.0).contains(&ndc.z) {
                        self.hits += 1;
                    }
                }
            });
        }
    }
    for range_m in [7_500.0_f32, 2_400.0, 520.0] {
        let eye = moon_center + axis_to_earth * range_m;
        let fwd = (moon_center - eye).normalize();
        let vp = lens * glam::Mat4::look_to_rh(eye, fwd, Vec3::Y);
        let mut probe = BoresightMoonProbe { vp, hits: 0 };
        world.emit_cosmic_field(SightCone::from_view(eye, fwd), &mut probe);
        assert!(probe.hits >= 3, "Nose-Cam aimed at Moon from {range_m}m must see 3D Moon-Cube quads, got {}", probe.hits);
    }

    // 3. Entering <= 384m awakens full 6-face LunarWorld into slices 6..11 without eviction at 105m radar ceiling
    let approach_pos = moon_center + axis_to_earth * (moon_half + 240.0);
    let mask = world.visible_slice_mask(approach_pos);
    assert_ne!((mask >> 6) & 0x3F, 0, "Approaching within 240m must enable lunar slice mask bits 6..11");
    assert!(world.pending_slices());
    let mut drained_slices = Vec::new();
    struct QuadCounter(usize);
    impl MeshTarget for QuadCounter {
        fn push_quad(&mut self, _: Quad) {
            self.0 += 1;
        }
    }
    let mut lunar_quads = QuadCounter(0);
    while let Some(slice_idx) = world.drain_modified_slice(&mut lunar_quads) {
        drained_slices.push(slice_idx);
    }
    assert_eq!(drained_slices, vec![6, 7, 8, 9, 10, 11]);
    assert!(lunar_quads.0 > 500, "Awakened Moon slices 6..11 must upload cratered voxel quads");
    let mask_radar = world.visible_slice_mask(moon_center + axis_to_earth * (moon_half + 104.0));
    assert_ne!((mask_radar >> 6) & 0x3F, 0, "Radar-ceiling altitude (104m) must retain awakened Moon");

    // 4. Nose-Cam target lock (`mark_along`) tracks moving Luna across SolarEphemeris ticks
    let mut world_moving = CelestialSystem::barren_heliosphere(SolarProportions::true_scale(64), 1.0);
    let eye0 = Vec3::new(0.0, 40.0, 0.0);
    let moon0 = world_moving.body_center(BodyId::LUNA).expect("Luna");
    let dir0 = (moon0 - eye0).normalize();
    let mut course = NavigationCourse::Uncharted;
    assert!(course.mark_along(((eye0, dir0), (eye0, dir0)), (eye0, Vec3::ZERO), world_moving.as_ref()));
    let _ = world_moving.step_surface(45.0);
    let moon1 = world_moving.body_center(BodyId::LUNA).expect("Moved Luna");
    assert!((moon1 - moon0).length() > 40.0);
    course.sync_ephemeris(world_moving.celestial_atlas());
    if let NavigationCourse::Charted(arc) = course {
        arc.feed_nodes(|nodes, _| {
            let surf_cheb = (nodes[ARC_NODES - 1] - moon1).abs().max_element();
            assert!(surf_cheb <= 12.0, "Synced target lock must ride with moving Luna, got cheb={surf_cheb}");
        });
    } else {
        panic!("Expected charted lunar course");
    }
}
