use glam::Vec3;
use world::{CosmicFrame, SurvivorWelcome};

/// Polymorphic server-assigned arrival stance avoiding nullable `Option<Vec3>` fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WelcomeSpawn {
    Natural,
    Beacon(Vec3),
}

/// Polymorphic server orbital epoch handoff avoiding nullable `Option<f64>` fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EpochSync {
    Aligned,
    Pending(f64),
}

/// Polymorphic server survivor kit/checkpoint assignment avoiding nullable `Option<SurvivorWelcome>` fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WelcomeKit {
    LocalFallback,
    Assigned(SurvivorWelcome),
}

/// Polymorphic initial off-world frame anchor avoiding nullable `Option<(CosmicFrame, Vec3)>` fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FrameAnchor {
    Aligned,
    Pending(CosmicFrame, Vec3),
}

/// Encapsulated handshake arrival state uniting beacon spawn, orbital epoch, survivor kit, and frame anchor (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WelcomeArrival {
    spawn: WelcomeSpawn,
    epoch: EpochSync,
    kit: WelcomeKit,
    anchor: FrameAnchor,
}

impl WelcomeArrival {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        spawn: WelcomeSpawn,
        epoch: EpochSync,
        kit: WelcomeKit,
        anchor: FrameAnchor,
    ) -> Self {
        Self { spawn, epoch, kit, anchor }
    }

    pub const fn with_spawn(spawn: WelcomeSpawn) -> Self {
        Self::new(spawn, EpochSync::Aligned, WelcomeKit::LocalFallback, FrameAnchor::Aligned)
    }

    pub const fn natural() -> Self {
        Self::with_spawn(WelcomeSpawn::Natural)
    }

    pub const fn assigned_spawn(&self) -> Option<Vec3> {
        match self.spawn {
            WelcomeSpawn::Natural => None,
            WelcomeSpawn::Beacon(pt) => Some(pt),
        }
    }

    pub const fn assigned_survivor(&self) -> Option<SurvivorWelcome> {
        match self.kit {
            WelcomeKit::LocalFallback => None,
            WelcomeKit::Assigned(w) => Some(w),
        }
    }

    pub fn assign_spawn(&mut self, pt: Vec3) {
        self.spawn = WelcomeSpawn::Beacon(pt);
    }

    pub fn assign_epoch(&mut self, epoch_secs: f64) {
        if epoch_secs.is_finite() {
            self.epoch = EpochSync::Pending(epoch_secs);
        }
    }

    pub fn assign_survivor(&mut self, welcome: SurvivorWelcome) {
        self.kit = WelcomeKit::Assigned(welcome);
        if let Some(rec) = welcome.restored_record() {
            if !rec.frame().earth_centered() {
                self.anchor = FrameAnchor::Pending(rec.frame(), rec.position());
            }
        }
    }

    pub fn take_pending_epoch(&mut self) -> Option<f64> {
        match std::mem::replace(&mut self.epoch, EpochSync::Aligned) {
            EpochSync::Aligned => None,
            EpochSync::Pending(ep) => Some(ep),
        }
    }

    pub fn take_pending_anchor(&mut self) -> Option<(CosmicFrame, Vec3)> {
        match std::mem::replace(&mut self.anchor, FrameAnchor::Aligned) {
            FrameAnchor::Aligned => None,
            FrameAnchor::Pending(f, p) => Some((f, p)),
        }
    }
}
