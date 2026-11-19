use glam::Vec3;
use super::{ClimateZone, PerimeterEdge, PerimeterFold, PlanetaryTopology};

/// Narrow 2-method boundary interface governing perimeter coordinate folding and seam normal beveling.
pub trait SeamBoundary: Send + Sync {
    fn blend_normal(&self, position: Vec3, face_up: Vec3) -> Vec3;
    fn fold_constrained(&self, face_index: usize, raw_point: Vec3, constrained: Vec3) -> Vec3;
}

/// Null-Object `SeamBoundary` leaving planar single-face coordinates and normals untouched.
pub struct OpenBoundary;

impl OpenBoundary {
    /// Primary constructor initializing the planar Null-Object boundary.
    pub const fn new() -> Self {
        Self
    }
}

impl SeamBoundary for OpenBoundary {
    fn blend_normal(&self, _position: Vec3, face_up: Vec3) -> Vec3 {
        face_up
    }

    fn fold_constrained(&self, _face_index: usize, _raw_point: Vec3, constrained: Vec3) -> Vec3 {
        constrained
    }
}

/// Sharp 6-face cubic boundary folding perimeter crossings across all climate zones.
pub struct CubicBoundary {
    folds: [PerimeterFold; 6],
}

impl CubicBoundary {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(folds: [PerimeterFold; 6]) -> Self {
        Self { folds }
    }

    /// Secondary constructor assembling perimeter folds for all six canonical climate zones.
    pub fn sharp(half_size: f32) -> Self {
        let zones = ClimateZone::all();
        Self::new(std::array::from_fn(|idx| PerimeterFold::in_zone(zones[idx], half_size)))
    }
}

impl SeamBoundary for CubicBoundary {
    fn blend_normal(&self, _position: Vec3, face_up: Vec3) -> Vec3 {
        face_up
    }

    fn fold_constrained(&self, face_index: usize, raw_point: Vec3, constrained: Vec3) -> Vec3 {
        self.folds
            .get(face_index)
            .map_or(constrained, |f| f.fold(raw_point, constrained))
    }
}

/// Focused 2-method `SeamBoundary` decorator smoothing surface normals continuously across cube seam bevels.
pub struct BeveledBoundary<B> {
    origin: B,
    topology: PlanetaryTopology,
    half_size: f32,
    margin: f32,
}

impl<B> BeveledBoundary<B> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: B, topology: PlanetaryTopology, half_size: f32, margin: f32) -> Self {
        Self { origin, topology, half_size, margin }
    }

    /// Secondary constructor decorating `origin` with cubic seam bevels.
    pub fn decorating(origin: B, half_size: f32, margin: f32) -> Self {
        Self::new(origin, PlanetaryTopology::cubic(half_size), half_size, margin)
    }
}

impl<B: SeamBoundary> SeamBoundary for BeveledBoundary<B> {
    fn blend_normal(&self, position: Vec3, face_up: Vec3) -> Vec3 {
        let dominant = ClimateZone::dominant(position);
        for edge in PerimeterEdge::all() {
            let bevel = self.topology.bevel(dominant, edge, self.half_size, self.margin);
            if bevel.contains(position) {
                return bevel.normal_at(position);
            }
        }
        self.origin.blend_normal(position, face_up)
    }

    fn fold_constrained(&self, face_index: usize, raw_point: Vec3, constrained: Vec3) -> Vec3 {
        self.origin.fold_constrained(face_index, raw_point, constrained)
    }
}
