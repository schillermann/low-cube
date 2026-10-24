use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{BlockMedium, BlockMold, CleaveOutcome, Face, MeshTarget, SurfaceFind};
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

    fn canopy_drag(&self, point: Vec3) -> f32 {
        self.origin.canopy_drag(point)
    }

    fn excavate(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.origin.excavate(eye, direction)
    }

    fn surface_medium(&self, eye: Vec3, direction: Vec3) -> BlockMedium {
        self.origin.surface_medium(eye, direction)
    }

    fn loosen(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> bool {
        self.origin.loosen(eye, direction, stage)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        self.origin.strike_timber(eye, direction)
    }

    fn forage_timber(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        self.origin.forage_timber(eye, direction)
    }

    fn place_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.origin.place_block(eye, direction, avatar_pos)
    }

    fn place_molded_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        self.origin.place_molded_block(eye, direction, avatar_pos, mold)
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        self.origin.step_surface(dt)
    }

    fn weightless_at(&self, point: Vec3) -> bool {
        self.origin.weightless_at(point)
    }

    fn lethal_at(&self, point: Vec3) -> bool {
        self.origin.lethal_at(point)
    }

    fn confine_beneath_crust(&self, point: Vec3) -> Vec3 {
        self.origin.confine_beneath_crust(point)
    }
}
