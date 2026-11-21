use glam::Vec3;
use world::{BiomedCubeWorld, ClimateZone, GravityField, MeshTarget, PlanetaryWorld, Quad};
use acoustics::{AcousticWave, BoarSnort};
use crate::{BoarActivity, BoarSounder, SightCone, TestWildBoar as WildBoar, WoodlandFauna};

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
fn test_boar_sounder_anchors_on_temperate_oaks_with_tusked_mesh_and_zero_allocations() {
    let world = BiomedCubeWorld::beveled(32, 1.0);
    let arid_observer = world.constrain_to_surface(Vec3::new(17.0, 0.0, 0.0), 1.0);
    let mut sounder = BoarSounder::seeded();

    assert!(sounder.step_spatial(1.0 / 60.0, arid_observer, Vec3::NEG_Z, false, &world).is_none());
    for boar in sounder.boars() {
        let pos = boar.position();
        assert_eq!(ClimateZone::dominant(pos), ClimateZone::Temperate);
        assert!(world.surface_up_at(pos).y > 0.85);
        assert!(boar.vitality_at(8));
        assert!(boar.rooting());
    }

    let mut mesh = FacetBatch::empty();
    sounder.emit(&mut mesh);
    assert!(
        mesh.quads.len() >= 3 * 120,
        "Each wild boar must emit humped torso, bristled crest, snout disc, curved tusks, ears, tail, and legs"
    );
    for quad in &mesh.quads {
        quad.transfer(|v0, v1, v2, v3, normal, _| {
            assert!(v0.is_finite() && v1.is_finite() && v2.is_finite() && v3.is_finite());
            assert!((normal.length() - 1.0).abs() < 1e-4, "Wild boar quad normal must be normalized");
        });
    }

    let listener = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let mut fauna = WoodlandFauna::seeded();
    let _ = fauna.step_spatial(1.0 / 60.0, listener, Vec3::NEG_Z, false, &world);
    assert_eq!(fauna.boars().boars().len(), 3);
    let mut culled = FacetBatch::empty();
    fauna.emit_visible(SightCone::from_view(Vec3::new(170.0, 0.0, 0.0), Vec3::X), &mut culled);
    assert_eq!(culled.quads.len(), 0, "SightCone must cull distant off-face woodland fauna");
    let allocs = crate::ALLOCATOR.measure(|| {
        for _ in 0..90 {
            let _ = fauna.step_spatial(1.0 / 60.0, listener, Vec3::NEG_Z, false, &world);
        }
    });
    assert_eq!(allocs, 0, "WoodlandFauna with BoarSounder must allocate zero heap memory");
}

#[test]
fn test_wild_boar_snorts_in_warning_and_gores_intruder_with_tusk_charge() {
    let snort = BoarSnort::spatial(Vec3::new(3.5, 17.0, -1.0), Vec3::new(0.0, 17.0, 0.0), Vec3::NEG_Z, Vec3::Y, 13);
    assert_eq!(snort.sample_rate(), 24000);
    let grunt_peak = (0..18).map(|k| snort.amplitude_at(0.11 + (k as f32) * 0.002).abs()).fold(0.0_f32, f32::max);
    let roar_peak = (0..18).map(|k| snort.amplitude_at(0.40 + (k as f32) * 0.002).abs()).fold(0.0_f32, f32::max);
    assert!(grunt_peak > 0.14 && roar_peak > 0.16);
    let (gain_l, gain_r) = snort.channel_gains();
    assert!(gain_r > gain_l * 1.5);
    let wav = snort.stereo_riff_wave();
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 2);

    let world = BiomedCubeWorld::beveled(32, 1.0);
    let clearing = world.constrain_to_surface(Vec3::new(2.0, 20.0, -2.0), 0.0);
    let mut boar = WildBoar::new(
        BoarActivity::Rooting { pos: clearing, heading: Vec3::NEG_X, timer: 4.0, phase: 0.0 },
        Vec3::Y,
        91,
        (8, 0),
    );

    let intruder_warn = clearing + Vec3::new(7.0, 0.0, 0.0);
    let (warned, shoved_warn) = boar.step(1.0 / 60.0, intruder_warn, false, &world);
    assert!(warned && shoved_warn.is_none() && boar.bristled());

    let intruder_close = clearing + Vec3::new(3.6, 0.0, 0.0);
    let (charged, _) = boar.step(1.0 / 60.0, intruder_close, false, &world);
    assert!(charged && boar.charging());

    let mut gored = None;
    for _ in 0..40 {
        let (_, hit) = boar.step(1.0 / 30.0, intruder_close, false, &world);
        if hit.is_some() {
            gored = hit;
            break;
        }
    }
    assert!(gored.is_some(), "Charging wild boar must reach intruder and deliver tusk knockback");
}

#[test]
fn test_buried_or_walled_wild_boar_relocates_and_never_gores_through_obstacles() {
    use world::{BlockMold, WorldEarthworks};

    let mut world = BiomedCubeWorld::barren_beveled(32, 1.0);
    let spot = world.constrain_to_surface(Vec3::new(3.25, 24.0, 0.25), 0.0);
    let mut buried = WildBoar::new(
        BoarActivity::Rooting { pos: spot, heading: Vec3::NEG_X, timer: 4.0, phase: 0.0 },
        Vec3::Y,
        411,
        (8, 0),
    );

    // Build a 2m (4-layer) column directly on top of the rooting boar's spot
    let avatar_stand = world.constrain_to_surface(Vec3::new(5.25, 24.0, 0.25), 1.0);
    for layer in 0..4 {
        let aim_y = spot.y + (layer as f32) * 0.5;
        let eye = Vec3::new(spot.x, aim_y + 1.4, spot.z);
        assert!(world.place_molded_surface_block(eye, Vec3::NEG_Y, avatar_stand, BlockMold::cube()));
    }
    let roof_top = world.constrain_to_surface(Vec3::new(spot.x, spot.y + 5.0, spot.z), 1.0);
    assert!(roof_top.y > spot.y + 2.5);

    // Stepping the buried boar must relocate it onto open Temperate ground rather than goring through the roof
    let (_, hit_from_underground) = buried.step(1.0 / 60.0, roof_top, false, &world);
    assert!(hit_from_underground.is_none());
    assert!((buried.position() - spot).length() > 3.0, "Buried boar must relocate out from beneath solid structures");

    // Place a player on top of the 2m stone pedestal and aim a charging boar directly at the pedestal base
    let base_start = world.constrain_to_surface(Vec3::new(spot.x - 2.0, 24.0, spot.z), 0.0);
    let mut charger = WildBoar::new(
        BoarActivity::TuskCharge { pos: base_start, target: roof_top, phase: 0.0, timer: 2.5 },
        Vec3::Y,
        523,
        (8, 0),
    );
    for _ in 0..45 {
        let (_, ped_hit) = charger.step(1.0 / 30.0, roof_top, false, &world);
        assert!(ped_hit.is_none(), "Charging boar at base of 2m stone pedestal must never gore player on top");
    }
}
