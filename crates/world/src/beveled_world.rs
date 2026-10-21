use glam::Vec3;
use crate::{
    ClimateZone, GravityField, MeshTarget, PerimeterEdge, PlanetaryTopology,
    PlanetaryWorld, SeamBevel,
};

/// Decorator enabling smooth, continuous surface normal transitions in bevel zones across planetary seams.
pub struct BeveledWorld {
    world: Box<dyn PlanetaryWorld>,
    topology: PlanetaryTopology,
    half_size: f32,
    margin: f32,
}

impl BeveledWorld {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        world: Box<dyn PlanetaryWorld>,
        topology: PlanetaryTopology,
        half_size: f32,
        margin: f32,
    ) -> Self {
        Self {
            world,
            topology,
            half_size,
            margin,
        }
    }

    /// Secondary constructor deriving cubic topology for a world of specified half size.
    pub fn decorating(world: Box<dyn PlanetaryWorld>, half_size: f32, margin: f32) -> Self {
        let topology = PlanetaryTopology::cubic(half_size);
        Self::new(world, topology, half_size, margin)
    }

    /// Dispatches continuous bevel behavior if within a seam transition zone ("Tell, Don't Ask").
    fn with_bevel<T>(
        &self,
        position: Vec3,
        on_bevel: impl FnOnce(SeamBevel) -> T,
        on_origin: impl FnOnce() -> T,
    ) -> T {
        let dominant = ClimateZone::dominant(position);
        for edge in PerimeterEdge::all() {
            let bevel = self.topology.bevel(dominant, edge, self.half_size, self.margin);
            if bevel.contains(position) {
                return on_bevel(bevel);
            }
        }
        on_origin()
    }
}

impl GravityField for BeveledWorld {
    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.with_bevel(
            position,
            |bevel| bevel.normal_at(position),
            || self.world.surface_up_at(position),
        )
    }

    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        -self.surface_up_at(position)
    }
}

impl PlanetaryWorld for BeveledWorld {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.world.constrain_to_surface(position, player_height)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.world.emit_surface(target);
    }

    fn canopy_drag(&self, position: Vec3) -> f32 {
        self.world.canopy_drag(position)
    }

    fn excavate_surface(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.world.excavate_surface(eye, direction)
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> bool {
        self.world.loosen_surface(eye, direction, stage)
    }

    fn place_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.world.place_surface_block(eye, direction, avatar_pos)
    }

    fn face_count(&self) -> usize {
        self.world.face_count()
    }

    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) {
        self.world.emit_face(index, target);
    }

    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        self.world.emit_modified_face(target)
    }
}
