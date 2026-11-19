use glam::Vec3;
use crate::{ClimateZone, GravityField, PerimeterEdge, PlanetaryTopology};

/// Focused 2-method `GravityField` decorator smoothing surface normals continuously across planetary seam bevels.
pub struct BeveledGravity<G> {
    origin: G,
    topology: PlanetaryTopology,
    half_size: f32,
    margin: f32,
}

impl<G> BeveledGravity<G> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: G, topology: PlanetaryTopology, half_size: f32, margin: f32) -> Self {
        Self { origin, topology, half_size, margin }
    }

    /// Secondary constructor deriving cubic seam topology for a gravity field of specified half size.
    pub fn decorating(origin: G, half_size: f32, margin: f32) -> Self {
        Self::new(origin, PlanetaryTopology::cubic(half_size), half_size, margin)
    }
}

impl<G: GravityField> GravityField for BeveledGravity<G> {
    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        let dominant = ClimateZone::dominant(position);
        for edge in PerimeterEdge::all() {
            let bevel = self.topology.bevel(dominant, edge, self.half_size, self.margin);
            if bevel.contains(position) {
                return bevel.normal_at(position);
            }
        }
        self.origin.surface_up_at(position)
    }

    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        -self.surface_up_at(position)
    }
}
