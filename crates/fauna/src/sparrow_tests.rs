use glam::Vec3;
use world::{BiomedCubeWorld, MeshTarget, PlanetaryWorld, Quad};
use acoustics::{AcousticWave, SparrowChirp};
use crate::sparrow::{SparrowActivity, SparrowFlock, TestSparrow as Sparrow};

struct QuadCollector {
    quads: Vec<Quad>,
}

impl QuadCollector {
    fn empty() -> Self {
        Self { quads: Vec::new() }
    }
}

impl MeshTarget for QuadCollector {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

#[test]
fn test_sparrow_flock_flies_pecks_ground_and_sits_on_trees_with_zero_allocations() {
    let world = BiomedCubeWorld::beveled(32, 1.0);
    let listener = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let mut flock = SparrowFlock::seeded();

    flock.step_spatial(1.0 / 60.0, listener, Vec3::NEG_Z, &world);

    let flying_count = flock.birds().iter().filter(|b| b.airborne()).count();
    let pecking_count = flock.birds().iter().filter(|b| b.ground_pecking()).count();
    let perched_count = flock.birds().iter().filter(|b| b.tree_perched()).count();

    assert!(flying_count >= 2, "Flock must include sparrows actively flying around");
    assert!(pecking_count >= 2, "Flock must include sparrows pecking on the ground");
    assert!(perched_count >= 2, "Flock must include sparrows sitting on standing trees");

    for bird in flock.birds().iter().filter(|b| b.tree_perched()) {
        assert!(
            world.canopy_drag(bird.position()) > 0.0,
            "Tree-perched sparrow must sit within a standing tree crown"
        );
    }

    let mut mesh = QuadCollector::empty();
    flock.emit(&mut mesh);
    assert!(
        mesh.quads.len() >= 9 * 28,
        "Each sparrow must emit sculpted 3D torso, head, beak, tail, and wing quads"
    );
    for quad in &mesh.quads {
        quad.transfer(|v0, v1, v2, v3, normal, _| {
            assert!(v0.is_finite() && v1.is_finite() && v2.is_finite() && v3.is_finite());
            assert!((normal.length() - 1.0).abs() < 1e-4, "Sparrow quad normal must be normalized");
        });
    }

    let allocs = crate::ALLOCATOR.measure(|| {
        for _ in 0..120 {
            let _ = flock.step_spatial(1.0 / 60.0, listener, Vec3::NEG_Z, &world);
        }
    });
    assert_eq!(allocs, 0, "SparrowFlock step must allocate zero heap memory");
}

#[test]
fn test_sparrow_transitions_between_flight_ground_pecking_and_tree_perching() {
    let mut world = BiomedCubeWorld::beveled(32, 1.0);
    let listener = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let perch = world
        .arboreal_perch_near(listener, 0)
        .expect("Temperate face must provide standing tree perches");
    let safe_listener = perch + Vec3::new(-9.0, -3.0, -9.0);

    let start = perch + Vec3::new(-3.0, 1.0, -2.0);
    let mut tree_bird = Sparrow::new(
        SparrowActivity::Flying {
            origin: start,
            target: perch,
            progress: 0.96,
            wing_phase: 0.0,
        },
        Vec3::Y,
        42,
    );
    assert!(tree_bird.airborne());
    let chirped_on_landing = tree_bird.step(0.25, safe_listener, &world);
    assert!(
        tree_bird.tree_perched() && chirped_on_landing,
        "Sparrow arriving at a tree crown must perch on the tree and chirp"
    );

    // Felling the tree underneath the perched sparrow must cause it to take off
    let ground_under_perch = world.constrain_to_surface(Vec3::new(perch.x, 22.0, perch.z), 0.0);
    for dx in [-0.75_f32, 0.0, 0.75] {
        for dz in [-0.75_f32, 0.0, 0.75] {
            let trunk_x = perch.x + dx;
            let trunk_z = perch.z + dz;
            let eye = Vec3::new(trunk_x, ground_under_perch.y + 0.9, trunk_z + 1.2);
            let dir = (Vec3::new(trunk_x, ground_under_perch.y + 0.65, trunk_z) - eye).normalize();
            for _ in 0..4 {
                let _ = world.strike_timber(eye, dir);
            }
        }
    }
    tree_bird.step(1.0 / 60.0, safe_listener, &world);
    assert!(
        tree_bird.airborne(),
        "Perched sparrow must immediately fly away when its tree is felled"
    );

    // Ground landing and pecking head-dip animation
    let clearing = world.constrain_to_surface(Vec3::new(2.5, 20.0, 2.5), 0.0);
    let mut ground_bird = Sparrow::new(
        SparrowActivity::Flying {
            origin: clearing + Vec3::new(2.0, 1.2, 0.0),
            target: clearing,
            progress: 0.96,
            wing_phase: 0.0,
        },
        Vec3::Y,
        99,
    );
    ground_bird.step(0.25, safe_listener, &world);
    assert!(ground_bird.ground_pecking(), "Sparrow arriving at a ground clearing must start pecking");

    let (mut upright_mesh, mut dipping_mesh) = (QuadCollector::empty(), QuadCollector::empty());
    ground_bird.emit(&mut upright_mesh);
    ground_bird.step(0.16, safe_listener, &world);
    ground_bird.emit(&mut dipping_mesh);
    let v_up: Vec<Vec3> = upright_mesh.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect();
    let v_dip: Vec<Vec3> = dipping_mesh.quads.iter().map(|q| q.transfer(|v0, _, _, _, _, _| v0)).collect();
    assert_ne!(v_up, v_dip, "Ground-pecking sparrow must animate head and beak dipping toward the turf");

    // Startling the ground-pecking sparrow by walking close makes it flee away from the human
    let bird_before_flee = ground_bird.position();
    let approaching_human = bird_before_flee + Vec3::new(-2.8, 0.0, -1.2);
    let initial_human_dist = (bird_before_flee - approaching_human).length();
    assert!(
        ground_bird.step(1.0 / 60.0, approaching_human, &world),
        "Startled sparrow must emit an alarm chirp when taking off"
    );
    assert!(ground_bird.airborne(), "Ground-pecking sparrow must take flight when a human gets too close");
    for _ in 0..30 {
        ground_bird.step(1.0 / 30.0, approaching_human, &world);
    }
    let fled_human_dist = (ground_bird.position() - approaching_human).length();
    assert!(
        fled_human_dist > initial_human_dist + 3.0,
        "Frightened sparrow must fly away from the approaching human ({initial_human_dist} -> {fled_human_dist})"
    );
}

#[test]
fn test_sparrow_chirp_synthesizes_melodic_three_syllable_birdsong_waveform() {
    let chirp = SparrowChirp::new(24000, 1.0, 0.0, 1.0);
    assert_eq!(chirp.sample_rate(), 24000);

    let syl1 = chirp.amplitude_at(0.05).abs();
    let gap1 = chirp.amplitude_at(0.12).abs();
    let syl2 = chirp.amplitude_at(0.19).abs();
    let gap2 = chirp.amplitude_at(0.27).abs();
    let syl3 = chirp.amplitude_at(0.34).abs();

    assert!(syl1 > 0.05 && syl2 > 0.05 && syl3 > 0.05, "All three chirp syllables must have audible amplitude");
    assert_eq!(gap1, 0.0, "Inter-syllable pause must be silent");
    assert_eq!(gap2, 0.0, "Inter-syllable pause must be silent");

    let wav = chirp.riff_wave();
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..12], b"WAVE");

    let world = BiomedCubeWorld::beveled(32, 1.0);
    let listener = world.constrain_to_surface(Vec3::new(0.0, 17.0, 0.0), 1.0);
    let mut flock = SparrowFlock::seeded();
    let mut chirped = false;
    for _ in 0..240 {
        if flock.step_spatial(1.0 / 30.0, listener, Vec3::NEG_Z, &world) {
            chirped = true;
            break;
        }
    }
    assert!(chirped, "SparrowFlock must emit periodic birdsong chirps near the listener");

    // Distance attenuation & directional stereo panning verification
    let near_right = SparrowChirp::spatial(
        listener + Vec3::new(2.5, 0.5, -1.0),
        listener,
        Vec3::NEG_Z,
        Vec3::Y,
        101,
    );
    let far_left = SparrowChirp::spatial(
        listener + Vec3::new(-18.0, 2.0, -4.0),
        listener,
        Vec3::NEG_Z,
        Vec3::Y,
        202,
    );
    let (nr_l, nr_r) = near_right.channel_gains();
    let (fl_l, fl_r) = far_left.channel_gains();
    assert!(
        nr_r > nr_l * 1.5,
        "Sparrow to the right of the listener gaze must pan strongly to the right stereo channel ({nr_l} vs {nr_r})"
    );
    assert!(
        fl_l > fl_r * 1.5,
        "Sparrow to the left of the listener gaze must pan strongly to the left stereo channel ({fl_l} vs {fl_r})"
    );
    assert!(
        near_right.amplitude_at(0.19).abs() > far_left.amplitude_at(0.19).abs() * 3.5,
        "Distant sparrows must sound significantly quieter than nearby sparrows"
    );
    let stereo_wav = near_right.stereo_riff_wave();
    assert_eq!(&stereo_wav[0..4], b"RIFF");
    assert_eq!(u16::from_le_bytes([stereo_wav[22], stereo_wav[23]]), 2, "Spatial chirp WAV must be 2-channel stereo");
}
