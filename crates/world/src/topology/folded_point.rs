use glam::Vec3;
use super::CubeFace;

/// Encapsulated 3D spatial position folded onto a destination planetary face.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoldedPoint {
    face: CubeFace,
    point: Vec3,
}

impl FoldedPoint {
    /// Primary constructor assigning destination face and folded spatial position directly.
    pub fn new(face: CubeFace, point: Vec3) -> Self {
        Self { face, point }
    }

    /// Destination cubic face of the folded point.
    pub fn face(&self) -> CubeFace {
        self.face
    }

    /// 3D spatial position in world coordinates.
    pub fn point(&self) -> Vec3 {
        self.point
    }

    /// Surface normal of the destination face.
    pub fn surface_up(&self) -> Vec3 {
        self.face.normal()
    }

    /// Pull direction of planetary gravity on the destination face.
    pub fn pull_direction(&self) -> Vec3 {
        -self.face.normal()
    }

    /// Predicate verifying whether this folded point resides on the given face.
    pub fn is_on_face(&self, face: CubeFace) -> bool {
        self.face == face
    }

    /// Euclidean distance from this point to an arbitrary spatial target.
    pub fn distance_to(&self, target: Vec3) -> f32 {
        (self.point - target).length()
    }

    /// Feeds encapsulated face and point into a consumer without extracting raw data ("Tell, Don't Ask").
    pub fn transfer<T>(&self, consumer: impl FnOnce(CubeFace, Vec3) -> T) -> T {
        consumer(self.face, self.point)
    }
}
