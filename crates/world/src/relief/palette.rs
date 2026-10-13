use glam::Vec3;
use crate::topology::ClimateZone;

/// Encapsulated geological color palette for surface and subterranean strata ("Tell, Don't Ask").
#[derive(Clone, Copy, Debug)]
pub struct ReliefPalette {
    surface: Vec3,
    subterranean_floor: Vec3,
    subterranean_wall: Vec3,
    subterranean_ceiling: Vec3,
}

impl ReliefPalette {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        surface: Vec3,
        subterranean_floor: Vec3,
        subterranean_wall: Vec3,
        subterranean_ceiling: Vec3,
    ) -> Self {
        Self {
            surface,
            subterranean_floor,
            subterranean_wall,
            subterranean_ceiling,
        }
    }

    /// Secondary constructor deriving authentic geological palette from planetary climate zone.
    pub fn for_zone(zone: ClimateZone) -> Self {
        match zone {
            ClimateZone::Temperate => Self::new(
                Vec3::new(0.28, 0.65, 0.25),
                Vec3::new(0.18, 0.18, 0.20),
                Vec3::new(0.22, 0.22, 0.25),
                Vec3::new(0.15, 0.15, 0.17),
            ),
            ClimateZone::Volcanic => Self::new(
                Vec3::new(0.35, 0.18, 0.18),
                Vec3::new(0.85, 0.25, 0.08),
                Vec3::new(0.25, 0.15, 0.15),
                Vec3::new(0.15, 0.10, 0.10),
            ),
            ClimateZone::Desert => Self::new(
                Vec3::new(0.85, 0.75, 0.35),
                Vec3::new(0.45, 0.38, 0.22),
                Vec3::new(0.55, 0.45, 0.28),
                Vec3::new(0.35, 0.28, 0.18),
            ),
            ClimateZone::Arctic => Self::new(
                Vec3::new(0.75, 0.88, 0.95),
                Vec3::new(0.25, 0.45, 0.65),
                Vec3::new(0.40, 0.60, 0.75),
                Vec3::new(0.30, 0.50, 0.68),
            ),
            ClimateZone::Jungle => Self::new(
                Vec3::new(0.12, 0.45, 0.22),
                Vec3::new(0.15, 0.25, 0.18),
                Vec3::new(0.20, 0.35, 0.22),
                Vec3::new(0.12, 0.20, 0.15),
            ),
            ClimateZone::Steppe => Self::new(
                Vec3::new(0.68, 0.48, 0.32),
                Vec3::new(0.32, 0.24, 0.18),
                Vec3::new(0.42, 0.32, 0.22),
                Vec3::new(0.28, 0.20, 0.15),
            ),
        }
    }

    /// Feeds surface color to consumer closure.
    pub fn inspect_surface<R>(&self, consumer: impl FnOnce(Vec3) -> R) -> R {
        consumer(self.surface)
    }

    /// Feeds subterranean cavern colors to consumer closure.
    pub fn inspect_subterranean<R>(&self, consumer: impl FnOnce(Vec3, Vec3, Vec3) -> R) -> R {
        consumer(self.subterranean_floor, self.subterranean_wall, self.subterranean_ceiling)
    }
}
