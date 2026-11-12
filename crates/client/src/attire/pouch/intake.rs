use super::super::relic::HeldRelic;
use super::super::tray::SatchelTray;
use super::super::YouthAttire;
use super::palms::PalmStack;

/// Encapsulated harvest, foraging, timber-felling, and crafting intake across palms and satchel tray.
pub(super) struct SatchelIntake {
    palms: PalmStack,
    tray: SatchelTray,
}

impl SatchelIntake {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(palms: PalmStack, tray: SatchelTray) -> Self {
        Self { palms, tray }
    }

    fn resource_tally(&self, relic: HeldRelic, attire: YouthAttire) -> u32 {
        let (r, l) = attire.held_pair();
        u32::from(r == relic) + u32::from(l == relic) + self.tray.count(relic)
    }

    pub(super) fn excavated_block(mut self, side: f32, relic: HeldRelic, attire: YouthAttire) -> (PalmStack, SatchelTray, YouthAttire) {
        if relic == HeldRelic::FieldCamera {
            self.palms.clear_placed_cameras();
        }
        if relic == HeldRelic::CantileverCrane {
            for (part, qty) in [
                (HeldRelic::FieldCamera, 1),
                (HeldRelic::RocketEngine, 1),
                (HeldRelic::AircraftEngine, 1),
                (HeldRelic::SteeringWheel, 1),
                (HeldRelic::CarWheel, 8),
                (HeldRelic::CrossAxle, 4),
                (HeldRelic::CabinSeat, 2),
                (HeldRelic::PedalCluster, 2),
            ] {
                if self.resource_tally(part, attire) == 0 {
                    self.tray.stow_relic(part, qty);
                }
            }
        }
        let (hand, other) = attire.hand_pair(side);
        if hand.free_palm() || (!hand.placeable_block() && other.free_palm()) {
            let (target, opp) = if hand.free_palm() { (side, other) } else { (-side, hand) };
            if opp == relic {
                self.palms.sync_mold_from_opposite(target);
            } else {
                self.palms.reset_mold(target);
            }
            self.palms.assign_blocks(target, 1);
            self.tray.focus_side(target);
            return (self.palms, self.tray, attire.with_hand_relic(target, relic));
        }
        let target = if hand.placeable_block() { side } else if other.placeable_block() { -side } else { 0.0 };
        let held = if target > 0.0 { attire.held_pair().0 } else { attire.held_pair().1 };
        if target != 0.0 && held != relic {
            self.tray.stow_relic(held, self.palms.blocks_on(target));
            self.palms.assign_blocks(target, 1);
            self.palms.reset_mold(target);
            self.tray.focus_side(target);
            return (self.palms, self.tray, attire.with_hand_relic(target, relic));
        }
        self.tray.focus_side(if target != 0.0 { target } else { side });
        self.tray.stow_relic(relic, 1);
        (self.palms, self.tray, attire)
    }

    pub(super) fn foraged(mut self, side: f32, relic: HeldRelic, attire: YouthAttire) -> (PalmStack, SatchelTray, YouthAttire) {
        let (r, l) = attire.held_pair();
        self.tray.focus_side(side);
        if (side > 0.0 && r.free_palm()) || (r.free_palm() && !l.free_palm()) {
            (self.palms, self.tray, attire.with_relics(relic, l))
        } else if l.free_palm() {
            (self.palms, self.tray, attire.with_relics(r, relic))
        } else {
            self.tray.stow_relic(relic, 1);
            (self.palms, self.tray, attire)
        }
    }

    pub(super) fn felled_timber(mut self, side: f32, attire: YouthAttire) -> (PalmStack, SatchelTray, YouthAttire) {
        let (r, l) = attire.held_pair();
        let next_attire = if r.free_palm() {
            self.palms.assign_blocks(1.0, 1);
            self.tray.stow_relic(HeldRelic::TimberBlock, 2);
            attire.with_relics(HeldRelic::TimberBlock, l)
        } else if l.free_palm() {
            self.palms.assign_blocks(-1.0, 1);
            self.tray.stow_relic(HeldRelic::TimberBlock, 2);
            attire.with_relics(r, HeldRelic::TimberBlock)
        } else {
            self.tray.stow_relic(HeldRelic::TimberBlock, 3);
            attire
        };
        self.tray.focus_side(side);
        self.palms.reconcile_attire(next_attire);
        (self.palms, self.tray, next_attire)
    }

    pub(super) fn crafted_batch(
        mut self, ingredients: [(HeldRelic, u32); 2], product: HeldRelic, batch_yield: u32, side: f32, mut attire: YouthAttire,
    ) -> (PalmStack, SatchelTray, YouthAttire) {
        for (ing, mut rem) in ingredients {
            while rem > 0 {
                let (r, l) = attire.held_pair();
                if !self.tray.take_stowed(ing) {
                    if r == ing {
                        self.palms.assign_blocks(1.0, 0);
                        attire = attire.with_relics(HeldRelic::EmptyPalm, l);
                    } else if l == ing {
                        self.palms.assign_blocks(-1.0, 0);
                        attire = attire.with_relics(r, HeldRelic::EmptyPalm);
                    }
                }
                rem -= 1;
            }
        }
        let (hand, other) = attire.hand_pair(side);
        if hand.gripped() && !other.free_palm() {
            self.tray.stow_relic(hand, self.palms.blocks_on(side));
            self.palms.assign_blocks(side, 0);
            attire = attire.with_hand_relic(side, HeldRelic::EmptyPalm);
        }
        let (mut palms, mut tray, next_attire) = self.foraged(side, product, attire);
        if batch_yield > 1 {
            tray.stow_relic(product, batch_yield - 1);
        }
        palms.reconcile_attire(next_attire);
        (palms, tray, next_attire)
    }
}
