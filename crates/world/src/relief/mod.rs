mod cantilever_rig;
mod clipmap;
mod column_clearance;
mod corner;
mod craft_cluster;
mod craft_entity;
mod crust_vault;
mod elevation;
mod face;
mod facet;
mod mineral_vein;
mod molded_structure;
mod natural_strata;
mod overhang_molds;
mod palette;
mod planned_mount;
mod shaft_excavation;
mod sight_ray;
mod strata;
mod terraced;
mod turf_habitat;
mod turf_scars;
mod vehicle_assembly;
mod wall;
mod zoned;

#[cfg(test)]
mod cantilever_rig_tests;
#[cfg(test)]
mod corner_seam_tests;
#[cfg(test)]
mod mineral_vein_tests;
#[cfg(test)]
mod shaft_story_tests;
#[cfg(test)]
mod soil_clod_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod watertight_tests;
#[cfg(test)]
mod wedge_seam_tests;

pub(crate) use cantilever_rig::{BlueprintHoist, RigArm};
pub(crate) use corner::NaturalCrown;
pub(crate) use craft_cluster::CraftCluster;
pub(crate) use craft_entity::PosedHull;
pub(crate) use molded_structure::{MoldedStructure, SanctuaryPlinth, SectorCanvas, SectorVault};
pub(crate) use natural_strata::NaturalStrata;
pub(crate) use shaft_excavation::ShaftExcavation;
pub(crate) use turf_habitat::{CanopyHabitat, NaturalHabitat, TurfHabitat};
pub(crate) use turf_scars::TurfScars;
pub use craft_entity::{
    BlueprintCatalog, CellRole, CraftBlueprint, CraftIdentity, CraftPose, DeckFrame, HelmLocks,
    HullOptic, HullWireframe, JetTone, OpticMount, SeatOccupancy, VesselSink,
    MAX_EXHAUST_JETS, MAX_WIRE_EDGES,
};
pub use crust_vault::CrustVault;
pub use clipmap::{ClipmapLod, PlanetHeightClipmap};
pub use elevation::ReliefElevation;
pub use face::{FacetedReliefFace, MeadowRing, MeadowStamp, MEADOW_RING_SLOTS};
pub use facet::ReliefFacet;
pub use mineral_vein::{ExposedStratum, HollowVeins, LivingVeins, SubterraneanVeins};
pub use palette::ReliefPalette;
pub use planned_mount::{MountDomain, MountSite, PlannedMount};
pub use sight_ray::SurfaceSightRay;
pub use terraced::TerracedRelief;
pub use vehicle_assembly::{
    AttitudeGuidance, CockpitAxes, PilotOrder, RigidImpulse, VehicleAssembly, VehicleAxes,
    VehiclePart,
};
#[cfg(test)]
pub use elevation::ZonedRelief;
#[cfg(test)]
pub use strata::ReliefStrata;
