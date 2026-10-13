mod cavity;
mod corner;
mod elevation;
mod face;
mod facet;
mod palette;
mod strata;

#[cfg(test)]
mod tests;

pub use cavity::{CavitySpan, NullCavity, SubterraneanCavity, ZonedCavity};
pub use corner::ReliefCorner;
pub use elevation::ReliefElevation;
pub use face::FacetedReliefFace;
pub use facet::ReliefFacet;
pub use palette::ReliefPalette;
#[cfg(test)]
pub use elevation::ZonedRelief;
#[cfg(test)]
pub use strata::ReliefStrata;

