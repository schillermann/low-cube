use std::f32::consts::FRAC_PI_2;
use glam::{Quat, Vec3};
use crate::frame::SurfaceFrame;
use super::cube_face::CubeFace;
use super::folded_point::FoldedPoint;
use super::perimeter::PerimeterEdge;

/// Interface for geometric coordinate and vector folding across seam boundaries.
pub trait CoordinateFold: Send + Sync {
    fn fold_point(&self, point: Vec3) -> Vec3;
    fn fold_vector(&self, vector: Vec3) -> Vec3;
    fn fold_step(&self, point: Vec3) -> FoldedPoint;
}

/// Encapsulated 90° seam fold connecting two adjacent faces across a shared perimeter edge.
#[derive(Clone, Copy, Debug)]
pub struct SeamFold {
    source: CubeFace,
    source_edge: PerimeterEdge,
    destination: CubeFace,
    destination_edge: PerimeterEdge,
    rotation: Quat,
    seam_center: Vec3,
}

impl SeamFold {
    /// Primary constructor: assigns transformation fields directly without computation.
    pub fn new(
        source: CubeFace,
        source_edge: PerimeterEdge,
        destination: CubeFace,
        destination_edge: PerimeterEdge,
        rotation: Quat,
        seam_center: Vec3,
    ) -> Self {
        Self {
            source,
            source_edge,
            destination,
            destination_edge,
            rotation,
            seam_center,
        }
    }

    /// Secondary constructor assembling 90° rotation and seam center from adjacent faces and half size.
    pub fn between(
        source: CubeFace,
        source_edge: PerimeterEdge,
        destination: CubeFace,
        destination_edge: PerimeterEdge,
        half_size: f32,
    ) -> Self {
        let n_src = source.normal();
        let n_dst = destination.normal();
        let axis = n_src.cross(n_dst).normalize();
        let rotation = Quat::from_axis_angle(axis, FRAC_PI_2);
        let seam_center = (n_src + n_dst) * half_size;

        Self::new(
            source,
            source_edge,
            destination,
            destination_edge,
            rotation,
            seam_center,
        )
    }

    /// Predicate verifying whether this seam fold connects the specified face and outer edge.
    pub fn connects(&self, face: CubeFace, edge: PerimeterEdge) -> bool {
        self.source == face && self.source_edge == edge
    }

    /// Predicate verifying whether another seam fold represents the exact reciprocal transition.
    pub fn is_reciprocal_to(&self, other: &Self) -> bool {
        self.source == other.destination
            && self.source_edge == other.destination_edge
            && self.destination == other.source
            && self.destination_edge == other.source_edge
    }

    /// Destination face onto which motion unfolds across the seam.
    pub fn destination_face(&self) -> CubeFace {
        self.destination
    }

    /// Reciprocal outer edge boundary on the destination face.
    pub fn destination_edge(&self) -> PerimeterEdge {
        self.destination_edge
    }

    /// Folds a 3D position across the seam by rotating 90° around the seam axis.
    pub fn fold_point(&self, point: Vec3) -> Vec3 {
        self.seam_center + self.rotation * (point - self.seam_center)
    }

    /// Folds a 3D position across the seam, returning a living FoldedPoint entity.
    pub fn fold_step(&self, point: Vec3) -> FoldedPoint {
        FoldedPoint::new(self.destination, self.fold_point(point))
    }

    /// Folds a 3D direction or velocity vector across the 90° seam rotation.
    pub fn fold_vector(&self, vector: Vec3) -> Vec3 {
        self.rotation * vector
    }

    /// Decomposes and folds tangential coordinates from source surface frame to destination surface frame.
    pub fn fold_coordinates(
        &self,
        frame_src: &SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
    ) -> FoldedPoint {
        let world_pt = frame_src.point_at(u, v, elevation);
        self.fold_step(world_pt)
    }
}

impl CoordinateFold for SeamFold {
    fn fold_point(&self, point: Vec3) -> Vec3 {
        self.fold_point(point)
    }

    fn fold_vector(&self, vector: Vec3) -> Vec3 {
        self.fold_vector(vector)
    }

    fn fold_step(&self, point: Vec3) -> FoldedPoint {
        self.fold_step(point)
    }
}
