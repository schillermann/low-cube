use super::planting_ground::PlantingGround;

/// Decorator preserving open cardinal transit corridors across planetary faces.
pub struct MeridianGround<'a> {
    origin: Box<dyn PlantingGround + 'a>,
    corridor_clearance: f32,
}

impl<'a> MeridianGround<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn PlantingGround + 'a>, corridor_clearance: f32) -> Self {
        Self {
            origin,
            corridor_clearance,
        }
    }
}

impl<'a> PlantingGround for MeridianGround<'a> {
    fn receptive(&self, cell_u: i32, cell_v: i32, hash: i32) -> bool {
        if !self.origin.receptive(cell_u, cell_v, hash) {
            return false;
        }

        let u = (cell_u as f32) + 0.5;
        let v = (cell_v as f32) + 0.5;
        u.abs() >= self.corridor_clearance && v.abs() >= self.corridor_clearance
    }

    fn ground_elevation(&self, cell_u: i32, cell_v: i32) -> f32 {
        self.origin.ground_elevation(cell_u, cell_v)
    }
}
