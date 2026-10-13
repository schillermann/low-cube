mod caldera;
mod dunes;
mod glacial;
mod karst;
mod steppe;

#[cfg(test)]
mod tests;

pub use caldera::VolcanicCaldera;
pub use dunes::DesertDunes;
pub use glacial::GlacialMassif;
pub use karst::JungleKarst;
pub use steppe::SteppeTerrace;
