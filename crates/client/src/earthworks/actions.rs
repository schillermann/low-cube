use world::PlanetaryWorld;
use crate::attire::{EvaAttire, StrideRhythm};
use crate::blueprint::BlueprintFolio;
use crate::controls::{ActionBindings, InputTrigger};
use crate::grip::DigGrip;
use super::reach::SurfaceReach;
use super::toss::TossCadence;
use super::PalmEarthworks;

/// Collaborator executing active EVA hand contact and CAD blueprint folio builds (`1` field).
pub(super) struct PalmContact {
    state: PalmEarthworks,
}

impl PalmContact {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(state: PalmEarthworks) -> Self {
        Self { state }
    }

    pub(super) fn act_folio(
        self,
        folio: &mut BlueprintFolio,
        attire: EvaAttire,
        world: &mut dyn PlanetaryWorld,
    ) -> (PalmEarthworks, EvaAttire, bool, bool) {
        let cur = self.state.reconcile_attire(attire);
        let (ns, na, built, handled) = folio.act_open(cur.satchel, attire, world);
        (PalmEarthworks::with_state(ns, cur.dig.halted(), TossCadence::Quiet), na, built, handled)
    }

    pub(super) fn try_build_folio(
        self,
        folio: &mut BlueprintFolio,
        trigger: InputTrigger,
        bindings: &ActionBindings,
        attire: EvaAttire,
        world: &mut dyn PlanetaryWorld,
    ) -> (PalmEarthworks, EvaAttire, bool) {
        let cur = self.state.reconcile_attire(attire);
        let (ns, na, built) = folio.try_build_trigger(trigger, bindings, cur.satchel, attire, world);
        (PalmEarthworks::with_state(ns, cur.dig.halted(), TossCadence::Quiet), na, built)
    }

    pub(super) fn act_with_hand(
        self,
        side: f32,
        reach: SurfaceReach,
        attire: EvaAttire,
        rhythm: StrideRhythm,
        world: &mut dyn PlanetaryWorld,
    ) -> (PalmEarthworks, EvaAttire, StrideRhythm, bool) {
        let ew = self.state;
        if ew.satchel.open() {
            let (next_satchel, next_attire, _) = ew.satchel.transfer_hand(side, attire);
            return (
                PalmEarthworks::with_dig(next_satchel, ew.dig.with_grip(DigGrip::idle())),
                next_attire,
                rhythm,
                false,
            );
        }
        let (ns, na, nd, nr, changed) = reach.strike(side, ew.satchel, attire, rhythm, ew.dig, world);
        let next_drag = if !ew.drag.active()
            && !changed
            && nd.untouched()
            && nd.grip().pending_place_on(side)
            && na.hand_pair(side).0.scalable_module()
        {
            super::ScaleDrag::begin(side, reach)
        } else {
            super::ScaleDrag::Idle
        };
        (PalmEarthworks::with_full_state(ns, nd, ew.toss, next_drag), na, nr, changed)
    }
}
