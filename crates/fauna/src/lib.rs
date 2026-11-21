mod planet;
mod roe_deer;
mod sparrow;
mod wild_boar;

#[cfg(test)]
mod allocation_audit;
#[cfg(test)]
#[global_allocator]
static ALLOCATOR: allocation_audit::HeapLedger = allocation_audit::HeapLedger::new();
#[cfg(test)]
mod roe_deer_tests;
#[cfg(test)]
mod sparrow_tests;
#[cfg(test)]
mod wild_boar_tests;

use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld};
pub use world::SightCone;
pub use self::planet::{AmbientWoodland, WoodlandPlanet};
pub use self::roe_deer::{RoeActivity, RoeHerd, TestRoeDeer};
pub use self::sparrow::{SparrowActivity, SparrowFlock, TestSparrow};
pub use self::wild_boar::{BoarActivity, BoarSounder, RemoteSounder, TestWildBoar};

/// Composite woodland wildlife population combining house sparrows, Temperate roe deer, and wild boars.
#[derive(Clone, Copy, Debug)]
pub struct WoodlandFauna {
    sparrows: SparrowFlock,
    roe_herd: RoeHerd,
    boar_sounder: BoarSounder,
}

impl WoodlandFauna {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sparrows: SparrowFlock, roe_herd: RoeHerd, boar_sounder: BoarSounder) -> Self {
        Self { sparrows, roe_herd, boar_sounder }
    }

    /// Secondary constructor seeding woodland sparrows, Temperate roe deer, and Temperate wild boars.
    pub const fn seeded() -> Self {
        Self::new(SparrowFlock::seeded(), RoeHerd::seeded(), BoarSounder::seeded())
    }

    pub const fn boars(&self) -> &BoarSounder {
        &self.boar_sounder
    }

    /// Steps sparrows, roe deer, and wild boars, returning any wild-boar tusk knockback impulse.
    pub fn step_spatial(
        &mut self,
        dt: f32,
        listener: Vec3,
        view_dir: Vec3,
        crouching: bool,
        world: &dyn PlanetaryWorld,
    ) -> Option<Vec3> {
        if !world.woodland_habitat(listener) {
            return None;
        }
        let _ = self.sparrows.step_spatial(dt, listener, view_dir, world);
        let _ = self.roe_herd.step_spatial(dt, listener, view_dir, crouching, world);
        self.boar_sounder.step_spatial(dt, listener, view_dir, crouching, world)
    }

    /// Strikes any wild boar within `max_reach` along `(eye, view_dir)`, returning `(hit_boar, dropped_meat)`.
    pub fn strike_boar(
        &mut self,
        eye: Vec3,
        view_dir: Vec3,
        power: u8,
        max_reach: f32,
        world: &mut dyn PlanetaryWorld,
    ) -> (bool, bool) {
        self.boar_sounder.strike_boar(eye, view_dir, power, max_reach, world)
    }

    /// Emits 3D faceted meshes only for wildlife inside `cone`.
    pub fn emit_visible(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        self.sparrows.emit_visible(cone, target);
        self.roe_herd.emit_visible(cone, target);
        self.boar_sounder.emit_visible(cone, target);
    }
}
