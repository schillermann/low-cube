use crate::NATURAL_OVERHANG_STAMP;
use crate::block::{
    BlockMedium, BlockMold, CornerHeading, GridCell, MoldArchetype, TurfSod, WedgeHeading,
};
use crate::frame::SurfaceFrame;
use crate::relief::NaturalCrown;
use super::atom_shape::AtomSurface;

/// Granular settling behavior flag stored directly in `RawAtom` without heap indirection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GranularFlag {
    Rigid,
    Granular,
}

impl GranularFlag {
    pub const fn for_medium(medium: BlockMedium) -> Self {
        if medium.settles_under_gravity() {
            Self::Granular
        } else {
            Self::Rigid
        }
    }

    pub const fn granular(self) -> bool {
        matches!(self, Self::Granular)
    }
}

/// Unboxed 3D mold or natural crown orientation of a `0.5 m` `RawAtom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AtomKind {
    Molded {
        archetype: MoldArchetype,
        heading: WedgeHeading,
        corner: CornerHeading,
    },
    Crown(NaturalCrown),
}

impl AtomKind {
    pub const fn molded(archetype: MoldArchetype, heading: WedgeHeading) -> Self {
        Self::Molded {
            archetype,
            heading,
            corner: BlockMold::corner_for(heading),
        }
    }

    pub fn rotated_quarter(self) -> Self {
        match self {
            Self::Molded { archetype, heading, corner } => Self::Molded {
                archetype,
                heading: heading.rotated_ccw(),
                corner: corner.rotated_ccw(),
            },
            Self::Crown(crown) => Self::Crown(crown.rotated_quarter()),
        }
    }
}

/// Unboxed `0.5 m` voxel atom (`4` private fields, `Copy`, `0` heap pointers).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RawAtom {
    medium: BlockMedium,
    kind: AtomKind,
    granular: GranularFlag,
    sod: TurfSod,
}

impl RawAtom {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        medium: BlockMedium,
        kind: AtomKind,
        granular: GranularFlag,
        sod: TurfSod,
    ) -> Self {
        Self { medium, kind, granular, sod }
    }

    pub const fn full_cube(medium: BlockMedium) -> Self {
        Self::molded(medium, MoldArchetype::FullCube, WedgeHeading::South)
    }

    pub const fn cube_of(medium: BlockMedium) -> Self {
        Self::full_cube(medium)
    }

    pub const fn turf_cube() -> Self {
        Self::full_cube(BlockMedium::TurfEarth)
    }

    pub const fn molded(medium: BlockMedium, archetype: MoldArchetype, heading: WedgeHeading) -> Self {
        Self::new(
            medium,
            AtomKind::molded(archetype, heading),
            GranularFlag::for_medium(medium),
            TurfSod::bare(),
        )
    }

    pub fn from_mold(mold: BlockMold, direction: glam::Vec3, frame: &SurfaceFrame) -> Self {
        let (arch, wedge, corner) = mold.resolve_atom_orientation(direction, frame);
        let medium = mold.medium();
        Self::new(
            medium,
            AtomKind::Molded { archetype: arch, heading: wedge, corner },
            GranularFlag::for_medium(medium),
            TurfSod::bare(),
        )
    }

    pub const fn from_stamp(stamp: [u8; 3], verdant: bool) -> Self {
        let medium = BlockMedium::from_code(stamp[2]);
        let arch = MoldArchetype::from_code(stamp[0]);
        let heading = WedgeHeading::from_code(stamp[1]);
        let sod = if verdant && matches!(medium, BlockMedium::TurfEarth) {
            TurfSod::verdant()
        } else {
            TurfSod::bare()
        };
        Self::new(medium, AtomKind::molded(arch, heading), GranularFlag::for_medium(medium), sod)
    }

    pub const fn natural_crown(crown: NaturalCrown, verdant: bool) -> Self {
        let sod = if verdant { TurfSod::verdant() } else { TurfSod::bare() };
        Self::new(BlockMedium::TurfEarth, AtomKind::Crown(crown), GranularFlag::Rigid, sod)
    }

    pub const fn at_floor(self, floor_elev: f32) -> AtomSurface {
        AtomSurface::new(self, floor_elev)
    }

    pub const fn medium(self) -> BlockMedium { self.medium }
    pub const fn granular(self) -> GranularFlag { self.granular }
    pub const fn kind(self) -> AtomKind { self.kind }
    pub const fn settles_under_gravity(self) -> bool { self.granular.granular() }

    pub const fn heading(self) -> WedgeHeading {
        match self.kind {
            AtomKind::Molded { heading, .. } => heading,
            AtomKind::Crown(_) => WedgeHeading::South,
        }
    }

    pub const fn stamp(self) -> [u8; 3] {
        match self.kind {
            AtomKind::Molded { archetype, heading, .. } => {
                [archetype.code(), heading.code(), self.medium.code()]
            }
            AtomKind::Crown(_) => [NATURAL_OVERHANG_STAMP, 0, self.medium.code()],
        }
    }

    pub const fn fertile_soil(self) -> bool {
        matches!(self.medium, BlockMedium::TurfEarth)
    }

    pub const fn verdant_turf(self) -> bool {
        self.fertile_soil() && self.sod.verdant_coat()
    }

    pub const fn bare_turf(self) -> bool {
        self.fertile_soil() && !self.sod.verdant_coat()
    }

    pub const fn bare_surface(self, ground_bare: bool) -> bool {
        if self.fertile_soil() { !self.sod.verdant_coat() } else { ground_bare }
    }

    pub const fn smothered(mut self) -> Self {
        if self.fertile_soil() {
            self.sod = TurfSod::bare();
        }
        self
    }

    pub fn colonized(mut self, cell: GridCell, dt: f32, grass_neighbors: u32) -> (Self, bool) {
        if !self.fertile_soil() {
            return (self, false);
        }
        let (next, grown) = self.sod.colonized(cell, dt, grass_neighbors);
        self.sod = next;
        (self, grown)
    }

    pub fn rotated_quarter(mut self) -> Self {
        self.kind = self.kind.rotated_quarter();
        self
    }

    pub fn step_camera_turn(&mut self, delta: i32) -> bool {
        if self.medium != BlockMedium::FieldCamera {
            return false;
        }
        if let AtomKind::Molded { archetype, heading, corner } = self.kind {
            let next_arch = MoldArchetype::from_code((archetype.code() as i32 + delta).rem_euclid(8) as u8);
            self.kind = AtomKind::Molded { archetype: next_arch, heading, corner };
            true
        } else {
            false
        }
    }

    /// Physical mass in kilograms of this `0.5 m × 0.5 m × 0.5 m` (`0.125 m³`) atom.
    pub const fn mass_kg(self) -> f64 {
        let base = match self.medium {
            BlockMedium::GoldOre => 420.0,
            BlockMedium::IronOre | BlockMedium::TitaniumHull => 260.0,
            BlockMedium::CopperOre | BlockMedium::VitrifiedBasalt => 220.0,
            BlockMedium::CrustRock | BlockMedium::QuarriedStone | BlockMedium::OriginStone | BlockMedium::SpawnStone => 190.0,
            BlockMedium::CarbonSilicate => 135.0,
            BlockMedium::TurfEarth | BlockMedium::Sand => 145.0,
            BlockMedium::FusionTorch => 310.0,
            BlockMedium::RocketNozzle | BlockMedium::AircraftEngine => 170.0,
            _ => 85.0,
        };
        match self.kind {
            AtomKind::Molded { archetype: MoldArchetype::HalfSlab, .. } => base * 0.5,
            AtomKind::Molded { archetype, .. } if !archetype.full_height_flat() => base * 0.65,
            _ => base,
        }
    }
}
