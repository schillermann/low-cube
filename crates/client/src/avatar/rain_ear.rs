use acoustics::ShowerDemand;
use world::{MeshTarget, PlanetaryWeather as _, PlanetaryWorld, TroposphereDeck};
use crate::controls::{InputTrigger, KeyboardControls};
use super::condition::AvatarCondition;
use super::AvatarMotor;

/// Focused `AvatarMotor` decorator driving continuous woodland rain, cabin-muffled rain, and polar snow acoustics.
pub(super) struct RainEarMotor<M> {
    origin: M,
}

impl<M> RainEarMotor<M> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(origin: M) -> Self {
        Self { origin }
    }

    pub(super) const fn inner(&self) -> &M {
        &self.origin
    }

    pub(super) fn inner_mut(&mut self) -> &mut M {
        &mut self.origin
    }
}

impl<M: AvatarMotor + AvatarCondition> AvatarMotor for RainEarMotor<M> {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        self.origin.rotate_view(dx, dy);
    }

    fn act_hand(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> bool {
        self.origin.act_hand(side, world)
    }

    fn release_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        self.origin.release_hand(world)
    }

    fn step_digging(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        self.origin.step_digging(world)
    }

    fn adjust_aimed_rig(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        self.origin.adjust_aimed_rig(trigger, world)
    }

    fn cast_triggered_relic(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        self.origin.cast_triggered_relic(trigger, world)
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        self.origin.step_motion(dt, controls, world);
        if self.origin.perished() {
            ShowerDemand::calm().publish();
            return;
        }
        if let Some(eph) = world.celestial_atlas().ephemeris().filter(|e| e.frame().earth_centered()) {
            let deck = TroposphereDeck::new(eph.proportions().earth_half(), eph.epoch_secs() as f32);
            let (rate, frozen) = deck.precipitation_at(self.origin.ear_locus());
            ShowerDemand::new(rate, frozen, self.origin.sheltered_berth()).publish();
        } else {
            ShowerDemand::calm().publish();
        }
    }

    fn hunt_fauna(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> (bool, bool) {
        self.origin.hunt_fauna(side, world)
    }

    fn emit_reticle_for_aspect(&self, aspect: f32, target: &mut dyn MeshTarget) {
        self.origin.emit_reticle_for_aspect(aspect, target);
    }
}
