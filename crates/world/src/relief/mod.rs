mod cavity;
mod corner;
mod elevation;
mod face;
mod facet;
mod palette;
mod strata;
mod terraced;

#[cfg(test)]
mod corner_seam_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod watertight_tests;
#[cfg(test)]
mod wedge_seam_tests;

pub use cavity::{CavitySpan, NullCavity, SubterraneanCavity, ZonedCavity};
pub use corner::ReliefCorner;
pub use elevation::ReliefElevation;
pub use face::FacetedReliefFace;
pub use facet::ReliefFacet;
pub use palette::ReliefPalette;
pub use terraced::{SurfaceSightRay, TerracedRelief};
#[cfg(test)]
pub use elevation::ZonedRelief;
#[cfg(test)]
pub use strata::ReliefStrata;

