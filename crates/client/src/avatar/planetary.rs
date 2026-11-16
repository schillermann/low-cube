use glam::{Mat4, Quat, Vec3};
use fauna::{SightCone, WoodlandFauna};
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{PhotoShutter, SightMode, YouthAttire};
use crate::controls::{ActionBindings, InputTrigger, KeyboardControls};
use crate::flight::GravitationalFlight;
#[cfg(test)]
use crate::flight::SurfaceFlight;
use super::condition::AvatarCondition;
use super::embodied::EmbodiedAvatar;
use super::satchel_guard::SatchelAvatar;
use super::Avatar;

/// Vital mortality decorator over `SatchelAvatar<EmbodiedAvatar>` gating actions during lethal core contact and cardiac blackout.
pub struct PlanetaryAvatar {
    origin: SatchelAvatar<EmbodiedAvatar>,
}

impl PlanetaryAvatar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn from_guarded(origin: SatchelAvatar<EmbodiedAvatar>) -> Self {
        Self { origin }
    }

    pub fn with_attire_and_bindings(
        initial_position: Vec3, flight: Box<dyn GravitationalFlight>, attire: YouthAttire, bindings: ActionBindings,
    ) -> Self {
        Self::from_guarded(SatchelAvatar::new(EmbodiedAvatar::with_attire_and_bindings(
            initial_position,
            flight,
            attire,
            bindings,
        )))
    }

    pub fn with_starting_satchel(mut self, provider: &dyn crate::server_satchel::StartingSatchel) -> Self {
        self.origin.provision_satchel(provider);
        self
    }

    #[cfg(test)]
    pub fn with_test_engines(self) -> Self {
        self.with_starting_satchel(&crate::server_satchel::TestKitSatchel::new())
    }
    #[cfg(test)]
    pub fn with_attire(initial_position: Vec3, flight: Box<dyn GravitationalFlight>, attire: YouthAttire) -> Self {
        Self::with_attire_and_bindings(initial_position, flight, attire, ActionBindings::seeded())
    }
    #[cfg(test)]
    pub fn with_flight(initial_position: Vec3, flight: Box<dyn GravitationalFlight>) -> Self {
        Self::with_attire(initial_position, flight, YouthAttire::squire())
    }
    #[cfg(test)]
    pub fn at(initial_position: Vec3) -> Self {
        Self::with_flight(initial_position, Box::new(SurfaceFlight::terrestrial(1.0)))
    }

    fn incapacitated(&self, world: &dyn PlanetaryWorld) -> bool {
        self.origin.perished() || self.origin.lethal_contact(world)
    }
}

impl Avatar for PlanetaryAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        if !self.origin.perished() {
            self.origin.rotate_view(dx, dy);
        }
    }

    fn use_left_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.incapacitated(world) {
            false
        } else {
            self.origin.use_left_hand(world)
        }
    }

    fn use_right_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.incapacitated(world) {
            false
        } else {
            self.origin.use_right_hand(world)
        }
    }

    fn release_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.origin.perished() {
            self.origin.abort_hands(world)
        } else {
            self.origin.release_hand(world)
        }
    }

    fn step_digging(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.origin.perished() {
            false
        } else {
            self.origin.step_digging(world)
        }
    }

    fn apply_attire(&mut self, attire: YouthAttire) {
        self.origin.apply_attire(attire);
    }

    fn apply_bindings(&mut self, bindings: ActionBindings) {
        self.origin.apply_bindings(bindings);
    }

    fn dispatch_trigger(&mut self, trigger: InputTrigger) -> bool {
        self.origin.dispatch_trigger(trigger)
    }

    fn adjust_aimed_rig(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        if self.origin.perished() {
            false
        } else {
            self.origin.adjust_aimed_rig(trigger, world)
        }
    }

    fn cast_triggered_relic(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        if self.incapacitated(world) {
            false
        } else {
            self.origin.cast_triggered_relic(trigger, world)
        }
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        if self.incapacitated(world) {
            self.origin.step_blackout(dt, world);
        } else {
            self.origin.step_motion(dt, controls, world);
        }
    }

    fn blend_subtick(&mut self, alpha: f32) {
        self.origin.blend_subtick(alpha);
    }

    fn step_fauna(&mut self, dt: f32, fauna: &mut WoodlandFauna, world: &dyn PlanetaryWorld) {
        self.origin.step_fauna(dt, fauna, world);
    }

    fn hunt_fauna(&mut self, side: f32, fauna: &mut WoodlandFauna, world: &mut dyn PlanetaryWorld) -> (bool, bool) {
        if self.incapacitated(world) {
            (false, false)
        } else {
            self.origin.hunt_fauna(side, fauna, world)
        }
    }

    fn sight_cone(&self) -> SightCone {
        self.origin.sight_cone()
    }

    fn sight_cone_for_mode(&self, sight: SightMode) -> SightCone {
        self.origin.sight_cone_for_mode(sight)
    }

    fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        self.origin.view_projection_matrix(aspect)
    }

    fn view_projection_for_sight(&self, aspect: f32, sight: SightMode) -> Mat4 {
        self.origin.view_projection_for_sight(aspect, sight)
    }

    fn surface_rotation(&self) -> Quat {
        self.origin.surface_rotation()
    }

    fn emit_appearance(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_appearance(target);
    }

    fn emit_appearance_for_sight(&self, sight: SightMode, target: &mut dyn MeshTarget) {
        self.origin.emit_appearance_for_sight(sight, target);
    }

    fn emit_surroundings(&self, world: &dyn PlanetaryWorld, target: &mut dyn MeshTarget) {
        self.origin.emit_surroundings(world, target);
    }

    fn meadow_anchor(&self) -> (i32, i32, i32) {
        self.origin.meadow_anchor()
    }

    fn take_photo_shutter(&mut self) -> PhotoShutter {
        self.origin.take_photo_shutter()
    }

    fn apply_polaroid_mosaic(&mut self, mosaic: Vec<Vec3>) {
        self.origin.apply_polaroid_mosaic(mosaic);
    }

    #[cfg(test)]
    fn polaroid_tilt(&self) -> f32 {
        self.origin.polaroid_tilt()
    }

    fn emit_reticle(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_reticle(target);
    }

    fn sync_world_presence(&self, world: &mut dyn PlanetaryWorld) {
        self.origin.sync_world_presence(world);
    }

    fn land_on_world(&mut self, world: &dyn PlanetaryWorld) {
        self.origin.land_on_world(world);
    }
}
