use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{Face, MeshTarget};
use super::{ClimateZone, FacePerimeter, PerimeterCrossing, PlanetaryTopology, SeamFold};

/// Decorator enabling seamless 90° coordinate folding across outer perimeter boundaries.
pub struct FoldedFace {
    origin: Box<dyn Face>,
    frame: SurfaceFrame,
    perimeter: FacePerimeter,
    seams: [SeamFold; 4],
}

impl FoldedFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        origin: Box<dyn Face>,
        frame: SurfaceFrame,
        perimeter: FacePerimeter,
        seams: [SeamFold; 4],
    ) -> Self {
        Self {
            origin,
            frame,
            perimeter,
            seams,
        }
    }

    /// Secondary constructor decorating an existing face for a canonical climate zone.
    pub fn in_zone(origin: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> Self {
        let topology = PlanetaryTopology::cubic(half_size);
        let perimeter = FacePerimeter::square(half_size);
        let seams = topology.seams_for(zone);
        let frame = zone.surface_frame(half_size);
        Self::new(origin, frame, perimeter, seams)
    }

    /// Backwards-compatible alias for in_zone.
    pub fn on_cube(origin: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> Self {
        Self::in_zone(origin, zone, half_size)
    }
}

impl Face for FoldedFace {
    fn matches(&self, point: Vec3) -> bool {
        self.origin.matches(point)
    }

    fn surface_up(&self) -> Vec3 {
        self.origin.surface_up()
    }

    fn pull_direction(&self) -> Vec3 {
        self.origin.pull_direction()
    }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        let constrained = self.origin.constrain(point, distance);
        let crossing = self.perimeter.crossing_point(&self.frame, point);
        match crossing {
            PerimeterCrossing::Interior => constrained,
            PerimeterCrossing::Crossing { edge, .. } => {
                self.seams[edge.index()].fold_point(constrained)
            }
        }
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_surface(target);
    }
}
