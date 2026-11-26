use glam::Vec3;
use crate::{BlockMedium, PlanetaryWorld};

/// Polymorphic planetary arrival-site collection contract ("Tell, Don't Ask").
pub trait ArrivalSites {
    fn collect_sites<W: PlanetaryWorld + ?Sized>(&self, world: &W, out: &mut Vec<(Vec3, Vec3)>);
}

/// Concrete `ArrivalSites` querying placed blocks of a specific `BlockMedium` (`1` field).
#[derive(Clone, Copy, Debug)]
pub struct MediumArrivalSites {
    medium: BlockMedium,
}

impl MediumArrivalSites {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(medium: BlockMedium) -> Self {
        Self { medium }
    }

    pub const fn admin_plinths() -> Self {
        Self::new(BlockMedium::SpawnStone)
    }

    pub const fn origin_plinth() -> Self {
        Self::new(BlockMedium::OriginStone)
    }
}

impl ArrivalSites for MediumArrivalSites {
    fn collect_sites<W: PlanetaryWorld + ?Sized>(&self, world: &W, out: &mut Vec<(Vec3, Vec3)>) {
        world.collect_medium_sites(self.medium, out);
    }
}

/// Decorator preferring `preferred` arrival sites and consulting `fallback` only when none exist (`2` fields).
#[derive(Clone, Copy, Debug)]
pub struct PrioritizedArrivalSites<P, F> {
    preferred: P,
    fallback: F,
}

impl<P, F> PrioritizedArrivalSites<P, F> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(preferred: P, fallback: F) -> Self {
        Self { preferred, fallback }
    }
}

impl PrioritizedArrivalSites<MediumArrivalSites, MediumArrivalSites> {
    /// Sanctuary arrival chain preferring admin `SpawnStone` plinths over the world's `OriginStone`.
    pub const fn sanctuary() -> Self {
        Self::new(MediumArrivalSites::admin_plinths(), MediumArrivalSites::origin_plinth())
    }
}

impl<P: ArrivalSites, F: ArrivalSites> ArrivalSites for PrioritizedArrivalSites<P, F> {
    fn collect_sites<W: PlanetaryWorld + ?Sized>(&self, world: &W, out: &mut Vec<(Vec3, Vec3)>) {
        let mark = out.len();
        self.preferred.collect_sites(world, out);
        if out.len() == mark {
            self.fallback.collect_sites(world, out);
        }
    }
}
