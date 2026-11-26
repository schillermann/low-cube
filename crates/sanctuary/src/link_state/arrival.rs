use glam::Vec3;
use world::{CosmicFrame, SurvivorWelcome};

/// Polymorphic server-assigned arrival stance avoiding nullable `Option<Vec3>` fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WelcomeSpawn {
    Natural,
    Beacon(Vec3),
}

impl WelcomeSpawn {
    pub const fn from_optional(point: Option<Vec3>) -> Self {
        match point {
            Some(pt) => Self::Beacon(pt),
            None => Self::Natural,
        }
    }

    pub const fn assigned_point(self) -> Option<Vec3> {
        match self {
            Self::Natural => None,
            Self::Beacon(pt) => Some(pt),
        }
    }

    pub fn constrain_unstreamed(self, position: Vec3, player_height: f32, ground: Vec3) -> Vec3 {
        let Self::Beacon(beacon) = self else { return ground };
        if player_height <= 0.0 {
            return ground;
        }
        let zone = world::ClimateZone::dominant(beacon);
        let up = zone.normal();
        let plinth_top = beacon - up;
        let delta = position - plinth_top;
        let within_cell = delta.dot(zone.tangent()).abs() <= 0.25
            && delta.dot(zone.bitangent()).abs() <= 0.25;
        let stand = position + up * (plinth_top + up * player_height - position).dot(up);
        if within_cell && delta.dot(up) >= player_height - 0.30 && stand.dot(up) > ground.dot(up) {
            stand
        } else {
            ground
        }
    }
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

    pub fn constrain_unstreamed(&self, position: Vec3, player_height: f32, ground: Vec3) -> Vec3 {
        self.spawn.constrain_unstreamed(position, player_height, ground)
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
