use std::collections::HashMap;
use glam::Vec3;
use crate::{ClimateZone, MeshTarget, Quad, SurfaceFrame};
use super::{Arboreal, TimberDepository, TimberYield, ZonedForest};

struct TestDepository {
    tallies: HashMap<TimberYield, u32>,
}

impl TestDepository {
    fn empty() -> Self {
        Self {
            tallies: HashMap::new(),
        }
    }

    fn amount_of(&self, yield_kind: TimberYield) -> u32 {
        *self.tallies.get(&yield_kind).unwrap_or(&0)
    }
}

impl TimberDepository for TestDepository {
    fn deposit_timber(&mut self, yield_kind: TimberYield, quantity: u32) {
        *self.tallies.entry(yield_kind).or_insert(0) += quantity;
    }
}

struct TestMeshTarget {
    quads: Vec<Quad>,
}

impl TestMeshTarget {
    fn empty() -> Self {
        Self { quads: Vec::new() }
    }

    fn count(&self) -> usize {
        self.quads.len()
    }
}

impl MeshTarget for TestMeshTarget {
    fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }
}

#[test]
fn temperate_tree_harvest_lifecycle_transitions() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 32.0);
    let forest = ZonedForest::new(ClimateZone::Temperate);
    let tree: Box<dyn Arboreal> = forest.plant(frame, 0.0, 0.0, 0.0);

    assert!(tree.solid());
    assert!(tree.standing());

    // Physical trunk blocks avatar motion within trunk radius
    let ground = frame.point_at(0.2, 0.2, 1.0);
    let constrained = tree.constrain_surface(ground, 0.2, 0.2, &frame, 1.0);
    assert!(
        frame.height_of(constrained) > frame.height_of(ground),
        "Standing tree must physically block movement through trunk"
    );

    let mut depository = TestDepository::empty();

    // First strike: vitality 3 -> 2
    let tree = tree.strike(1, &mut depository);
    assert!(tree.standing());
    assert_eq!(depository.amount_of(TimberYield::StandardLumber), 0);

    // Second strike: vitality 2 -> 1
    let tree = tree.strike(1, &mut depository);
    assert!(tree.standing());

    // Fatal strike: vitality 1 -> 0, yields harvest and vanishes completely into ClearedPlot
    let plot = tree.strike(1, &mut depository);
    assert!(!plot.standing());
    assert!(!plot.solid());
    assert_eq!(depository.amount_of(TimberYield::StandardLumber), 4);
    assert_eq!(depository.amount_of(TimberYield::Twig), 2);
    assert_eq!(depository.amount_of(TimberYield::Sapling), 1);

    // Cleared plot no longer constrains avatar motion
    let unblocked = plot.constrain_surface(ground, 0.2, 0.2, &frame, 1.0);
    assert_eq!(unblocked, ground);

    // Striking cleared plot does nothing
    let plot = plot.strike(1, &mut depository);
    assert!(!plot.solid());
}

#[test]
fn all_climate_zones_render_and_yield_distinct_resources() {
    let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 32.0);

    let expectations = [
        (ClimateZone::Temperate, TimberYield::StandardLumber),
        (ClimateZone::Arctic, TimberYield::AlpineHardwood),
        (ClimateZone::Jungle, TimberYield::TropicalHardwood),
        (ClimateZone::Desert, TimberYield::SucculentFiber),
        (ClimateZone::Steppe, TimberYield::SteppeDrywood),
        (ClimateZone::Volcanic, TimberYield::VolcanicBasaltwood),
    ];

    for (zone, expected_yield) in expectations {
        let forest = ZonedForest::new(zone);
        let tree = forest.plant(frame, 1.0, 2.0, 0.0);

        let mut target = TestMeshTarget::empty();
        tree.emit_surface(&mut target);
        assert!(target.count() > 0, "Zone {:?} must emit quads", zone);

        let mut depository = TestDepository::empty();
        let stump = tree.strike(10, &mut depository);
        assert!(!stump.standing());
        assert!(
            depository.amount_of(expected_yield) > 0,
            "Zone {:?} must yield {:?}",
            zone,
            expected_yield
        );
    }
}

#[test]
fn test_biomed_cube_world_contains_trees_across_biomes() {
    use crate::{BiomedCubeWorld, PlanetaryWorld};
    let world = BiomedCubeWorld::of_size(32);
    let mut target = TestMeshTarget::empty();
    world.emit_surface(&mut target);

    assert!(
        target.count() > 6144 + 500,
        "BiomedCubeWorld must emit surface quads including planted trees, got {}",
        target.count()
    );
}



