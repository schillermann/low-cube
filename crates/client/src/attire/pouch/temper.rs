use world::PalmMolds;
use super::super::relic::{HeldRelic, ToolCondition};
use super::super::EvaAttire;

const TRACKED_COPIES: usize = 4;

/// Per-hand and per-slot tool surface-finish memory across stowing (`2` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SatchelTemper {
    palms: (u16, u16),
    stowed: [[u16; TRACKED_COPIES]; HeldRelic::COUNT],
}

impl SatchelTemper {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        palms: (u16, u16),
        stowed: [[u16; TRACKED_COPIES]; HeldRelic::COUNT],
    ) -> Self {
        Self { palms, stowed }
    }

    pub(super) const fn empty() -> Self {
        Self::new((0, 0), [[0; TRACKED_COPIES]; HeldRelic::COUNT])
    }

    pub(super) const fn palm_wear(self, side: f32) -> u16 {
        if side > 0.0 { self.palms.0 } else { self.palms.1 }
    }

    pub(super) fn assign_palm_wear(&mut self, side: f32, wear: u16) {
        if side > 0.0 {
            self.palms.0 = wear;
        } else {
            self.palms.1 = wear;
        }
    }

    #[cfg(test)]
    pub(super) const fn palm_condition(self, side: f32, relic: HeldRelic) -> ToolCondition {
        ToolCondition::from_wear(self.palm_wear(side), relic.endurance_span())
    }

    pub(super) const fn stowed_condition(self, relic: HeldRelic, rank: usize) -> ToolCondition {
        if !relic.wearable() || rank >= TRACKED_COPIES {
            ToolCondition::Pristine
        } else {
            ToolCondition::from_wear(self.stowed[relic.code() as usize][rank], relic.endurance_span())
        }
    }

    pub(super) fn wardrobe_molds(self, base: PalmMolds, attire: EvaAttire) -> PalmMolds {
        let (r, l) = attire.held_pair();
        let rm = if r.wearable() {
            ToolCondition::from_wear(self.palm_wear(1.0), r.endurance_span()).into_mold()
        } else {
            base.on_side(1.0)
        };
        let lm = if l.wearable() {
            ToolCondition::from_wear(self.palm_wear(-1.0), l.endurance_span()).into_mold()
        } else {
            base.on_side(-1.0)
        };
        PalmMolds::new(rm, lm)
    }

    pub(super) fn push_stowed_wear(&mut self, relic: HeldRelic, wear: u16) {
        if !relic.wearable() {
            return;
        }
        let row = &mut self.stowed[relic.code() as usize];
        let mut k = TRACKED_COPIES - 1;
        while k > 0 {
            row[k] = row[k - 1];
            k -= 1;
        }
        row[0] = wear;
    }

    pub(super) fn take_stowed_wear(&mut self, relic: HeldRelic, rank: usize) -> u16 {
        if !relic.wearable() || rank >= TRACKED_COPIES {
            return 0;
        }
        let row = &mut self.stowed[relic.code() as usize];
        let wear = row[rank];
        for k in rank..(TRACKED_COPIES - 1) {
            row[k] = row[k + 1];
        }
        row[TRACKED_COPIES - 1] = 0;
        wear
    }

    pub(super) fn reconcile_attire(&mut self, attire: EvaAttire) {
        let (r, l) = attire.held_pair();
        if !r.wearable() {
            self.palms.0 = 0;
        }
        if !l.wearable() {
            self.palms.1 = 0;
        }
    }

    pub(super) fn transition_attire(&mut self, prev: EvaAttire, next: EvaAttire) {
        let ((pr, pl), (nr, nl)) = (prev.held_pair(), next.held_pair());
        if nr != pr {
            self.palms.0 = 0;
        }
        if nl != pl {
            self.palms.1 = 0;
        }
        self.reconcile_attire(next);
    }

    pub(super) fn strain_palm(&mut self, side: f32, relic: HeldRelic, strain: u16) -> bool {
        if !relic.wearable() || strain == 0 {
            return false;
        }
        let next = self.palm_wear(side).saturating_add(strain);
        if next >= relic.endurance_span() {
            self.assign_palm_wear(side, 0);
            true
        } else {
            self.assign_palm_wear(side, next);
            false
        }
    }
}
