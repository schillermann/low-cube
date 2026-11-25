use world::SolarProportions;
use crate::cosmos_scale::{ArgumentCosmos, CosmosScale, EpsteinReachCosmos, FileCosmos, MeasuredCosmos};
use crate::server_scale::ScaledCube;
use crate::toml_document::TomlDocument;

#[test]
fn test_cosmos_decorator_chain_couples_to_world_size_and_prioritizes_cli_belt_density() {
    let base = Box::new(MeasuredCosmos::new(Box::new(ScaledCube::new(8))));
    let server_doc = TomlDocument::new("[cosmos]\nasteroid_density = 0.4\n".to_string());
    let client_doc = TomlDocument::new("[cosmos]\nasteroid_density = 1.75\n".to_string());
    let s_layer = Box::new(FileCosmos::from_document(&server_doc, "server.toml", base));
    let c_layer = Box::new(FileCosmos::from_document(&client_doc, "client.toml", s_layer));
    let args = vec![
        "client".to_string(),
        "--asteroid-density".to_string(),
        "2.25".to_string(),
    ];
    let cli_layer = EpsteinReachCosmos::new(ArgumentCosmos::from_arguments(&args, c_layer));
    let p = cli_layer.proportions();
    assert_eq!(p.earth_blocks(), 1024);
    assert!((p.distance_scale() - 1.0).abs() < 1e-5);
    assert!((p.belt_density() - 2.25).abs() < 1e-5);
    assert!(p.astronomical_unit() <= SolarProportions::EPSTEIN_HOUR_AU_CAP);
}

#[test]
fn test_sanctuary_handshake_transmits_solar_proportions_to_guest() {
    let custom = SolarProportions::compressed(32, 0.08).with_belt_density(1.5);
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
fn test_world_size_factor_scales_planets_sun_moons_and_caps_epstein_outer_belt_under_one_hour() {
    use world::{BodyId, SolarEphemeris};

    let p_1 = MeasuredCosmos::new(Box::new(ScaledCube::new(1))).proportions();
    let p_8 = MeasuredCosmos::new(Box::new(ScaledCube::new(8))).proportions();
    let p_32 = EpsteinReachCosmos::new(MeasuredCosmos::new(Box::new(ScaledCube::new(32)))).proportions();

    assert_eq!(p_1.earth_blocks(), 128);
    assert_eq!(p_8.earth_blocks(), 1024);
    assert_eq!(p_32.earth_blocks(), 4096);

    let snaps_1 = SolarEphemeris::new(p_1, 0.0).snapshot_all(0.0);
    let snaps_8 = SolarEphemeris::new(p_8, 0.0).snapshot_all(0.0);
    let snaps_32 = SolarEphemeris::new(p_32, 0.0).snapshot_all(0.0);
    let find = |snaps: &[world::BodySnapshot], id: BodyId| snaps.iter().copied().find(|s| s.id() == id).unwrap();

    let (sun1, sun8) = (find(&snaps_1, BodyId::SOL), find(&snaps_8, BodyId::SOL));
    let (mars1, mars8, mars32) = (find(&snaps_1, BodyId::MARS), find(&snaps_8, BodyId::MARS), find(&snaps_32, BodyId::MARS));
    let (luna1, luna8) = (find(&snaps_1, BodyId::LUNA), find(&snaps_8, BodyId::LUNA));

    assert!(((sun8.half_size() / sun1.half_size()) - 8.0).abs() < 0.02, "Sun must scale 8x with size = 8");
    assert!(((mars8.orbit_radius() / mars1.orbit_radius()) - 8.0).abs() < 0.01, "Mars orbit must scale 8x with size = 8");
    assert!(((mars8.half_size() / mars1.half_size()) - 8.0).abs() < 0.05, "Mars diameter must scale 8x with size = 8");
    assert!(
        (luna1.half_size() - 18.0).abs() < 1.0 && (luna8.half_size() - 140.0).abs() < 1.0,
        "Luna quantized voxel half_size must scale from 18m (36 blocks) to 140m (280 blocks)"
    );
    assert!(mars32.half_size() > 1000.0, "Mars on 4096^3 server must exceed legacy 512-block cap");

    // 3.20 AU outer belt worst-case conjunction (4.20 AU) with 8 G Flip & Burn (`t = 2 * sqrt(d / a)`):
    let worst_case_belt_m = p_32.astronomical_unit() * 4.20;
    let epstein_accel = 8.0 * 9.81;
    let flip_burn_seconds = 2.0 * (worst_case_belt_m / epstein_accel).sqrt();
    assert!(
        flip_burn_seconds < 3600.0,
        "Outer Asteroid Belt must stay reachable in < 1 hour even on 4096^3 server, got {flip_burn_seconds:.1}s"
    );
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

    let mut aimed_at_sun = SolarViewProbe {
        vp: proj * Mat4::look_to_rh(eye, to_sun, Vec3::Y),
        solar_quads_in_view: 0,
    };
    world.emit_cosmic_field(SightCone::from_view(eye, to_sun), &mut aimed_at_sun);
    assert!(aimed_at_sun.solar_quads_in_view >= 2, "Looking toward solar system center must show 3D cubic Sun");

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

    let prop = SolarProportions::compressed(32, 0.02);
    let world = CelestialSystem::barren_heliosphere(prop, 1.0);
    let atlas = world.celestial_atlas();
    let mars = world.body_center(BodyId::MARS).expect("Mars");
    let phobos = world.body_center(BodyId::PHOBOS).expect("Phobos");
    let mid = (mars + phobos) * 0.5;
    let mask = world.visible_slice_mask(mid);
    assert_ne!((mask >> 6) & 0x3F, 0, "Primary nearby body (Mars/Phobos) must occupy slot 0 (slices 6..11)");
    assert_ne!((mask >> 18) & 0x3F, 0, "Secondary nearby body must simultaneously occupy slot 1 (slices 18..23)");

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

    let wide = PerspectiveLens::human_eye();
    let tele = wide.magnified(24.0);
    assert!(tele.projection_matrix(1.0).y_axis.y > wide.projection_matrix(1.0).y_axis.y * 18.0);
}
