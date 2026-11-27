mod arm;
mod blueprint_mount;
mod mesh;
mod registry;

use crate::block::GridCell;
pub use self::arm::RigArm;
pub(crate) use self::blueprint_mount::BlueprintHoist;
pub use self::registry::{RigPlacement, RigRegistry};

pub(super) const BLOCK_STEP: f32 = GridCell::SPAN;
pub(super) const MAX_HOIST_STEPS: i32 = 8;
