use world::{MeshTarget, PlanetaryWorld};
use crate::controls::{InputTrigger, KeyboardControls};
use super::condition::AvatarCondition;
use super::AvatarMotor;

/// Focused `AvatarMotor` decorator suspending gaze, stride, digging, and reticle emission while the satchel UI is open.
pub(super) struct SatchelAvatar<M> {
    origin: M,
}

impl<M> SatchelAvatar<M> {
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

impl<M: AvatarCondition> AvatarCondition for SatchelAvatar<M> {
    fn perished(&self) -> bool { self.origin.perished() }
    fn lethal_contact(&self, world: &dyn PlanetaryWorld) -> bool { self.origin.lethal_contact(world) }
    fn satchel_open(&self) -> bool { self.origin.satchel_open() }
    fn driving(&self) -> bool { self.origin.driving() }
    fn abort_hands(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.origin.abort_hands(world) }
    fn step_satchel_idle(&mut self, dt: f32) { self.origin.step_satchel_idle(dt); }
    fn step_blackout(&mut self, dt: f32, world: &dyn PlanetaryWorld) { self.origin.step_blackout(dt, world); }
}

impl<M: AvatarMotor + AvatarCondition> AvatarMotor for SatchelAvatar<M> {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        if !self.origin.satchel_open() {
            self.origin.rotate_view(dx, dy);
        }
    }

    fn act_hand(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> bool {
        self.origin.act_hand(side, world)
    }

    fn release_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.origin.satchel_open() {
            self.origin.abort_hands(world)
        } else {
            self.origin.release_hand(world)
        }
    }

    fn step_digging(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        !self.origin.satchel_open() && self.origin.step_digging(world)
    }

    fn adjust_aimed_rig(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        self.origin.adjust_aimed_rig(trigger, world)
    }

    fn cast_triggered_relic(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        self.origin.cast_triggered_relic(trigger, world)
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        if self.origin.satchel_open() {
            self.origin.step_satchel_idle(dt);
        } else {
            self.origin.step_motion(dt, controls, world);
        }
    }

    fn hunt_fauna(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> (bool, bool) {
        if self.origin.satchel_open() || self.origin.driving() {
            (false, false)
        } else {
            self.origin.hunt_fauna(side, world)
        }
    }

    fn emit_reticle_for_aspect(&self, aspect: f32, target: &mut dyn MeshTarget) {
        if !self.origin.satchel_open() {
            self.origin.emit_reticle_for_aspect(aspect, target);
        }
    }
}
