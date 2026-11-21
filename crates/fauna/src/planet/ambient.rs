use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld, SightCone};
use crate::{RoeHerd, SparrowFlock, WoodlandFauna};

/// Focused 3-method wildlife realm contract decoupling fauna simulation from planetary pass-throughs.
pub trait FaunaRealm: Send {
    fn strike_realm(
        &mut self,
        eye: Vec3,
        direction: Vec3,
        power: u8,
        max_reach: f32,
        world: &mut dyn PlanetaryWorld,
    ) -> (bool, bool);

    fn step_realm(
        &mut self,
        dt: f32,
        listener: Vec3,
        view_dir: Vec3,
        crouching: bool,
        world: &mut dyn PlanetaryWorld,
    ) -> Option<Vec3>;

    fn emit_realm(&self, cone: SightCone, world: &dyn PlanetaryWorld, target: &mut dyn MeshTarget);
}

impl FaunaRealm for WoodlandFauna {
    fn strike_realm(
        &mut self,
        eye: Vec3,
        direction: Vec3,
        power: u8,
        max_reach: f32,
        world: &mut dyn PlanetaryWorld,
    ) -> (bool, bool) {
        self.strike_boar(eye, direction, power, max_reach, world)
    }

    fn step_realm(
        &mut self,
        dt: f32,
        listener: Vec3,
        view_dir: Vec3,
        crouching: bool,
        world: &mut dyn PlanetaryWorld,
    ) -> Option<Vec3> {
        self.step_spatial(dt, listener, view_dir, crouching, world)
    }

    fn emit_realm(&self, cone: SightCone, _world: &dyn PlanetaryWorld, target: &mut dyn MeshTarget) {
        self.emit_visible(cone, target);
    }
}

/// Ambient co-op wildlife realm layering local sparrows and roe deer over synchronized world boars.
pub struct AmbientRealm {
    sparrows: SparrowFlock,
    roe_herd: RoeHerd,
}

impl AmbientRealm {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sparrows: SparrowFlock, roe_herd: RoeHerd) -> Self {
        Self { sparrows, roe_herd }
    }

    pub const fn seeded() -> Self {
        Self::new(SparrowFlock::seeded(), RoeHerd::seeded())
    }
}

impl FaunaRealm for AmbientRealm {
    fn strike_realm(
        &mut self,
        eye: Vec3,
        direction: Vec3,
        power: u8,
        max_reach: f32,
        world: &mut dyn PlanetaryWorld,
    ) -> (bool, bool) {
        world.strike_boar(eye, direction, power, max_reach)
    }

    fn step_realm(
        &mut self,
        dt: f32,
        listener: Vec3,
        view_dir: Vec3,
        crouching: bool,
        world: &mut dyn PlanetaryWorld,
    ) -> Option<Vec3> {
        if world.woodland_habitat(listener) {
            let _ = self.sparrows.step_spatial(dt, listener, view_dir, world);
            let _ = self.roe_herd.step_spatial(dt, listener, view_dir, crouching, world);
        }
        world.step_wildlife(dt, listener, view_dir, crouching)
    }

    fn emit_realm(&self, cone: SightCone, world: &dyn PlanetaryWorld, target: &mut dyn MeshTarget) {
        self.sparrows.emit_visible(cone, target);
        self.roe_herd.emit_visible(cone, target);
        world.emit_wildlife(cone, target);
    }
}
