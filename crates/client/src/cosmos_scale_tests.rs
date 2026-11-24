use world::SolarProportions;
use crate::cosmos_scale::{ArgumentCosmos, CosmosScale, FileCosmos, MeasuredCosmos};
use crate::server_scale::ScaledCube;
use crate::toml_document::TomlDocument;

#[test]
fn test_cosmos_decorator_chain_prioritizes_cli_over_client_and_server_toml() {
    let base = Box::new(MeasuredCosmos::new(Box::new(ScaledCube::new(2))));
    let server_doc = TomlDocument::new(
        "[cosmos]\nbody_scale = 0.5\nsun_scale = 0.8\ndistance_scale = 0.1\nasteroid_density = 0.4\n".to_string(),
    );
    let client_doc = TomlDocument::new("[cosmos]\nasteroid_density = 1.75\n".to_string());
    let s_layer = Box::new(FileCosmos::from_document(&server_doc, "server.toml", base));
    let c_layer = Box::new(FileCosmos::from_document(&client_doc, "client.toml", s_layer));
    let args = vec![
        "client".to_string(),
        "--distance-scale".to_string(),
        "0.25".to_string(),
    ];
    let cli_layer = ArgumentCosmos::from_arguments(&args, c_layer);
    let p = cli_layer.proportions();
    assert_eq!(p.earth_blocks(), 256);
    assert!((p.body_scale() - 0.5).abs() < 1e-5);
    assert!((p.sun_scale() - 0.8).abs() < 1e-5);
    assert!((p.distance_scale() - 0.25).abs() < 1e-5);
    assert!((p.belt_density() - 1.75).abs() < 1e-5);
}

#[test]
fn test_sanctuary_handshake_transmits_solar_proportions_to_guest() {
    let custom = SolarProportions::true_scale(32)
        .with_body_scale(0.6)
        .with_sun_scale(1.4)
        .with_distance_scale(0.08)
        .with_belt_density(1.5);
    let world = world::CelestialSystem::barren_authoritative(32, 1.0);
    let mut host = sanctuary::SanctuaryHost::bind_cosmos("127.0.0.1:0", world, custom)
        .expect("bind cosmos host");
    let addr = host.local_addr().expect("host addr");
    let mut guest = sanctuary::RemoteLink::bind(sanctuary::JoinTarget::Direct(addr))
        .expect("bind guest");
    let mut received = None;
    for _ in 0..20 {
        let _ = host.tick(0.02);
        if let Some(prop) = guest.step_cosmos_handshake(0.02) {
            received = Some(prop);
            break;
        }
    }
    assert_eq!(received, Some(custom), "Guest must receive host SolarProportions over LWEL handshake");
}

#[test]
fn test_solar_system_scale_factor_governs_distances_planets_sun_moons_and_orrery() {
    use glam::Vec3;
    use world::{BodyId, CelestialAtlas, SolarEphemeris};
    use crate::menu::OrreryFrame;
    use crate::rayleigh_shell::{AtlasSkyProjection, SkyProjection};

    let base_doc = TomlDocument::new("[cosmos]\nscale = 0.25\n".to_string());
    let scaled_cosmos = FileCosmos::from_document(
        &base_doc,
        "server.toml",
        Box::new(MeasuredCosmos::new(Box::new(ScaledCube::new(1)))),
    );
    let p_quarter = scaled_cosmos.proportions();
    assert!((p_quarter.distance_scale() - 0.25).abs() < 1e-5);
    assert!((p_quarter.body_scale() - 0.25).abs() < 1e-5);
    assert!((p_quarter.sun_scale() - 0.25).abs() < 1e-5);

    let p_unit = SolarProportions::true_scale(128);
    let snaps_unit = SolarEphemeris::new(p_unit, 0.0).snapshot_all(0.0);
    let snaps_quarter = SolarEphemeris::new(p_quarter, 0.0).snapshot_all(0.0);

    let find = |snaps: &[world::BodySnapshot], id: BodyId| snaps.iter().copied().find(|s| s.id() == id).unwrap();
    let (sun1, sun_q) = (find(&snaps_unit, BodyId::SOL), find(&snaps_quarter, BodyId::SOL));
    let (mars1, mars_q) = (find(&snaps_unit, BodyId::MARS), find(&snaps_quarter, BodyId::MARS));
    let (jup1, jup_q) = (find(&snaps_unit, BodyId::JUPITER), find(&snaps_quarter, BodyId::JUPITER));
    let (luna1, luna_q) = (find(&snaps_unit, BodyId::LUNA), find(&snaps_quarter, BodyId::LUNA));
    let (phobos1, phobos_q) = (find(&snaps_unit, BodyId::PHOBOS), find(&snaps_quarter, BodyId::PHOBOS));

    assert!(((sun_q.half_size() / sun1.half_size()) - 0.25).abs() < 0.02, "Sun half_size must scale by 0.25");
    assert!(((mars_q.orbit_radius() / mars1.orbit_radius()) - 0.25).abs() < 0.01, "Mars orbit must scale by 0.25");
    assert!(((mars_q.half_size() / mars1.half_size()) - 0.25).abs() < 0.06, "Mars size must scale by 0.25");
    assert!(((jup_q.half_size() / jup1.half_size()) - 0.25).abs() < 0.01, "Jupiter size must scale by 0.25");
    assert!(((luna_q.orbit_radius() / luna1.orbit_radius()) - 0.25).abs() < 0.02, "Luna orbit must scale with distance_scale");
    assert!(((luna_q.half_size() / luna1.half_size()) - 0.25).abs() < 0.08, "Luna size must scale with body_scale");
    assert!(phobos_q.half_size() < phobos1.half_size(), "Small moons like Phobos must scale below 16 blocks");

    // Independent sun_scale and body_scale adjust sky shader and Nose-Cam Orrery proportions
    let p_big_sun = p_unit.with_sun_scale(2.5).with_body_scale(2.0).with_distance_scale(0.2);
    let atlas_unit = CelestialAtlas::heliosphere(SolarEphemeris::new(p_unit, 0.0));
    let atlas_custom = CelestialAtlas::heliosphere(SolarEphemeris::new(p_big_sun, 0.0));
    let sun_ang_1 = AtlasSkyProjection::solar(atlas_unit).project_from(Vec3::ZERO).transfer(|c, _| c[7][3]);
    let sun_ang_2 = AtlasSkyProjection::solar(atlas_custom).project_from(Vec3::ZERO).transfer(|c, _| c[7][3]);
    assert!(sun_ang_2 > sun_ang_1 * 10.0, "Sky Sun angular size must scale with sun_scale / distance_scale");

    let (ve1, vm1) = OrreryFrame::from_atlas(atlas_unit).visual_radii();
    let (ve2, vm2) = OrreryFrame::from_atlas(atlas_custom).visual_radii();
    assert!(ve2 > ve1 * 4.5 && vm2 > vm1 * 9.0, "Nose-Cam Holo Earth & Moon cubes must scale with distance_scale and body_scale");
}

#[test]
fn test_sun_remains_fixed_at_solar_system_center_when_rocket_steers() {
    use glam::{Mat4, Vec3};
    use world::{BodyId, CelestialSystem, MeshTarget, Quad, SightCone};
    use crate::lens::{PerspectiveLens, ProjectionLens};

    let world = CelestialSystem::barren_heliosphere(SolarProportions::true_scale(128), 1.0);
    let eph = world.celestial_atlas().ephemeris().expect("heliosphere ephemeris");
    let sun_center = eph.snapshot_all(0.0)[0].center().as_vec3();
    let eye = Vec3::new(0.25, 66.0, 0.25);
    let to_sun = (sun_center - eye).normalize();
    assert!(to_sun.y > 0.20, "Temperate spawn face must receive daylight from tilted ecliptic Sun");
    assert_eq!(world.body_center(BodyId::SOL), Some(sun_center));

    let proj = PerspectiveLens::human_eye().projection_matrix(16.0 / 9.0);
    struct SolarViewProbe {
        vp: Mat4,
        solar_quads_in_view: usize,
    }
    impl MeshTarget for SolarViewProbe {
        fn push_quad(&mut self, q: Quad) {
            if !q.radiate(|g| g >= 0.99) {
                return;
            }
            q.transfer(|v0, v1, v2, v3, _, _| {
                let c = (v0 + v1 + v2 + v3) * 0.25;
                let clip = self.vp * c.extend(1.0);
                if clip.w > 0.03 {
                    let ndc_xy = clip.truncate() / clip.w;
                    let log_ndc_z = (1.0 + (clip.w - 0.03)).log2() * 0.060206 - 1.0;
                    if ndc_xy.x.abs() < 0.25 && ndc_xy.y.abs() < 0.25 && (-1.0..1.0).contains(&log_ndc_z) {
                        self.solar_quads_in_view += 1;
                    }
                }
            });
        }
    }

    // Aimed directly at SOL -> 3D cubic Sun is dead-center in viewport
    let mut aimed_at_sun = SolarViewProbe {
        vp: proj * Mat4::look_to_rh(eye, to_sun, Vec3::Y),
        solar_quads_in_view: 0,
    };
    world.emit_cosmic_field(SightCone::from_view(eye, to_sun), &mut aimed_at_sun);
    assert!(aimed_at_sun.solar_quads_in_view >= 2, "Looking toward solar system center must show 3D cubic Sun");

    // Rocket steers 90 degrees away -> Sun stays fixed in world space and leaves viewport
    let steered_away = to_sun.cross(Vec3::Y).normalize();
    let mut turned_probe = SolarViewProbe {
        vp: proj * Mat4::look_to_rh(eye, steered_away, Vec3::Y),
        solar_quads_in_view: 0,
    };
    world.emit_cosmic_field(SightCone::from_view(eye, steered_away), &mut turned_probe);
    assert_eq!(turned_probe.solar_quads_in_view, 0, "Sun must remain fixed in solar system center and never follow rocket steering");
}

#[test]
fn test_dual_slot_vicinity_pool_planetary_atmospheres_and_optical_telescope_zoom() {
    use glam::Vec3;
    use world::{BodyId, CelestialSystem};
    use crate::lens::{PerspectiveLens, ProjectionLens};
    use crate::rayleigh_shell::{AtlasSkyProjection, SkyProjection};

    // 1. Mars + Phobos awaken simultaneously into GPU banks `6..11` and `18..23` without evicting each other
    let prop = SolarProportions::compressed(32, 0.02);
    let world = CelestialSystem::barren_heliosphere(prop, 1.0);
    let atlas = world.celestial_atlas();
    let mars = world.body_center(BodyId::MARS).expect("Mars");
    let phobos = world.body_center(BodyId::PHOBOS).expect("Phobos");
    let mid = (mars + phobos) * 0.5;
    let mask = world.visible_slice_mask(mid);
    assert_ne!((mask >> 6) & 0x3F, 0, "Primary nearby body (Mars/Phobos) must occupy slot 0 (slices 6..11)");
    assert_ne!((mask >> 18) & 0x3F, 0, "Secondary nearby body must simultaneously occupy slot 1 (slices 18..23)");

    // 2. Multi-world AtmosphericShell: Earth & Mars carry distinct atmospheres; Luna is airless vacuum
    let luna = world.body_center(BodyId::LUNA).expect("Luna");
    let (_, earth_sky, earth_night) = AtlasSkyProjection::solar(atlas)
        .project_from(Vec3::new(0.0, 20.0, 0.0))
        .sky_illumination(Vec3::Y);
    let (_, mars_sky, mars_night) = AtlasSkyProjection::solar(atlas)
        .project_from(mars + Vec3::Y * 9.5)
        .sky_illumination(Vec3::Y);
    let (_, _, luna_night) = AtlasSkyProjection::solar(atlas)
        .project_from(luna + Vec3::Y * 10.0)
        .sky_illumination(Vec3::Y);
    assert!(earth_sky.z > earth_sky.x && earth_night == 0.0, "Earth shell must scatter azure blue");
    assert!(mars_sky.x > mars_sky.z && mars_night == 0.0, "Mars shell must scatter salmon-butterscotch dust");
    assert!(luna_night > 0.99, "Airless Luna must preserve dark vacuum cosmos in broad daylight");

    // 3. Nose-Cam Optical Telescope Zoom (`1× → 6× → 24×`) narrows FOV while preserving 100M-meter cosmic reach
    let wide = PerspectiveLens::human_eye();
    let tele = wide.magnified(24.0);
    assert!(tele.projection_matrix(1.0).y_axis.y > wide.projection_matrix(1.0).y_axis.y * 18.0);
}
