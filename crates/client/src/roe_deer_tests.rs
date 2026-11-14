use glam::Vec3;
use world::{BiomedCubeWorld, ClimateZone, GravityField, MeshTarget, PlanetaryWorld, Quad};
use crate::allocation_audit::count_allocations;
use crate::cardiac::{AcousticWave, RoeBark, RoeHoofbeats};
use crate::roe_deer::{RoeActivity, RoeHerd, TestRoeDeer as RoeDeer, WoodlandFauna};

struct FacetBatch {
    quads: Vec<Quad>,
}

impl FacetBatch {
    fn empty() -> Self {
        Self { quads: Vec::new() }
    }
}

impl MeshTarget for FacetBatch {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

#[test]
fn test_roe_herd_anchors_on_temperate_zone_with_sculpted_mesh_and_zero_allocations() {
    let world = BiomedCubeWorld::beveled(32, 1.0);
    let desert_observer = world.constrain_to_surface(Vec3::new(17.0, 0.0, 0.0), 1.0);
    let mut herd = RoeHerd::seeded();

    herd.step_spatial(1.0 / 60.0, desert_observer, Vec3::NEG_Z, false, &world);

    let grazing_count = herd.deer().iter().filter(|d| d.grazing()).count();
    let alert_count = herd.deer().iter().filter(|d| d.alert()).count();
    let trotting_count = herd.deer().iter().filter(|d| d.trotting()).count();
    let antlered_count = herd.deer().iter().filter(|d| d.antlered()).count();

    assert!(grazing_count >= 1, "Herd must include roe deer peacefully grazing");
    assert!(alert_count >= 1, "Herd must include a roe deer keeping alert watch");
    assert!(trotting_count >= 1, "Herd must include a roe deer trotting across the glade");
    assert_eq!(antlered_count, 2, "Herd must pair roebucks with antlers alongside does");

    for member in herd.deer() {
        let pos = member.position();
        assert_eq!(
            ClimateZone::dominant(pos),
            ClimateZone::Temperate,
            "Roe deer must exclusively inhabit ClimateZone::Temperate ({pos:?})"
        );
        assert!(world.surface_up_at(pos).y > 0.85, "Roe deer must stay on Temperate meadow ground");
    }

    let mut mesh = FacetBatch::empty();
    herd.emit(&mut mesh);
    assert!(
        mesh.quads.len() >= 4 * 120,
        "Each roe deer must emit sculpted torso, white rump mirror, neck, muzzle, ears, legs, and buck antlers"
    );
    for quad in &mesh.quads {
        quad.transfer(|v0, v1, v2, v3, normal, _| {
            assert!(v0.is_finite() && v1.is_finite() && v2.is_finite() && v3.is_finite());
            assert!((normal.length() - 1.0).abs() < 1e-4, "Roe deer quad normal must be normalized");
        });
    }

    let temperate_listener = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let mut fauna = WoodlandFauna::seeded();
    fauna.step_spatial(1.0 / 60.0, temperate_listener, Vec3::NEG_Z, false, &world);
    let allocs = count_allocations(|| {
        for _ in 0..90 {
            fauna.step_spatial(1.0 / 60.0, temperate_listener, Vec3::NEG_Z, false, &world);
        }
    });
    assert_eq!(allocs, 0, "WoodlandFauna step must allocate zero heap memory");
}

#[test]
fn test_roe_deer_rewards_crouching_stealth_and_bounds_away_with_hoofbeats_when_startled() {
    let world = BiomedCubeWorld::beveled(32, 1.0);
    let glade = world.constrain_to_surface(Vec3::new(2.0, 20.0, -2.0), 0.0);

    let mut buck = RoeDeer::new(
        RoeActivity::Grazing {
            position: glade,
            yaw: 0.0,
            chew_phase: 0.0,
            remaining: 4.0,
        },
        Vec3::Y,
        77,
        true,
    );
    let doe = RoeDeer::new(
        RoeActivity::AlertStance {
            position: glade,
            yaw: 0.0,
            scan_phase: 0.0,
            remaining: 4.0,
        },
        Vec3::Y,
        88,
        false,
    );
    let (mut buck_mesh, mut doe_mesh) = (FacetBatch::empty(), FacetBatch::empty());
    buck.emit(&mut buck_mesh);
    doe.emit(&mut doe_mesh);
    assert!(
        buck_mesh.quads.len() > doe_mesh.quads.len(),
        "Antlered roebuck must emit additional forked antler geometry compared to a doe"
    );

    // Crouching human at 5.0m stays below crouch flee threshold (3.4m), keeping the deer from fleeing
    let sneaking_human = glade + Vec3::new(5.0, 0.0, 0.0);
    assert!(!buck.step(1.0 / 60.0, sneaking_human, true, &world));
    assert!(
        buck.alert() && !buck.bounding(),
        "Crouching observer at 5m must only make the roe deer watch attentively without fleeing"
    );

    // Standing up at 5.0m breaches standing flee threshold (7.5m), triggering hoofbeats and bounding leaps
    let bolted = buck.step(1.0 / 60.0, sneaking_human, false, &world);
    assert!(bolted && buck.bounding(), "Standing observer at 5m must startle the roe deer into bounding flight");

    let initial_dist = (glade - sneaking_human).length();
    for _ in 0..40 {
        buck.step(1.0 / 30.0, sneaking_human, false, &world);
    }
    let escaped_pos = buck.position();
    assert!(
        (escaped_pos - sneaking_human).length() > initial_dist + 4.5,
        "Bounding roe deer must leap away to a safer Temperate clearing"
    );
    assert_eq!(ClimateZone::dominant(escaped_pos), ClimateZone::Temperate);
}

#[test]
fn test_roe_hoofbeats_synthesizes_receding_three_bound_gallop_and_grass_rustle() {
    let hoofbeats = RoeHoofbeats::new(24000, 1.0, 0.0, 1.0);
    assert_eq!(hoofbeats.sample_rate(), 24000);
    let bound1_peak = (0..24)
        .map(|k| hoofbeats.amplitude_at(0.02 + (k as f32) * 0.002).abs())
        .fold(0.0_f32, f32::max);
    let bound2_peak = (0..24)
        .map(|k| hoofbeats.amplitude_at(0.23 + (k as f32) * 0.002).abs())
        .fold(0.0_f32, f32::max);
    let bound3_peak = (0..24)
        .map(|k| hoofbeats.amplitude_at(0.44 + (k as f32) * 0.002).abs())
        .fold(0.0_f32, f32::max);
    assert!(bound1_peak > bound2_peak && bound2_peak > bound3_peak && bound3_peak > 0.12);
    assert_eq!(hoofbeats.amplitude_at(0.19), 0.0, "Airborne phase between bounds must be silent");
    assert_eq!(hoofbeats.amplitude_at(0.40), 0.0, "Airborne phase between bounds must be silent");

    let listener = Vec3::new(0.0, 17.0, 0.0);
    let near_right = RoeHoofbeats::spatial(listener + Vec3::new(3.5, 0.0, -1.0), listener, Vec3::NEG_Z, Vec3::Y, 11);
    let far_left = RoeHoofbeats::spatial(listener + Vec3::new(-24.0, 0.0, -3.0), listener, Vec3::NEG_Z, Vec3::Y, 22);
    let (nr_l, nr_r) = near_right.channel_gains();
    let (fl_l, fl_r) = far_left.channel_gains();
    assert!(nr_r > nr_l * 1.5, "Hoofbeats to the right must pan right ({nr_l} vs {nr_r})");
    assert!(fl_l > fl_r * 1.5, "Hoofbeats to the left must pan left ({fl_l} vs {fl_r})");
    let near_peak = (0..20).map(|k| near_right.amplitude_at(0.02 + (k as f32) * 0.002).abs()).fold(0.0_f32, f32::max);
    let far_peak = (0..20).map(|k| far_left.amplitude_at(0.02 + (k as f32) * 0.002).abs()).fold(0.0_f32, f32::max);
    assert!(near_peak > far_peak * 3.0);

    let stereo = near_right.stereo_riff_wave();
    assert_eq!(&stereo[0..4], b"RIFF");
    assert_eq!(u16::from_le_bytes([stereo[22], stereo[23]]), 2);
}

#[test]
fn test_roe_bark_and_herd_alternate_between_bounding_hoofbeats_and_alarm_bark() {
    let bark = RoeBark::new(24000, 1.0, 0.0, 1.0);
    assert_eq!(bark.sample_rate(), 24000);
    let primary_peak = (0..12)
        .map(|k| bark.amplitude_at(0.05 + (k as f32) * 0.0006).abs())
        .fold(0.0_f32, f32::max);
    let echo_peak = (0..12)
        .map(|k| bark.amplitude_at(0.29 + (k as f32) * 0.0006).abs())
        .fold(0.0_f32, f32::max);
    assert!(primary_peak > 0.20, "Primary alarm bark must have strong amplitude ({primary_peak})");
    assert_eq!(bark.amplitude_at(0.22), 0.0, "Inter-bark pause must be silent");
    assert!(echo_peak > 0.12, "Secondary echo bark must be audible ({echo_peak})");

    let spatial_bark = RoeBark::spatial(Vec3::new(3.0, 17.0, -1.0), Vec3::new(0.0, 17.0, 0.0), Vec3::NEG_Z, Vec3::Y, 9);
    let (bark_l, bark_r) = spatial_bark.channel_gains();
    assert!(bark_r > bark_l * 1.5, "Bark to the right must pan right ({bark_l} vs {bark_r})");
    let bark_wav = spatial_bark.stereo_riff_wave();
    assert_eq!(&bark_wav[0..4], b"RIFF");

    let world = BiomedCubeWorld::beveled(32, 1.0);
    let observer = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let mut herd = RoeHerd::seeded();
    herd.step_spatial(1.0 / 60.0, observer, Vec3::NEG_Z, false, &world);
    assert!(!herd.bark_queued(), "First startled leap starts with bounding hoofbeats");

    let first_deer_pos = herd.deer()[0].position();
    assert!(herd.step_spatial(1.0 / 60.0, first_deer_pos + Vec3::new(2.0, 0.0, 0.0), Vec3::NEG_Z, false, &world));
    assert!(herd.bark_queued(), "After bounding hoofbeats, next startled event queues laryngeal alarm bark");

    herd.step_spatial(3.0, observer + Vec3::new(0.0, 40.0, 0.0), Vec3::NEG_Z, false, &world);
    let second_deer_pos = herd.deer()[1].position();
    assert!(herd.step_spatial(1.0 / 60.0, second_deer_pos + Vec3::new(2.0, 0.0, 0.0), Vec3::NEG_Z, false, &world));
    assert!(!herd.bark_queued(), "After laryngeal alarm bark, herd cycles back to bounding hoofbeats");
}
