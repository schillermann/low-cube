use std::f32::consts::FRAC_PI_2;
use glam::{Quat, Vec3};
use crate::frame::SurfaceFrame;
use super::climate_zone::ClimateZone;
use super::folded_point::FoldedPoint;
use super::perimeter::PerimeterEdge;
use super::Topology;

/// Interface for geometric coordinate and vector folding across `QuadSphere` shell meridians.
pub trait CoordinateFold: Send + Sync {
    fn fold_point(&self, point: Vec3) -> Vec3;
    fn fold_vector(&self, vector: Vec3) -> Vec3;
    fn fold_step(&self, point: Vec3) -> FoldedPoint;
}

/// Directed meridian seam connecting two adjacent `QuadSphere` climate shells (`4` private fields).
#[derive(Clone, Copy, Debug)]
pub struct SeamFold {
    source: (ClimateZone, PerimeterEdge),
    destination: (ClimateZone, PerimeterEdge),
    rotation: Quat,
    radius: f32,
}

impl SeamFold {
    /// Primary constructor: assigns transformation fields directly without computation.
    pub const fn new(
        source: (ClimateZone, PerimeterEdge),
        destination: (ClimateZone, PerimeterEdge),
        rotation: Quat,
        radius: f32,
    ) -> Self {
        Self { source, destination, rotation, radius }
    }

    /// Secondary constructor assembling meridian chart rotation and radius between adjacent climate shells.
    pub fn between(
        source: ClimateZone,
        source_edge: PerimeterEdge,
        destination: ClimateZone,
        destination_edge: PerimeterEdge,
        radius: f32,
    ) -> Self {
        let n_src = source.normal();
        let n_dst = destination.normal();
        let axis = n_src.cross(n_dst).normalize();
        let rotation = Quat::from_axis_angle(axis, FRAC_PI_2);
        Self::new((source, source_edge), (destination, destination_edge), rotation, radius)
    }

    /// Predicate verifying whether this seam fold connects the specified climate zone and outer edge.
    pub fn connects(&self, zone: ClimateZone, edge: PerimeterEdge) -> bool {
        self.source == (zone, edge)
    }

    /// Predicate verifying whether another seam fold represents the exact reciprocal transition.
    pub fn reciprocal_to(&self, other: &Self) -> bool {
        self.source == other.destination && self.destination == other.source
    }

    /// Destination climate zone onto which motion unfolds across the meridian.
    pub fn destination(&self) -> ClimateZone {
        self.destination.0
    }

    /// Resolves the reciprocal return fold across this meridian within the planetary topology.
    pub fn reciprocal_on<'a>(&self, topology: &'a super::PlanetaryTopology) -> &'a SeamFold {
        topology.seam(self.destination.0, self.destination.1)
    }

    /// Unfolds `(u, v, elevation)` from the source `QuadSphere` shell chart onto the destination shell chart.
    pub fn fold_point(&self, point: Vec3) -> Vec3 {
        let src_sphere = self.source.0.surface_frame(self.radius);
        let dst_sphere = self.destination.0.surface_frame(self.radius);
        let (u_src, v_src) = src_sphere.decompose(point);
        let elev = src_sphere.elevation_of(point);

        let src_planar = SurfaceFrame::new(src_sphere.tangent(), src_sphere.bitangent(), src_sphere.normal(), self.radius);
        let dst_planar = SurfaceFrame::new(dst_sphere.tangent(), dst_sphere.bitangent(), dst_sphere.normal(), self.radius);
        let hinge = (self.source.0.normal() + self.destination.0.normal()) * self.radius;
        let chart_pt = src_planar.point_at(u_src, v_src, 0.0);
        let folded_chart = hinge + self.rotation * (chart_pt - hinge);
        let (u_dst, v_dst) = dst_planar.decompose(folded_chart);
        dst_sphere.point_at(u_dst, v_dst, elev)
    }

    /// Folds a 3D position across the meridian, returning a living `FoldedPoint` entity.
    pub fn fold_step(&self, point: Vec3) -> FoldedPoint {
        FoldedPoint::new(self.destination.0, self.fold_point(point))
    }

    /// Folds a 3D chart direction vector across the 90° shell chart rotation.
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
