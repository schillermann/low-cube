use glam::Vec3;
use world::PlanetaryWorld;

/// Focused interface exposing vital and satchel state to outer `Avatar` guard decorators.
pub(super) trait AvatarCondition {
    fn perished(&self) -> bool;
    fn lethal_contact(&self, world: &dyn PlanetaryWorld) -> bool;
    fn satchel_open(&self) -> bool;
    fn driving(&self) -> bool;
    fn sheltered_berth(&self) -> bool;
    fn ear_locus(&self) -> Vec3;
    fn abort_hands(&mut self, world: &mut dyn PlanetaryWorld) -> bool;
    fn step_satchel_idle(&mut self, dt: f32);
    fn step_blackout(&mut self, dt: f32, world: &dyn PlanetaryWorld);
    fn revive_on_earth(&mut self, world: &dyn PlanetaryWorld);
    fn reconcile_frame(&mut self, world: &mut dyn PlanetaryWorld) -> bool;
}
