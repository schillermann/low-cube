use glam::Vec3;
use world::{ModuleSpec, PlanetaryWorld};
use super::super::relic::HeldRelic;
use super::super::tray::SatchelTray;
use super::super::EvaAttire;
use super::palms::PalmStack;
use super::temper::SatchelTemper;

/// Collaborator handling palm/tray slot swaps and scaled multi-cell module consumption (`3` fields).
pub(super) struct SatchelTransfer {
    palms: PalmStack,
    tray: SatchelTray,
    temper: SatchelTemper,
}

impl SatchelTransfer {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(palms: PalmStack, tray: SatchelTray, temper: SatchelTemper) -> Self {
        Self { palms, tray, temper }
    }

    pub(super) fn transfer_hand(
        mut self,
        side: f32,
        attire: EvaAttire,
    ) -> (PalmStack, SatchelTray, SatchelTemper, EvaAttire, bool) {
        self.palms.reconcile_attire(attire);
        self.temper.reconcile_attire(attire);
        let (hand, other) = attire.hand_pair(side);
        if self.tray.open() {
            let (chosen, chosen_rank) = self.tray.selected_slot();
            if chosen.gripped() {
                let drawn_wear = self.temper.take_stowed_wear(chosen, chosen_rank);
                let (mut next_tray, _) = self.tray.taken_stowed(chosen);
                if hand.gripped() {
                    self.temper.push_stowed_wear(hand, self.temper.palm_wear(side));
                    next_tray = next_tray.stowed_with(hand, self.palms.blocks_on(side));
                }
                self.temper.assign_palm_wear(side, drawn_wear);
                if chosen.placeable_block()
                    && other.placeable_block()
                    && !chosen.independent_palm_mold()
                    && !other.independent_palm_mold()
                {
                    self.palms.sync_mold_from_opposite(side);
                } else if !chosen.placeable_block() || chosen.independent_palm_mold() {
                    self.palms.reset_mold(side);
                }
                self.palms.assign_blocks(side, u32::from(chosen.placeable_block()));
                let closed = next_tray.focused_on(side).closed();
                return (self.palms, closed, self.temper, attire.with_hand_relic(side, chosen), true);
            }
        }
        if hand.gripped() {
            self.temper.push_stowed_wear(hand, self.temper.palm_wear(side));
            self.temper.assign_palm_wear(side, 0);
            let stowed = self.tray.closed().stowed_with(hand, self.palms.blocks_on(side)).focused_on(side);
            self.palms.assign_blocks(side, 0);
            if hand.independent_palm_mold() {
                self.palms.reset_mold(side);
            }
            (self.palms, stowed, self.temper, attire.with_hand_relic(side, HeldRelic::EmptyPalm), true)
        } else if self.tray.open() {
            (self.palms, self.tray.closed(), self.temper, attire, true)
        } else {
            let (next_tray, opened) = self.tray.opened_if_populated(side);
            (self.palms, next_tray, self.temper, attire, opened)
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn place_scaled(
        mut self,
        side: f32,
        attire: EvaAttire,
        eye: Vec3,
        view_dir: Vec3,
        avatar_pos: Vec3,
        spec: ModuleSpec,
        world: &mut dyn PlanetaryWorld,
    ) -> (PalmStack, SatchelTray, SatchelTemper, EvaAttire, bool) {
        let (hand, _) = attire.hand_pair(side);
        if !world.place_scaled_surface_module(eye, view_dir, avatar_pos, spec) {
            return (self.palms, self.tray, self.temper, attire, false);
        }
        let focused = self.tray.closed().focused_on(side);
        let (next_tray, refilled) = focused.taken_stowed(hand);
        let next_hand = if refilled { hand } else { HeldRelic::EmptyPalm };
        self.palms.assign_blocks(side, u32::from(refilled));
        (self.palms, next_tray, self.temper, attire.with_hand_relic(side, next_hand), true)
    }
}
