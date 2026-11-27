use glam::Vec3;
use world::{
    BlockMedium, ClimateZone, CubePlanet, FacetedReliefFace, MeshTarget, NullFace, PlanetaryWorld,
    Quad, WorldEarthworks,
};
use crate::attire::{EvaAttire, HeldRelic};
use crate::avatar::{Avatar, AvatarMotor, AvatarPresence, PlanetaryAvatar};
use crate::controls::InputTrigger;

struct HologramBounds {
    min_pt: Vec3,
    max_pt: Vec3,
    quads: usize,
}

impl HologramBounds {
    fn empty() -> Self {
        Self {
            min_pt: Vec3::splat(f32::MAX),
            max_pt: Vec3::splat(f32::MIN),
            quads: 0,
        }
    }

    fn span(&self) -> Vec3 {
        if self.quads == 0 {
            Vec3::ZERO
        } else {
            self.max_pt - self.min_pt
        }
    }
}

impl MeshTarget for HologramBounds {
    fn push_quad(&mut self, quad: Quad) {
        self.quads += 1;
        quad.transfer(|v0, v1, v2, v3, _, _| {
            for p in [v0, v1, v2, v3] {
                self.min_pt = self.min_pt.min(p);
                self.max_pt = self.max_pt.max(p);
            }
        });
    }
}

#[test]
fn test_drag_hologram_stretches_scalable_tank_and_proxy_dismantles_atomically() {
    let face = Box::new(FacetedReliefFace::in_zone(ClimateZone::Temperate, 8.0, 32));
    let mut world = CubePlanet::new(vec![face], Box::new(NullFace));

    let spawn = world.constrain_to_surface(Vec3::new(0.25, 16.0, 2.25), 1.0);
    let mut builder = PlanetaryAvatar::at(spawn);
    builder.apply_attire(
        EvaAttire::astronaut().with_relics(HeldRelic::PropellantTank, HeldRelic::EmptyPalm),
    );

    // Aim down-forward onto cell (0, 1) (x in 0.0..0.5, z in 0.5..1.0)
    builder.rotate_degrees(0.0, 48.0);
    assert!(!builder.use_right_hand(&mut world), "Mouse press on scalable module begins drag without immediate placement");

    // Drag across to cell (2, 1) and scroll MoldRight (WheelDown) +3 steps -> 3 x 1 x 4 cells (1.5m x 0.5m x 2.0m)
    builder.rotate_degrees(38.0, -6.0);
    assert!(builder.dispatch_trigger(InputTrigger::WheelDown));
    assert!(builder.dispatch_trigger(InputTrigger::WheelDown));
    assert!(builder.dispatch_trigger(InputTrigger::WheelDown));

    let mut holo = HologramBounds::empty();
    builder.emit_placement_preview(&world, &mut holo);
    let span = holo.span();
    assert!(holo.quads >= 36, "Stretched drag hologram must emit 12 corner brackets, got {}", holo.quads);
    assert!(
        span.x > 0.90 && span.y > 1.85,
        "Dragged + scrolled hologram must stretch horizontally and vertically, got span={span:?}"
    );

    assert!(builder.release_hand(&mut world), "Releasing drag must commit the scaled PropellantTank module");

    // Verify both the anchor corner and a far ModuleProxy corner are solid PropellantTank cells
    let probe_far = Vec3::new(0.75, 10.2, 0.75);
    assert_eq!(
        world.aimed_surface_medium(probe_far, Vec3::NEG_Y),
        Some(BlockMedium::PropellantTank),
        "Far ModuleProxy cell must report PropellantTank medium"
    );
    let probe_origin = Vec3::new(0.25, 10.2, 0.75);
    assert_eq!(
        world.aimed_surface_medium(probe_origin, Vec3::NEG_Y),
        Some(BlockMedium::PropellantTank),
        "Anchor cell must also report PropellantTank medium"
    );

    // Dismantling the far ModuleProxy cell must atomically remove the entire multi-cell tank!
    assert_eq!(
        world.harvest_surface(probe_far, Vec3::NEG_Y),
        Some(BlockMedium::PropellantTank)
    );
    assert_ne!(
        world.aimed_surface_medium(probe_origin, Vec3::NEG_Y),
        Some(BlockMedium::PropellantTank),
        "Atomic ModuleProxy dismantling must clear the anchor cell and all sibling proxies"
    );
}
