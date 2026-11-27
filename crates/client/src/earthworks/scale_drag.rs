use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{BeltSatchel, EvaAttire};
use super::reach::SurfaceReach;

/// Polymorphic state for interactive `B × T × H` drag-scaling of scalable modules (`0` `Option` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScaleDrag {
    Idle,
    Stretching {
        side: f32,
        anchor_eye: Vec3,
        anchor_dir: Vec3,
        extra_height: i32,
    },
}

impl ScaleDrag {
    pub const fn active(self) -> bool {
        matches!(self, Self::Stretching { .. })
    }

    pub const fn begin(side: f32, reach: SurfaceReach) -> Self {
        Self::Stretching {
            side,
            anchor_eye: reach.eye(),
            anchor_dir: reach.view_dir(),
            extra_height: 0,
        }
    }

    pub fn step_height(self, delta: i32) -> (Self, bool) {
        match self {
            Self::Idle => (Self::Idle, false),
            Self::Stretching { side, anchor_eye, anchor_dir, extra_height } => {
                let next_h = (extra_height + delta).clamp(0, 15);
                (
                    Self::Stretching { side, anchor_eye, anchor_dir, extra_height: next_h },
                    true,
                )
            }
        }
    }

    pub fn emit_preview(
        self,
        reach: SurfaceReach,
        satchel: &BeltSatchel,
        attire: EvaAttire,
        world: &dyn PlanetaryWorld,
        target: &mut dyn MeshTarget,
    ) -> bool {
        let Self::Stretching { side, anchor_eye, anchor_dir, extra_height } = self else {
            return false;
        };
        let hand = attire.hand_pair(side).0;
        if !hand.scalable_module() {
            return false;
        }
        let mold = satchel.mold_on_side(side).in_medium(hand.block_medium());
        let Some(start) = world.plan_molded_block(anchor_eye, anchor_dir, reach.stance_origin(), mold) else {
            return true;
        };
        let end = world
            .plan_molded_block(reach.eye(), reach.view_dir(), reach.stance_origin(), mold)
            .unwrap_or(start);
        start.emit_drag_hologram(end, extra_height, target);
        true
    }

    pub fn commit(
        self,
        reach: SurfaceReach,
        satchel: BeltSatchel,
        attire: EvaAttire,
        world: &mut dyn PlanetaryWorld,
    ) -> Option<(BeltSatchel, EvaAttire, bool)> {
        let Self::Stretching { side, anchor_eye, anchor_dir, extra_height } = self else {
            return None;
        };
        let hand = attire.hand_pair(side).0;
        if !hand.scalable_module() {
            return Some((satchel, attire, false));
        }
        let mold = satchel.mold_on_side(side).in_medium(hand.block_medium());
        let Some(start) = world.plan_molded_block(anchor_eye, anchor_dir, reach.stance_origin(), mold) else {
            return Some((satchel, attire, false));
        };
        let end = world
            .plan_molded_block(reach.eye(), reach.view_dir(), reach.stance_origin(), mold)
            .unwrap_or(start);
        let Some((place_eye, place_dir, spec)) = start.resolve_dragged_module(end, extra_height) else {
            return Some((satchel, attire, false));
        };
        let (ns, na, placed) =
            satchel.place_scaled_from_hand(side, attire, place_eye, place_dir, reach.stance_origin(), spec, world);
        Some((ns, na, placed))
    }
}
