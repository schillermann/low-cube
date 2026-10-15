use crate::Face;
use crate::frame::SurfaceFrame;
use super::planting_ground::PlantingGround;

/// Base ground resolving surface elevation from an underlying planetary face.
pub struct SurfaceGround<'a> {
    origin: &'a dyn Face,
    frame: &'a SurfaceFrame,
    half_size: f32,
}

impl<'a> SurfaceGround<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: &'a dyn Face, frame: &'a SurfaceFrame, half_size: f32) -> Self {
        Self {
            origin,
            frame,
            half_size,
        }
    }
}

impl<'a> PlantingGround for SurfaceGround<'a> {
    fn receptive(&self, _cell_u: i32, _cell_v: i32, _hash: i32) -> bool {
        true
    }

    fn ground_elevation(&self, cell_u: i32, cell_v: i32) -> f32 {
        let probe = self.frame.point_at((cell_u as f32) + 0.5, (cell_v as f32) + 0.5, 0.0);
        let surface_point = self.origin.constrain(probe, 0.0);
        self.frame.height_of(surface_point) - self.half_size
    }
}
