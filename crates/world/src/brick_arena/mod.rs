mod arena;
mod atom;
mod atom_shape;
mod brick;
mod colonize;
mod coord;
mod inertia;
mod module;
mod module_mesh;
mod module_spec;
mod niche;
mod structure;
mod sweep;

#[cfg(test)]
mod tests;

pub use self::arena::BrickArena;
pub use self::atom::{AtomKind, GranularFlag, RawAtom};
pub use self::atom_shape::AtomSurface;
pub use self::brick::{Brick8, BrickCell};
pub use self::coord::{AnchorOffset, BrickCoord, CellCoord, LocalSlot};
pub use self::inertia::{BrickVolume, DismantledSpan, InertialBricks, SteinerMass, SymmetricInertia};
pub use self::module::{MachineKind, ScalableKind, SpaceModule, MAX_MODULE_AXIS_CELLS};
pub use self::module_mesh::{ModuleSculpture, SpanHologram};
pub use self::module_spec::{ModuleSpan, ModuleSpec};
pub use self::niche::{NicheConfinement, NICHE_CONTACT_TOLERANCE};
pub use self::structure::BrickStructure;
