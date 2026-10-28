use glam::Vec3;
use crate::topology::ClimateZone;

#[derive(Clone, Copy, Debug)]
struct FacetTones {
    surface: Vec3,
    chamfer: Vec3,
    cliff: Vec3,
}

#[derive(Clone, Copy, Debug)]
struct SlopeTones {
    ramp: Vec3,
    ridge: Vec3,
    valley: Vec3,
}

#[derive(Clone, Copy, Debug)]
struct SubterraneanTones {
    floor: Vec3,
    wall: Vec3,
    ceiling: Vec3,
}

/// Encapsulated geological color palette for surface and subterranean strata ("Tell, Don't Ask").
#[derive(Clone, Copy, Debug)]
pub struct ReliefPalette {
    facets: FacetTones,
    slopes: SlopeTones,
    cavern: SubterraneanTones,
}

impl ReliefPalette {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        surface: Vec3,
        subterranean_floor: Vec3,
        subterranean_wall: Vec3,
        subterranean_ceiling: Vec3,
        ramp: Vec3,
        ridge: Vec3,
        valley: Vec3,
        chamfer: Vec3,
        cliff: Vec3,
    ) -> Self {
        Self {
            facets: FacetTones { surface, chamfer, cliff },
            slopes: SlopeTones { ramp, ridge, valley },
            cavern: SubterraneanTones {
                floor: subterranean_floor,
                wall: subterranean_wall,
                ceiling: subterranean_ceiling,
            },
        }
    }

    /// Secondary constructor deriving uniform relief coloring for natural terrain biomes.
    pub fn uniform(
        surface: Vec3,
        subterranean_floor: Vec3,
        subterranean_wall: Vec3,
        subterranean_ceiling: Vec3,
    ) -> Self {
        Self::new(
            surface,
            subterranean_floor,
            subterranean_wall,
            subterranean_ceiling,
            surface,
            surface,
            surface,
            surface,
            subterranean_wall,
        )
    }

    /// Secondary constructor deriving authentic geological palette from planetary climate zone.
    pub fn for_zone(zone: ClimateZone) -> Self {
        match zone {
            ClimateZone::Temperate => Self::uniform(
                Vec3::new(0.28, 0.65, 0.25),
                Vec3::new(0.18, 0.18, 0.20),
                Vec3::new(0.22, 0.22, 0.25),
                Vec3::new(0.15, 0.15, 0.17),
            ),
            ClimateZone::Volcanic => Self::uniform(
                Vec3::new(0.35, 0.18, 0.18),
                Vec3::new(0.85, 0.25, 0.08),
                Vec3::new(0.25, 0.15, 0.15),
                Vec3::new(0.15, 0.10, 0.10),
            ),
            ClimateZone::Desert => Self::uniform(
                Vec3::new(0.85, 0.75, 0.35),
                Vec3::new(0.45, 0.38, 0.22),
                Vec3::new(0.55, 0.45, 0.28),
                Vec3::new(0.35, 0.28, 0.18),
            ),
            ClimateZone::Arctic => Self::uniform(
                Vec3::new(0.75, 0.88, 0.95),
                Vec3::new(0.25, 0.45, 0.65),
                Vec3::new(0.40, 0.60, 0.75),
                Vec3::new(0.30, 0.50, 0.68),
            ),
            ClimateZone::Jungle => Self::uniform(
                Vec3::new(0.12, 0.45, 0.22),
                Vec3::new(0.15, 0.25, 0.18),
                Vec3::new(0.20, 0.35, 0.22),
                Vec3::new(0.12, 0.20, 0.15),
            ),
            ClimateZone::Steppe => Self::uniform(
                Vec3::new(0.68, 0.48, 0.32),
                Vec3::new(0.32, 0.24, 0.18),
                Vec3::new(0.42, 0.32, 0.22),
                Vec3::new(0.28, 0.20, 0.15),
            ),
        }
    }

    /// Secondary constructor assembling vibrant Tron wireframe signature colors.
    pub fn tron() -> Self {
        Self::new(
            Vec3::new(0.0, 0.9, 1.0),
            Vec3::new(0.1, 0.2, 0.5),
            Vec3::new(1.0, 0.1, 0.1),
            Vec3::new(0.3, 0.1, 0.4),
            Vec3::new(1.0, 0.55, 0.0),
            Vec3::new(1.0, 0.0, 0.8),
            Vec3::new(1.0, 0.95, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 0.1, 0.1),
        )
    }

    /// Feeds surface color to consumer closure.
    pub fn inspect_surface<R>(&self, consumer: impl FnOnce(Vec3) -> R) -> R {
        consumer(self.facets.surface)
    }

    /// Feeds facet colors (surface, ramp, ridge, valley, chamfer, cliff) to consumer closure.
    pub fn inspect_facets<R>(&self, consumer: impl FnOnce(Vec3, Vec3, Vec3, Vec3, Vec3, Vec3) -> R) -> R {
        consumer(
            self.facets.surface,
            self.slopes.ramp,
            self.slopes.ridge,
            self.slopes.valley,
            self.facets.chamfer,
            self.facets.cliff,
        )
    }

    /// Feeds subterranean cavern colors to consumer closure.
    pub fn inspect_subterranean<R>(&self, consumer: impl FnOnce(Vec3, Vec3, Vec3) -> R) -> R {
        consumer(self.cavern.floor, self.cavern.wall, self.cavern.ceiling)
    }
}
