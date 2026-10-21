use crate::Face;
use crate::frame::SurfaceFrame;
use super::planting_ground::PlantingGround;

/// Decorator enforcing that trees are only planted on flat, unsloped Full-Cube blocks.
pub struct FullCubeGround<'a> {
    origin: Box<dyn PlantingGround + 'a>,
    face: &'a dyn Face,
    frame: &'a SurfaceFrame,
    half_size: f32,
}

impl<'a> FullCubeGround<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        origin: Box<dyn PlantingGround + 'a>,
        face: &'a dyn Face,
        frame: &'a SurfaceFrame,
        half_size: f32,
    ) -> Self {
        Self {
            origin,
            face,
            frame,
            half_size,
        }
    }

    fn flat_cube(&self, cell_u: i32, cell_v: i32) -> bool {
        let u0 = (cell_u as f32) * 0.5;
        let v0 = (cell_v as f32) * 0.5;
        let probes = [
            (u0 + 0.075, v0 + 0.075),
            (u0 + 0.425, v0 + 0.075),
            (u0 + 0.075, v0 + 0.425),
            (u0 + 0.425, v0 + 0.425),
            (u0 + 0.25, v0 + 0.25),
        ];

        let mut base_elev = 0.0;
        for (i, &(u, v)) in probes.iter().enumerate() {
            let probe_pt = self.frame.point_at(u, v, 0.0);
            let surface_pt = self.face.constrain(probe_pt, 0.0);
            let elev = self.frame.height_of(surface_pt) - self.half_size;

            if i == 0 {
                let half_blocks = elev * 2.0;
                if elev < 0.0 || (half_blocks - half_blocks.round()).abs() > 0.04 {
                    return false;
                }
                base_elev = elev;
            } else if (elev - base_elev).abs() > 0.02 {
                return false;
            }
        }
        true
    }
}

impl<'a> PlantingGround for FullCubeGround<'a> {
    fn receptive(&self, cell_u: i32, cell_v: i32, hash: i32) -> bool {
        if !self.origin.receptive(cell_u, cell_v, hash) {
            return false;
        }
        self.flat_cube(cell_u, cell_v)
    }

    fn ground_elevation(&self, cell_u: i32, cell_v: i32) -> f32 {
        self.origin.ground_elevation(cell_u, cell_v)
    }
}
