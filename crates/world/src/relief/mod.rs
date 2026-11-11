mod cantilever_rig;
mod column_clearance;
mod corner;
mod craft_cluster;
mod crust_vault;
mod elevation;
mod face;
mod facet;
mod molded_structure;
mod natural_strata;
mod overhang_molds;
mod palette;
mod shaft_excavation;
mod sight_ray;
mod strata;
mod terraced;
mod turf_scars;
mod vehicle_assembly;
mod wall;

#[cfg(test)]
mod cantilever_rig_tests;
#[cfg(test)]
mod corner_seam_tests;
#[cfg(test)]
mod soil_clod_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod watertight_tests;
#[cfg(test)]
mod wedge_seam_tests;

pub use crust_vault::CrustVault;
pub use elevation::ReliefElevation;
pub use face::FacetedReliefFace;
pub use facet::ReliefFacet;
pub use palette::ReliefPalette;
pub use sight_ray::SurfaceSightRay;
pub use terraced::TerracedRelief;
pub use vehicle_assembly::{VehicleAssembly, VehicleAxes, VehiclePart};
#[cfg(test)]
pub use elevation::ZonedRelief;
#[cfg(test)]
pub use strata::ReliefStrata;
