mod cavity;
mod corner;
mod crust_vault;
mod elevation;
mod face;
mod facet;
mod magma_core;
mod palette;
mod sight_ray;
mod strata;
mod terraced;
mod wall;

#[cfg(test)]
mod corner_seam_tests;
#[cfg(test)]
mod hollow_world_tests;
#[cfg(test)]
mod soil_clod_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod watertight_tests;
#[cfg(test)]
mod wedge_seam_tests;

pub use cavity::{CavitySpan, NullCavity, SubterraneanCavity, ZonedCavity};
pub use corner::ReliefCorner;
pub use crust_vault::CrustVault;
pub use elevation::ReliefElevation;
pub use face::FacetedReliefFace;
pub use facet::ReliefFacet;
pub use palette::ReliefPalette;
pub use sight_ray::SurfaceSightRay;
pub use terraced::TerracedRelief;
#[cfg(test)]
pub use elevation::ZonedRelief;
#[cfg(test)]
pub use strata::ReliefStrata;

