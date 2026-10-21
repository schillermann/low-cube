use glam::Vec3;
use world::PlanetaryWorld;
use crate::attire::{HeldRelic, StrideRhythm, YouthAttire};

const EXCAVATION_THRESHOLD: u32 = 4;

/// Immutable per-hand block inventory and excavation progress state ("Tell, Don't Ask").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PalmEarthworks {
    right_blocks: u32,
    left_blocks: u32,
    dig_progress: u32,
}

impl PalmEarthworks {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(right_blocks: u32, left_blocks: u32, dig_progress: u32) -> Self {
        Self {
            right_blocks,
            left_blocks,
            dig_progress,
        }
    }

    /// Secondary constructor initializing empty hands with zero digging progress.
    pub const fn empty() -> Self {
        Self::new(0, 0, 0)
    }

    /// Synchronizes block counts when attire relics are cycled or loaded.
    pub fn reconcile_attire(self, attire: YouthAttire) -> Self {
        let (right, left) = attire.held_pair();
        let next_right = if right.placeable_block() { self.right_blocks.max(1) } else { 0 };
        let next_left = if left.placeable_block() { self.left_blocks.max(1) } else { 0 };
        Self::new(next_right, next_left, self.dig_progress)
    }

    /// Preserves held earth blocks in free hands when ground weapon sites synchronize.
    pub fn merge_site_relics(
        self,
        attire: YouthAttire,
        site_pair: (HeldRelic, HeldRelic),
    ) -> (Self, YouthAttire) {
        let right = if site_pair.0.free_palm() && self.right_blocks > 0 {
            HeldRelic::ExcavatedEarth
        } else {
            site_pair.0
        };
        let left = if site_pair.1.free_palm() && self.left_blocks > 0 {
            HeldRelic::ExcavatedEarth
        } else {
            site_pair.1
        };
        let updated = attire.with_relics(right, left);
        (self.reconcile_attire(updated), updated)
    }

    /// Triggers hand swing animation according to free palms, tools, or weapons.
    pub fn trigger_hand_swing(attire: YouthAttire, rhythm: StrideRhythm, side: f32) -> StrideRhythm {
        let (right, left) = attire.held_pair();
        if right.free_palm() && left.free_palm() {
            return rhythm.trigger_right_strike().trigger_left_strike();
        }
        let active = if side > 0.0 { right } else { left };
        let other = if side > 0.0 { left } else { right };
        if active.gripped() || active.free_palm() {
            if side > 0.0 {
                rhythm.trigger_right_strike()
            } else {
                rhythm.trigger_left_strike()
            }
        } else if other.free_palm() {
            if side > 0.0 {
                rhythm.trigger_left_strike()
            } else {
                rhythm.trigger_right_strike()
            }
        } else {
            rhythm
        }
    }

    /// Executes block placement if the chosen hand holds a block, or digs earth with hands/shovel.
    #[allow(clippy::too_many_arguments)]
    pub fn act_with_hand(
        self,
        side: f32,
        attire: YouthAttire,
        rhythm: StrideRhythm,
        eye: Vec3,
        view_dir: Vec3,
        avatar_pos: Vec3,
        world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        let state = self.reconcile_attire(attire);
        let (right, left) = attire.held_pair();
        let active = if side > 0.0 { right } else { left };

        if active.placeable_block() {
            let next_rhythm = if side > 0.0 {
                rhythm.trigger_right_strike()
            } else {
                rhythm.trigger_left_strike()
            };
            if world.place_surface_block(eye, view_dir, avatar_pos) {
                let (next_state, next_attire) = state.consume_placed_block(side, attire);
                return (next_state, next_attire, next_rhythm, true);
            }
            return (state, attire, next_rhythm, false);
        }

        let (effort, recipient_side, next_rhythm) = if right.shovel_blade() || left.shovel_blade() {
            let shovel_side = if right.shovel_blade() { 1.0 } else { -1.0 };
            let target_side = if shovel_side > 0.0 && (left.free_palm() || left.placeable_block()) {
                -1.0
            } else if shovel_side < 0.0 && (right.free_palm() || right.placeable_block()) {
                1.0
            } else {
                shovel_side
            };
            let swing = if shovel_side > 0.0 {
                rhythm.trigger_right_strike()
            } else {
                rhythm.trigger_left_strike()
            };
            (EXCAVATION_THRESHOLD, target_side, swing)
        } else if right.free_palm() && left.free_palm() {
            (
                2,
                side,
                rhythm.trigger_right_strike().trigger_left_strike(),
            )
        } else if right.free_palm() || left.free_palm() {
            let free_side = if right.free_palm() { 1.0 } else { -1.0 };
            let swing = if free_side > 0.0 {
                rhythm.trigger_right_strike()
            } else {
                rhythm.trigger_left_strike()
            };
            (1, free_side, swing)
        } else {
            let swing = Self::trigger_hand_swing(attire, rhythm, side);
            return (state, attire, swing, false);
        };

        let progress = state.dig_progress + effort;
        if progress >= EXCAVATION_THRESHOLD {
            if world.excavate_surface(eye, view_dir) {
                let (next_state, next_attire) = state.collect_excavated_block(recipient_side, attire);
                return (next_state, next_attire, next_rhythm, true);
            }
            return (Self::new(state.right_blocks, state.left_blocks, 0), attire, next_rhythm, false);
        }

        (
            Self::new(state.right_blocks, state.left_blocks, progress),
            attire,
            next_rhythm,
            false,
        )
    }

    fn collect_excavated_block(self, side: f32, attire: YouthAttire) -> (Self, YouthAttire) {
        let (right, left) = attire.held_pair();
        if side > 0.0 {
            let next_right = self.right_blocks + 1;
            let next_attire = attire.with_relics(HeldRelic::ExcavatedEarth, left);
            (Self::new(next_right, self.left_blocks, 0), next_attire)
        } else {
            let next_left = self.left_blocks + 1;
            let next_attire = attire.with_relics(right, HeldRelic::ExcavatedEarth);
            (Self::new(self.right_blocks, next_left, 0), next_attire)
        }
    }

    fn consume_placed_block(self, side: f32, attire: YouthAttire) -> (Self, YouthAttire) {
        let (right, left) = attire.held_pair();
        if side > 0.0 {
            let next_right = self.right_blocks.saturating_sub(1);
            let next_relic = if next_right == 0 { HeldRelic::EmptyPalm } else { right };
            (
                Self::new(next_right, self.left_blocks, self.dig_progress),
                attire.with_relics(next_relic, left),
            )
        } else {
            let next_left = self.left_blocks.saturating_sub(1);
            let next_offhand = if next_left == 0 { HeldRelic::EmptyPalm } else { left };
            (
                Self::new(self.right_blocks, next_left, self.dig_progress),
                attire.with_relics(right, next_offhand),
            )
        }
    }
}
