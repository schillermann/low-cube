use crate::topology::ClimateZone;
use super::planting_ground::PlantingGround;

/// Decorator preserving an unobstructed avatar landing sanctuary at the planetary origin.
pub struct SpawnSanctuaryGround<'a> {
    origin: Box<dyn PlantingGround + 'a>,
    zone: ClimateZone,
    sanctuary_radius: f32,
}

impl<'a> SpawnSanctuaryGround<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        origin: Box<dyn PlantingGround + 'a>,
        zone: ClimateZone,
        sanctuary_radius: f32,
    ) -> Self {
        Self {
            origin,
            zone,
            sanctuary_radius,
        }
    }
}

impl<'a> PlantingGround for SpawnSanctuaryGround<'a> {
    fn receptive(&self, cell_u: i32, cell_v: i32, hash: i32) -> bool {
        if !self.origin.receptive(cell_u, cell_v, hash) {
            return false;
        }

        if self.zone == ClimateZone::Temperate {
            let u = (cell_u as f32) * 0.5 + 0.25;
            let v = (cell_v as f32) * 0.5 + 0.25;
            u.hypot(v) >= self.sanctuary_radius
        } else {
            true
        }
    }

    fn ground_elevation(&self, cell_u: i32, cell_v: i32) -> f32 {
        self.origin.ground_elevation(cell_u, cell_v)
    }
}
