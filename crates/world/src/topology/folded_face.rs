use glam::Vec3;
use crate::frame::SurfaceFrame;
use super::{ClimateZone, FacePerimeter, PerimeterCrossing, PlanetaryTopology, SeamFold};

/// Focused perimeter coordinate folder translating points that cross a face edge onto the adjacent cube face.
#[derive(Clone, Copy)]
pub struct PerimeterFold {
    frame: SurfaceFrame,
    perimeter: FacePerimeter,
    seams: [SeamFold; 4],
}

impl PerimeterFold {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(frame: SurfaceFrame, perimeter: FacePerimeter, seams: [SeamFold; 4]) -> Self {
        Self { frame, perimeter, seams }
    }

    /// Secondary constructor assembling the four perimeter folds for a canonical climate zone.
    pub fn in_zone(zone: ClimateZone, half_size: f32) -> Self {
        let topology = PlanetaryTopology::cubic(half_size);
        let perimeter = FacePerimeter::square(half_size);
        let seams = topology.seams_for(zone);
        let frame = zone.surface_frame(half_size);
        Self::new(frame, perimeter, seams)
    }

    /// Folds `constrained` across the crossed edge whenever a near-crust `raw_point` crosses the face perimeter.
    pub fn fold(&self, raw_point: Vec3, constrained: Vec3) -> Vec3 {
        if self.frame.elevation_of(raw_point).abs() > 14.0 {
            return constrained;
        }
        match self.perimeter.crossing_point(&self.frame, raw_point) {
            PerimeterCrossing::Interior => constrained,
            PerimeterCrossing::Crossing { edge, .. } => self.seams[edge.index()].fold_point(constrained),
        }
    }
}
