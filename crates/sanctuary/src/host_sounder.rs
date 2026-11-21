use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld, SightCone, SounderPresence};
use fauna::BoarSounder;

/// Polymorphic pending tusk-knockback state avoiding `Option<Vec3>` fields.
#[derive(Clone, Copy, Debug)]
pub enum PendingShove {
    Calm,
    Gored(Vec3),
}

impl PendingShove {
    pub fn take(&mut self) -> Option<Vec3> {
        match std::mem::replace(self, Self::Calm) {
            Self::Calm => None,
            Self::Gored(impulse) => Some(impulse),
        }
    }
}

/// Polymorphic pending boar-snort cue queued for a co-op host listener.
#[derive(Clone, Copy, Debug)]
pub enum PendingSnort {
    Silent,
    Vocal(usize, Vec3),
}

impl PendingSnort {
    pub fn take(&mut self) -> Option<(usize, Vec3)> {
        match std::mem::replace(self, Self::Silent) {
            Self::Silent => None,
            Self::Vocal(idx, pos) => Some((idx, pos)),
        }
    }
}

/// Authoritative wild-boar sounder owned by `SanctuaryHost` and replicated to all companions.
pub struct HostSounder {
    sounder: BoarSounder,
    host_shove: PendingShove,
    host_snort: PendingSnort,
}

impl HostSounder {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sounder: BoarSounder, host_shove: PendingShove, host_snort: PendingSnort) -> Self {
        Self { sounder, host_shove, host_snort }
    }

    /// Secondary constructor initializing the authoritative Temperate boar sounder.
    pub const fn seeded() -> Self {
        Self::new(BoarSounder::seeded(), PendingShove::Calm, PendingSnort::Silent)
    }

    pub fn presence(&self) -> SounderPresence {
        self.sounder.presence()
    }

    /// Steps the authoritative sounder against all active human `observers`, returning any guest `(slot, shove)`.
    pub fn step_authoritative(
        &mut self,
        dt: f32,
        observers: &[(u32, Vec3, bool)],
        world: &dyn PlanetaryWorld,
    ) -> Option<(u32, Vec3)> {
        let (vocal, gored) = self.sounder.step_cohort(dt, observers, world);
        if let Some((idx, pos)) = vocal {
            self.host_snort = PendingSnort::Vocal(idx, pos);
        }
        if let Some((slot, shove)) = gored {
            if slot == 1 {
                self.host_shove = PendingShove::Gored(shove);
                None
            } else {
                Some((slot, shove))
            }
        } else {
            None
        }
    }

    /// Delivers queued spatial snort audio and tusk knockback to a local co-op host listener.
    pub fn step_host_listener(&mut self, listener: Vec3, view_dir: Vec3, world: &dyn PlanetaryWorld) -> Option<Vec3> {
        if let Some(vocal) = self.host_snort.take() {
            self.sounder.play_snort_near(vocal, listener, view_dir, world.surface_up_at(listener));
        }
        self.host_shove.take()
    }

    pub fn evaluate_strike(
        &mut self,
        eye: Vec3,
        view_dir: Vec3,
        power: u8,
        max_reach: f32,
        play_audio: bool,
        world: &dyn PlanetaryWorld,
    ) -> (bool, Option<([Vec3; 2], Vec3)>) {
        self.sounder.evaluate_strike(eye, view_dir, power, max_reach, play_audio, world)
    }

    pub fn emit_visible(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        self.sounder.emit_visible(cone, target);
    }
}
