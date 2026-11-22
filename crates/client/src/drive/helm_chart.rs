use glam::Vec3;
use world::{CelestialAtlas, OpticMount, PlanetaryWorld, VehicleAssembly};
use super::clearance::{HullClearance, PadClearance};
use super::transfer_arc::{FlipBurnReadout, NavigationCourse};

/// Immutable telemetry packet for the Nose-Camera 3D Steuerkonsolen-Orrery and target range scale.
#[derive(Clone, Copy, Debug)]
pub struct OrrerySnapshot {
    atlas: CelestialAtlas,
    ship_pos: Vec3,
    ship_motion: (Vec3, Vec3),
    course: NavigationCourse,
}

impl OrrerySnapshot {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        atlas: CelestialAtlas,
        ship_pos: Vec3,
        ship_motion: (Vec3, Vec3),
        course: NavigationCourse,
    ) -> Self {
        Self { atlas, ship_pos, ship_motion, course }
    }

    /// Transfers `(atlas, ship_pos, (nose_axis, velocity), course)` to a HUD view without getters.
    pub fn transfer<R>(
        self,
        f: impl FnOnce(CelestialAtlas, Vec3, (Vec3, Vec3), NavigationCourse) -> R,
    ) -> R {
        f(self.atlas, self.ship_pos, self.ship_motion, self.course)
    }
}

/// Immutable telemetry packet for the Belly-Camera ventral ground-clearance scale and target brackets.
#[derive(Clone, Copy, Debug)]
pub struct BellySnapshot {
    clearance: PadClearance,
    course: NavigationCourse,
    ship_pos: Vec3,
}

impl BellySnapshot {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(clearance: PadClearance, course: NavigationCourse, ship_pos: Vec3) -> Self {
        Self { clearance, course, ship_pos }
    }

    /// Transfers `(clearance, course, ship_pos)` to a HUD view without getters.
    pub fn transfer<R>(self, f: impl FnOnce(PadClearance, NavigationCourse, Vec3) -> R) -> R {
        f(self.clearance, self.course, self.ship_pos)
    }
}

/// Polymorphic hull-optic HUD telemetry eliminating `Option` across cockpit, nose, and belly channels.
#[derive(Clone, Copy, Debug)]
pub enum OpticTelemetry {
    Silent,
    Nose(OrrerySnapshot),
    Belly(BellySnapshot),
}

/// Encapsulated spacecraft celestial atlas, plotted `NavigationCourse`, and live ventral `PadClearance`.
#[derive(Clone, Copy, Debug)]
pub struct HelmChart {
    atlas: CelestialAtlas,
    course: NavigationCourse,
    pad: PadClearance,
}

impl HelmChart {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(atlas: CelestialAtlas, course: NavigationCourse, pad: PadClearance) -> Self {
        Self { atlas, course, pad }
    }

    pub fn boarded(assembly: &VehicleAssembly, world: &dyn PlanetaryWorld) -> Self {
        let atlas = world.celestial_atlas();
        let pad = HullClearance::at_pose(assembly, Vec3::ZERO).sample_pad(atlas, world);
        Self::new(atlas, NavigationCourse::Uncharted, pad)
    }

    pub fn install_course(&mut self, course: NavigationCourse) {
        self.course = course;
    }

    pub fn sync_flight(
        &mut self,
        assembly: &VehicleAssembly,
        velocity: Vec3,
        world: &dyn PlanetaryWorld,
    ) {
        self.atlas = world.celestial_atlas();
        self.course.sync_ephemeris(self.atlas);
        self.pad = HullClearance::at_pose(assembly, velocity).sample_pad(self.atlas, world);
    }

    pub fn mark_target(
        &mut self,
        rays: ((Vec3, Vec3), (Vec3, Vec3)),
        ship_state: (Vec3, Vec3),
        world: &dyn PlanetaryWorld,
    ) -> bool {
        self.atlas = world.celestial_atlas();
        self.course.mark_along(rays, ship_state, world)
    }

    pub fn cockpit_snapshot(&self, assembly: &VehicleAssembly, velocity: Vec3) -> OrrerySnapshot {
        let nose = assembly.normal().normalize_or_zero();
        OrrerySnapshot::new(self.atlas, assembly.origin(), (nose, velocity), self.course)
    }

    pub fn flip_burn_readout(&self, assembly: &VehicleAssembly, velocity: Vec3) -> FlipBurnReadout {
        match self.course {
            NavigationCourse::Uncharted => FlipBurnReadout::Inactive,
            NavigationCourse::Charted(ref arc) => {
                arc.sample_flip_burn(assembly.origin(), velocity, assembly.normal())
            }
        }
    }

    pub fn optic_telemetry(
        &self,
        mount: OpticMount,
        assembly: &VehicleAssembly,
        velocity: Vec3,
    ) -> OpticTelemetry {
        let pos = assembly.origin();
        match mount {
            OpticMount::RocketNose => OpticTelemetry::Nose(self.cockpit_snapshot(assembly, velocity)),
            OpticMount::RocketBelly => {
                OpticTelemetry::Belly(BellySnapshot::new(self.pad, self.course, pos))
            }
            _ => OpticTelemetry::Silent,
        }
    }
}
