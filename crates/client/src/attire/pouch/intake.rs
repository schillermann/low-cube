use super::super::relic::HeldRelic;
use super::super::tray::SatchelTray;
use super::super::YouthAttire;
use super::palms::PalmStack;
use super::temper::SatchelTemper;

/// Encapsulated harvest, foraging, timber-felling, and crafting intake across palms, tray, and tool temper (`3` fields).
pub(super) struct SatchelIntake {
    palms: PalmStack,
    tray: SatchelTray,
    temper: SatchelTemper,
}

impl SatchelIntake {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(palms: PalmStack, tray: SatchelTray, temper: SatchelTemper) -> Self {
        Self { palms, tray, temper }
    }

    fn resource_tally(&self, relic: HeldRelic, attire: YouthAttire) -> u32 {
        let (r, l) = attire.held_pair();
        u32::from(r == relic) + u32::from(l == relic) + self.tray.count(relic)
    }

    pub(super) fn from_manifest(
        mut self,
        manifest: world::SatchelManifest,
        attire: YouthAttire,
    ) -> (PalmStack, SatchelTray, SatchelTemper, YouthAttire) {
        let clean = manifest.sanitized();
        let (rc, lc) = clean.held_codes();
        let (right, left) = (HeldRelic::from_code(rc), HeldRelic::from_code(lc));
        let next_attire = attire.with_relics(right, left);
        for code in 1..(HeldRelic::COUNT as u8) {
            let held = u32::from(right.code() == code) + u32::from(left.code() == code);
            let stowed = clean.count_of(code).saturating_sub(held);
            if stowed > 0 {
                self.tray = self.tray.stowed_with(HeldRelic::from_code(code), stowed);
            }
        }
        self.palms.reconcile_attire(next_attire);
        self.temper.reconcile_attire(next_attire);
        (self.palms, self.tray, self.temper, next_attire)
    }

    pub(super) fn excavated_block(
        mut self,
        side: f32,
        relic: HeldRelic,
        attire: YouthAttire,
    ) -> (PalmStack, SatchelTray, SatchelTemper, YouthAttire) {
        if relic == HeldRelic::CantileverCrane {
            for (part, qty) in [
                (HeldRelic::FieldCamera, 1),
                (HeldRelic::VehicleBlueprint, 1),
                (HeldRelic::FlightConsole, 1),
                (HeldRelic::RocketNozzle, 6),
                (HeldRelic::RocketCamera, 4),
                (HeldRelic::AircraftEngine, 1),
                (HeldRelic::SteeringWheel, 1),
                (HeldRelic::CarWheel, 8),
                (HeldRelic::CabinSeat, 2),
                (HeldRelic::PedalCluster, 2),
            ] {
                if self.resource_tally(part, attire) == 0 {
                    self.tray = self.tray.stowed_with(part, qty);
                }
            }
        }
        let (hand, other) = attire.hand_pair(side);
        if hand.free_palm() || (!hand.placeable_block() && other.free_palm()) {
            let (target, opp) = if hand.free_palm() { (side, other) } else { (-side, hand) };
            if opp == relic && !relic.independent_palm_mold() {
                self.palms.sync_mold_from_opposite(target);
            } else {
                self.palms.reset_mold(target);
            }
            self.palms.assign_blocks(target, 1);
            self.temper.assign_palm_wear(target, 0);
            return (self.palms, self.tray.focused_on(target), self.temper, attire.with_hand_relic(target, relic));
        }
        let target = if hand.placeable_block() { side } else if other.placeable_block() { -side } else { 0.0 };
        let held = if target > 0.0 { attire.held_pair().0 } else { attire.held_pair().1 };
        if target != 0.0 && held != relic {
            let next_tray = self.tray.stowed_with(held, self.palms.blocks_on(target)).focused_on(target);
            self.palms.assign_blocks(target, 1);
            self.palms.reset_mold(target);
            self.temper.assign_palm_wear(target, 0);
            return (self.palms, next_tray, self.temper, attire.with_hand_relic(target, relic));
        }
        let focus = if target != 0.0 { target } else { side };
        (self.palms, self.tray.focused_on(focus).stowed_with(relic, 1), self.temper, attire)
    }

    pub(super) fn foraged(
        mut self,
        side: f32,
        relic: HeldRelic,
        attire: YouthAttire,
    ) -> (PalmStack, SatchelTray, SatchelTemper, YouthAttire) {
        let (r, l) = attire.held_pair();
        self.tray = self.tray.focused_on(side);
        let next_attire = if (side > 0.0 && r.free_palm()) || (r.free_palm() && !l.free_palm()) {
            if !l.placeable_block() || relic.independent_palm_mold() || l.independent_palm_mold() {
                self.palms.reset_mold(1.0);
            }
            self.temper.assign_palm_wear(1.0, 0);
            attire.with_relics(relic, l)
        } else if l.free_palm() {
            if !r.placeable_block() || relic.independent_palm_mold() || r.independent_palm_mold() {
                self.palms.reset_mold(-1.0);
            }
            self.temper.assign_palm_wear(-1.0, 0);
            attire.with_relics(r, relic)
        } else {
            self.tray = self.tray.stowed_with(relic, 1);
            attire
        };
        self.palms.reconcile_attire(next_attire);
        (self.palms, self.tray, self.temper, next_attire)
    }

    pub(super) fn felled_timber(
        mut self,
        side: f32,
        attire: YouthAttire,
    ) -> (PalmStack, SatchelTray, SatchelTemper, YouthAttire) {
        let (r, l) = attire.held_pair();
        let next_attire = if r.free_palm() {
            self.palms.assign_blocks(1.0, 1);
            self.temper.assign_palm_wear(1.0, 0);
            self.tray = self.tray.stowed_with(HeldRelic::TimberBlock, 2);
            attire.with_relics(HeldRelic::TimberBlock, l)
        } else if l.free_palm() {
            self.palms.assign_blocks(-1.0, 1);
            self.temper.assign_palm_wear(-1.0, 0);
            self.tray = self.tray.stowed_with(HeldRelic::TimberBlock, 2);
            attire.with_relics(r, HeldRelic::TimberBlock)
        } else {
            self.tray = self.tray.stowed_with(HeldRelic::TimberBlock, 3);
            attire
        };
        self.palms.reconcile_attire(next_attire);
        (self.palms, self.tray.focused_on(side), self.temper, next_attire)
    }

    pub(super) fn crafted_batch(
        mut self,
        ingredients: [(HeldRelic, u32); 2],
        product: HeldRelic,
        batch_yield: u32,
        side: f32,
        mut attire: YouthAttire,
    ) -> (PalmStack, SatchelTray, SatchelTemper, YouthAttire) {
        for (ing, mut rem) in ingredients {
            while rem > 0 {
                let (r, l) = attire.held_pair();
                let (next_tray, taken) = self.tray.taken_stowed(ing);
                self.tray = next_tray;
                if taken {
                    let _ = self.temper.take_stowed_wear(ing, 0);
                } else if r == ing {
                    self.palms.assign_blocks(1.0, 0);
                    self.temper.assign_palm_wear(1.0, 0);
                    attire = attire.with_relics(HeldRelic::EmptyPalm, l);
                } else if l == ing {
                    self.palms.assign_blocks(-1.0, 0);
                    self.temper.assign_palm_wear(-1.0, 0);
                    attire = attire.with_relics(r, HeldRelic::EmptyPalm);
                }
                rem -= 1;
            }
        }
        let (hand, other) = attire.hand_pair(side);
        if hand.gripped() && !other.free_palm() {
            self.temper.push_stowed_wear(hand, self.temper.palm_wear(side));
            self.temper.assign_palm_wear(side, 0);
            self.tray = self.tray.stowed_with(hand, self.palms.blocks_on(side));
            self.palms.assign_blocks(side, 0);
            attire = attire.with_hand_relic(side, HeldRelic::EmptyPalm);
        }
        let (mut palms, mut tray, temper, next_attire) = self.foraged(side, product, attire);
        if batch_yield > 1 {
            tray = tray.stowed_with(product, batch_yield - 1);
        }
        palms.reconcile_attire(next_attire);
        (palms, tray, temper, next_attire)
    }

    pub(super) fn consume_manifest(
        mut self,
        manifest: &[(HeldRelic, u32)],
        mut attire: YouthAttire,
    ) -> Option<(PalmStack, SatchelTray, SatchelTemper, YouthAttire)> {
        if !manifest.iter().all(|&(relic, needed)| needed == 0 || self.resource_tally(relic, attire) >= needed) {
            return None;
        }
        for &(ing, mut rem) in manifest {
            while rem > 0 {
                let (r, l) = attire.held_pair();
                let (next_tray, taken) = self.tray.taken_stowed(ing);
                self.tray = next_tray;
                if taken {
                    let _ = self.temper.take_stowed_wear(ing, 0);
                } else if r == ing {
                    self.palms.assign_blocks(1.0, 0);
                    self.temper.assign_palm_wear(1.0, 0);
                    attire = attire.with_relics(HeldRelic::EmptyPalm, l);
                } else if l == ing {
                    self.palms.assign_blocks(-1.0, 0);
                    self.temper.assign_palm_wear(-1.0, 0);
                    attire = attire.with_relics(r, HeldRelic::EmptyPalm);
                }
                rem -= 1;
            }
        }
        self.palms.reconcile_attire(attire);
        Some((self.palms, self.tray, self.temper, attire))
    }
}
