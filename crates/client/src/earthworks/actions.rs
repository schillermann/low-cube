use world::PlanetaryWorld;
use crate::attire::{HeldRelic, StrideRhythm, YouthAttire};
use crate::blueprint::BlueprintFolio;
use crate::controls::{ActionBindings, InputTrigger};
use crate::grip::DigGrip;
use super::hearth::{HearthMedium, MealOutcome, WorkstationStation};
use super::reach::SurfaceReach;
use super::toss::TossCadence;
use super::PalmEarthworks;

/// Collaborator executing active hand contact, hearth workstations, nourishment, and folio builds (`1` field).
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
        attire: YouthAttire,
        world: &mut dyn PlanetaryWorld,
    ) -> (PalmEarthworks, YouthAttire, bool, bool) {
        let cur = self.state.reconcile_attire(attire);
        let (ns, na, built, handled) = folio.act_open(cur.satchel, attire, world);
        (PalmEarthworks::with_state(ns, cur.tome, cur.dig.halted(), TossCadence::Quiet), na, built, handled)
    }

    pub(super) fn try_build_folio(
        self,
        folio: &mut BlueprintFolio,
        trigger: InputTrigger,
        bindings: &ActionBindings,
        attire: YouthAttire,
        world: &mut dyn PlanetaryWorld,
    ) -> (PalmEarthworks, YouthAttire, bool) {
        let cur = self.state.reconcile_attire(attire);
        let (ns, na, built) = folio.try_build_trigger(trigger, bindings, cur.satchel, attire, world);
        (PalmEarthworks::with_state(ns, cur.tome, cur.dig.halted(), TossCadence::Quiet), na, built)
    }

    pub(super) fn act_with_hand(
        self,
        side: f32,
        reach: SurfaceReach,
        attire: YouthAttire,
        rhythm: StrideRhythm,
        world: &mut dyn PlanetaryWorld,
    ) -> (PalmEarthworks, YouthAttire, StrideRhythm, bool, MealOutcome) {
        let ew = self.state;
        if ew.satchel.open() {
            let (next_satchel, next_attire, _) = ew.satchel.transfer_hand(side, attire);
            return (
                PalmEarthworks::with_dig(next_satchel, ew.tome, ew.dig.with_grip(DigGrip::idle())),
                next_attire,
                rhythm,
                false,
                MealOutcome::Untouched,
            );
        }
        if ew.tome.open() {
            let (next_tome, next_satchel, next_attire, ok) = ew.tome.craft_selected(side, ew.satchel, attire);
            let next_tome = next_tome.drain_crafted_notes(world);
            let next_rhythm = if ok { SurfaceReach::swing_rhythm(next_attire, rhythm, side) } else { rhythm };
            return (
                PalmEarthworks::with_dig(next_satchel, next_tome, ew.dig.with_grip(DigGrip::idle())),
                next_attire,
                next_rhythm,
                false,
                MealOutcome::Untouched,
            );
        }
        if ew.dig.untouched() && !ew.dig.grip().any_held() {
            if let Some(aimed) = world.aimed_surface_medium(reach.eye(), reach.view_dir()) {
                let ambient = HearthMedium::from_vacuum(world.vacuum_medium_at(reach.eye()));
                if let Some((ns, nt, na, ok)) =
                    WorkstationStation::new(aimed, ambient).interact(side, ew.satchel, ew.tome, attire, world)
                {
                    let nr = SurfaceReach::swing_rhythm(na, rhythm, side);
                    return (PalmEarthworks::with_dig(ns, nt, ew.dig.with_grip(DigGrip::idle())), na, nr, ok, MealOutcome::Untouched);
                }
            }
            let held = attire.hand_pair(side).0;
            if held.edible_provision() {
                let find = world.forage_surface(reach.eye(), reach.view_dir());
                if find.gathered() {
                    let (ns, na) = ew
                        .satchel
                        .reconcile_attire(attire)
                        .collect_foraged(side, HeldRelic::from_surface_find(find), attire);
                    return (PalmEarthworks::with_dig(ns, ew.tome, ew.dig.with_grip(DigGrip::idle())), na, rhythm, true, MealOutcome::Untouched);
                }
                let (ns, na, ate) = ew.satchel.reconcile_attire(attire).consume_provision(side, attire);
                if ate {
                    world.note_consumed_provision(held.code());
                    let nr = SurfaceReach::swing_rhythm(attire, rhythm, side);
                    return (PalmEarthworks::with_dig(ns, ew.tome, ew.dig.with_grip(DigGrip::idle())), na, nr, false, MealOutcome::from_relic(held));
                }
            }
        }
        let (ns, na, nd, nr, changed) = reach.strike(side, ew.satchel, attire, rhythm, ew.dig, world);
        (PalmEarthworks::with_dig(ns, ew.tome, nd), na, nr, changed, MealOutcome::Untouched)
    }
}
