use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld};
use crate::controls::{InputTrigger, KeyboardControls};
use super::condition::AvatarCondition;
use super::span::BodySpan;
use super::AvatarMotor;

/// Polymorphic post-revival frame state avoiding boolean flags or nullable `Option` fields.
#[derive(Clone, Copy, Debug)]
enum RevivalLocus {
    Grounded,
    Earthbound,
}

impl RevivalLocus {
    fn reconcile<M: AvatarCondition>(self, avatar: &mut M, world: &mut dyn PlanetaryWorld) -> (Self, bool) {
        match self {
            Self::Grounded => (Self::Grounded, avatar.reconcile_frame(world)),
            Self::Earthbound => {
                let arrival = world
                    .assigned_spawn_point()
                    .unwrap_or_else(|| BodySpan::spawned_on(world).current());
                world.anchor_to_frame(world::CosmicFrame::earth(), arrival);
                avatar.revive_on_earth(world);
                (Self::Grounded, true)
            }
        }
    }
}

/// Focused `AvatarMotor` decorator gating motor actions during cardiac blackout and re-anchoring offworld revivals (`2` fields).
pub(super) struct MortalMotor<M> {
    origin: M,
    locus: RevivalLocus,
}

impl<M> MortalMotor<M> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn with_locus(origin: M, locus: RevivalLocus) -> Self {
        Self { origin, locus }
    }

    pub(super) const fn new(origin: M) -> Self {
        Self::with_locus(origin, RevivalLocus::Grounded)
    }

    pub(super) const fn inner(&self) -> &M {
        &self.origin
    }

    pub(super) fn inner_mut(&mut self) -> &mut M {
        &mut self.origin
    }
}

impl<M: AvatarCondition> MortalMotor<M> {
    fn incapacitated(&self, world: &dyn PlanetaryWorld) -> bool {
        self.origin.perished() || self.origin.lethal_contact(world)
    }

    pub(super) fn reconcile_cosmic_frame(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        let (next, handed_off) = self.locus.reconcile(&mut self.origin, world);
        self.locus = next;
        handed_off
    }
}

impl<M: AvatarCondition> AvatarCondition for MortalMotor<M> {
    fn perished(&self) -> bool { self.origin.perished() }
    fn lethal_contact(&self, world: &dyn PlanetaryWorld) -> bool { self.origin.lethal_contact(world) }
    fn satchel_open(&self) -> bool { self.origin.satchel_open() }
    fn driving(&self) -> bool { self.origin.driving() }
    fn sheltered_berth(&self) -> bool { self.origin.sheltered_berth() }
    fn ear_locus(&self) -> Vec3 { self.origin.ear_locus() }
    fn abort_hands(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.origin.abort_hands(world) }
    fn step_satchel_idle(&mut self, dt: f32) { self.origin.step_satchel_idle(dt); }
    fn step_blackout(&mut self, dt: f32, world: &dyn PlanetaryWorld) { self.origin.step_blackout(dt, world); }
    fn revive_on_earth(&mut self, world: &dyn PlanetaryWorld) { self.origin.revive_on_earth(world); }
    fn reconcile_frame(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.origin.reconcile_frame(world) }
}

impl<M: AvatarMotor + AvatarCondition> AvatarMotor for MortalMotor<M> {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        if !self.origin.perished() {
            self.origin.rotate_view(dx, dy);
        }
    }

    fn act_hand(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> bool {
        !self.incapacitated(world) && self.origin.act_hand(side, world)
    }

    fn release_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.incapacitated(world) {
            self.origin.abort_hands(world)
        } else {
            self.origin.release_hand(world)
        }
    }

    fn step_digging(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.incapacitated(world) {
            self.origin.abort_hands(world)
        } else {
            self.origin.step_digging(world)
        }
    }

    fn adjust_aimed_rig(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        !self.incapacitated(world) && self.origin.adjust_aimed_rig(trigger, world)
    }

    fn cast_triggered_relic(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        !self.incapacitated(world) && self.origin.cast_triggered_relic(trigger, world)
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        if self.incapacitated(world) {
            let was_perished = self.origin.perished();
            self.origin.step_blackout(dt, world);
            if was_perished && !self.origin.perished() && !world.cosmic_frame().earth_centered() {
                self.locus = RevivalLocus::Earthbound;
            }
        } else {
            self.origin.step_motion(dt, controls, world);
        }
    }

    fn hunt_fauna(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> (bool, bool) {
        if self.incapacitated(world) {
            (false, false)
        } else {
            self.origin.hunt_fauna(side, world)
        }
    }

    fn emit_reticle_for_aspect(&self, aspect: f32, target: &mut dyn MeshTarget) {
        self.origin.emit_reticle_for_aspect(aspect, target);
    }
}
