use glam::Vec3;

mod bevel;
mod climate_zone;
mod folded_face;
mod folded_point;
mod perimeter;
mod seam;
#[cfg(test)]
mod bevel_tests;
#[cfg(test)]
mod corner_tests;
#[cfg(test)]
mod tests;

pub use bevel::{BevelTransition, SeamBevel};
pub use climate_zone::ClimateZone;
pub type CubeFace = ClimateZone;
pub use folded_face::FoldedFace;
pub use folded_point::FoldedPoint;
pub use perimeter::{FacePerimeter, PerimeterCrossing, PerimeterEdge};
pub use seam::{CoordinateFold, SeamFold};

/// Interface representing topological planetary face connectivity and seam navigation.
pub trait Topology: Send + Sync {
    fn seam(&self, zone: ClimateZone, edge: PerimeterEdge) -> &SeamFold;
    fn neighbor_of(&self, zone: ClimateZone, edge: PerimeterEdge) -> ClimateZone;
    fn fold_point(&self, zone: ClimateZone, edge: PerimeterEdge, point: Vec3) -> FoldedPoint;
    fn fold_vector(&self, zone: ClimateZone, edge: PerimeterEdge, vector: Vec3) -> Vec3;
}

/// Planetary topology connecting the six faces of a cubic world across 12 seam neighborhoods (24 directed folds).
pub struct PlanetaryTopology {
    folds: [SeamFold; 24],
}

impl PlanetaryTopology {
    /// Primary constructor: assigns dense array of 24 seam folds directly.
    pub fn new(folds: [SeamFold; 24]) -> Self {
        Self { folds }
    }

    /// Secondary constructor assembling the 24 directed seam folds without half-born dummy objects.
    pub fn cubic(half_size: f32) -> Self {
        let transitions: [(ClimateZone, PerimeterEdge, ClimateZone, PerimeterEdge); 24] = [
            // Temperate (+Y)
            (ClimateZone::Temperate, PerimeterEdge::PositiveU, ClimateZone::Desert, PerimeterEdge::PositiveV),
            (ClimateZone::Temperate, PerimeterEdge::NegativeU, ClimateZone::Arctic, PerimeterEdge::PositiveV),
            (ClimateZone::Temperate, PerimeterEdge::PositiveV, ClimateZone::Jungle, PerimeterEdge::PositiveV),
            (ClimateZone::Temperate, PerimeterEdge::NegativeV, ClimateZone::Steppe, PerimeterEdge::PositiveV),
            // Volcanic (-Y)
            (ClimateZone::Volcanic, PerimeterEdge::PositiveU, ClimateZone::Desert, PerimeterEdge::NegativeV),
            (ClimateZone::Volcanic, PerimeterEdge::NegativeU, ClimateZone::Arctic, PerimeterEdge::NegativeV),
            (ClimateZone::Volcanic, PerimeterEdge::PositiveV, ClimateZone::Steppe, PerimeterEdge::NegativeV),
            (ClimateZone::Volcanic, PerimeterEdge::NegativeV, ClimateZone::Jungle, PerimeterEdge::NegativeV),
            // Desert (+X)
            (ClimateZone::Desert, PerimeterEdge::PositiveU, ClimateZone::Steppe, PerimeterEdge::NegativeU),
            (ClimateZone::Desert, PerimeterEdge::NegativeU, ClimateZone::Jungle, PerimeterEdge::PositiveU),
            (ClimateZone::Desert, PerimeterEdge::PositiveV, ClimateZone::Temperate, PerimeterEdge::PositiveU),
            (ClimateZone::Desert, PerimeterEdge::NegativeV, ClimateZone::Volcanic, PerimeterEdge::PositiveU),
            // Arctic (-X)
            (ClimateZone::Arctic, PerimeterEdge::PositiveU, ClimateZone::Jungle, PerimeterEdge::NegativeU),
            (ClimateZone::Arctic, PerimeterEdge::NegativeU, ClimateZone::Steppe, PerimeterEdge::PositiveU),
            (ClimateZone::Arctic, PerimeterEdge::PositiveV, ClimateZone::Temperate, PerimeterEdge::NegativeU),
            (ClimateZone::Arctic, PerimeterEdge::NegativeV, ClimateZone::Volcanic, PerimeterEdge::NegativeU),
            // Jungle (+Z)
            (ClimateZone::Jungle, PerimeterEdge::PositiveU, ClimateZone::Desert, PerimeterEdge::NegativeU),
            (ClimateZone::Jungle, PerimeterEdge::NegativeU, ClimateZone::Arctic, PerimeterEdge::PositiveU),
            (ClimateZone::Jungle, PerimeterEdge::PositiveV, ClimateZone::Temperate, PerimeterEdge::PositiveV),
            (ClimateZone::Jungle, PerimeterEdge::NegativeV, ClimateZone::Volcanic, PerimeterEdge::NegativeV),
            // Steppe (-Z)
            (ClimateZone::Steppe, PerimeterEdge::PositiveU, ClimateZone::Arctic, PerimeterEdge::NegativeU),
            (ClimateZone::Steppe, PerimeterEdge::NegativeU, ClimateZone::Desert, PerimeterEdge::PositiveU),
            (ClimateZone::Steppe, PerimeterEdge::PositiveV, ClimateZone::Temperate, PerimeterEdge::NegativeV),
            (ClimateZone::Steppe, PerimeterEdge::NegativeV, ClimateZone::Volcanic, PerimeterEdge::PositiveV),
        ];

        let folds = std::array::from_fn(|slot| {
            let (src, src_edge, dst, dst_edge) = transitions[slot];
            SeamFold::between(src, src_edge, dst, dst_edge, half_size)
        });

        Self::new(folds)
    }

    /// Total count of encapsulated seam folds.
    pub fn fold_count(&self) -> usize {
        self.folds.len()
    }

    /// Derives the four seam folds for a given climate zone.
    pub fn seams_for(&self, zone: ClimateZone) -> [SeamFold; 4] {
        [
            *self.seam(zone, PerimeterEdge::PositiveU),
            *self.seam(zone, PerimeterEdge::NegativeU),
            *self.seam(zone, PerimeterEdge::PositiveV),
            *self.seam(zone, PerimeterEdge::NegativeV),
        ]
    }

    /// Derives the seam bevel transition zone for a climate zone and edge.
    pub fn bevel(&self, zone: ClimateZone, edge: PerimeterEdge, half_size: f32, margin: f32) -> SeamBevel {
        self.seam(zone, edge).bevel(half_size, margin)
    }
}

impl Topology for PlanetaryTopology {
    fn seam(&self, zone: ClimateZone, edge: PerimeterEdge) -> &SeamFold {
        let slot = zone.index() * 4 + edge.index();
        &self.folds[slot]
    }

    fn neighbor_of(&self, zone: ClimateZone, edge: PerimeterEdge) -> ClimateZone {
        self.seam(zone, edge).destination()
    }

    fn fold_point(&self, zone: ClimateZone, edge: PerimeterEdge, point: Vec3) -> FoldedPoint {
        self.seam(zone, edge).fold_step(point)
    }

    fn fold_vector(&self, zone: ClimateZone, edge: PerimeterEdge, vector: Vec3) -> Vec3 {
        self.seam(zone, edge).fold_vector(vector)
    }
}
