use glam::Vec3;

mod cube_face;
mod folded_point;
mod perimeter;
mod seam;
#[cfg(test)]
mod tests;

pub use cube_face::CubeFace;
pub use folded_point::FoldedPoint;
pub use perimeter::{FacePerimeter, PerimeterCrossing, PerimeterEdge};
pub use seam::{CoordinateFold, SeamFold};

/// Interface representing topological planetary face connectivity and seam navigation.
pub trait Topology: Send + Sync {
    fn seam(&self, face: CubeFace, edge: PerimeterEdge) -> &SeamFold;
    fn neighbor_of(&self, face: CubeFace, edge: PerimeterEdge) -> CubeFace;
    fn fold_point(&self, face: CubeFace, edge: PerimeterEdge, point: Vec3) -> FoldedPoint;
    fn fold_vector(&self, face: CubeFace, edge: PerimeterEdge, vector: Vec3) -> Vec3;
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
        let transitions: [(CubeFace, PerimeterEdge, CubeFace, PerimeterEdge); 24] = [
            // Top (+Y)
            (CubeFace::Top, PerimeterEdge::PositiveU, CubeFace::Right, PerimeterEdge::PositiveV),
            (CubeFace::Top, PerimeterEdge::NegativeU, CubeFace::Left, PerimeterEdge::PositiveV),
            (CubeFace::Top, PerimeterEdge::PositiveV, CubeFace::Front, PerimeterEdge::PositiveV),
            (CubeFace::Top, PerimeterEdge::NegativeV, CubeFace::Back, PerimeterEdge::PositiveV),
            // Bottom (-Y)
            (CubeFace::Bottom, PerimeterEdge::PositiveU, CubeFace::Right, PerimeterEdge::NegativeV),
            (CubeFace::Bottom, PerimeterEdge::NegativeU, CubeFace::Left, PerimeterEdge::NegativeV),
            (CubeFace::Bottom, PerimeterEdge::PositiveV, CubeFace::Back, PerimeterEdge::NegativeV),
            (CubeFace::Bottom, PerimeterEdge::NegativeV, CubeFace::Front, PerimeterEdge::NegativeV),
            // Right (+X)
            (CubeFace::Right, PerimeterEdge::PositiveU, CubeFace::Back, PerimeterEdge::NegativeU),
            (CubeFace::Right, PerimeterEdge::NegativeU, CubeFace::Front, PerimeterEdge::PositiveU),
            (CubeFace::Right, PerimeterEdge::PositiveV, CubeFace::Top, PerimeterEdge::PositiveU),
            (CubeFace::Right, PerimeterEdge::NegativeV, CubeFace::Bottom, PerimeterEdge::PositiveU),
            // Left (-X)
            (CubeFace::Left, PerimeterEdge::PositiveU, CubeFace::Front, PerimeterEdge::NegativeU),
            (CubeFace::Left, PerimeterEdge::NegativeU, CubeFace::Back, PerimeterEdge::PositiveU),
            (CubeFace::Left, PerimeterEdge::PositiveV, CubeFace::Top, PerimeterEdge::NegativeU),
            (CubeFace::Left, PerimeterEdge::NegativeV, CubeFace::Bottom, PerimeterEdge::NegativeU),
            // Front (+Z)
            (CubeFace::Front, PerimeterEdge::PositiveU, CubeFace::Right, PerimeterEdge::NegativeU),
            (CubeFace::Front, PerimeterEdge::NegativeU, CubeFace::Left, PerimeterEdge::PositiveU),
            (CubeFace::Front, PerimeterEdge::PositiveV, CubeFace::Top, PerimeterEdge::PositiveV),
            (CubeFace::Front, PerimeterEdge::NegativeV, CubeFace::Bottom, PerimeterEdge::NegativeV),
            // Back (-Z)
            (CubeFace::Back, PerimeterEdge::PositiveU, CubeFace::Left, PerimeterEdge::NegativeU),
            (CubeFace::Back, PerimeterEdge::NegativeU, CubeFace::Right, PerimeterEdge::PositiveU),
            (CubeFace::Back, PerimeterEdge::PositiveV, CubeFace::Top, PerimeterEdge::NegativeV),
            (CubeFace::Back, PerimeterEdge::NegativeV, CubeFace::Bottom, PerimeterEdge::PositiveV),
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
}

impl Topology for PlanetaryTopology {
    fn seam(&self, face: CubeFace, edge: PerimeterEdge) -> &SeamFold {
        let slot = face.index() * 4 + edge.index();
        &self.folds[slot]
    }

    fn neighbor_of(&self, face: CubeFace, edge: PerimeterEdge) -> CubeFace {
        self.seam(face, edge).destination_face()
    }

    fn fold_point(&self, face: CubeFace, edge: PerimeterEdge, point: Vec3) -> FoldedPoint {
        self.seam(face, edge).fold_step(point)
    }

    fn fold_vector(&self, face: CubeFace, edge: PerimeterEdge, vector: Vec3) -> Vec3 {
        self.seam(face, edge).fold_vector(vector)
    }
}
