mod avatar;
mod dossier;
mod tape;

use glam::Vec3;
use world::{ClimateZone, CompanionPresence};

pub use self::avatar::BlackBoxAvatar;
#[cfg(test)]
pub use self::avatar::PresenceProbe;
pub use self::dossier::{DiskDossier, FileBlackBox};
#[cfg(test)]
pub use self::dossier::{IncidentReplay, MemoryDossier};
pub use self::tape::{LiveTape, SilentTape};

/// Sink receiving an avatar's living `CompanionPresence` without getters or matrix inversion.
pub trait PresenceSink {
    fn receive_presence(&mut self, presence: CompanionPresence);
}

/// Physical or topological anomaly detected on a simulation tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnomalyKind {
    CrustPenetration,
    NonFiniteKinematics,
    LethalCoreContact,
}

impl AnomalyKind {
    pub const fn code(self) -> u8 {
        match self {
            Self::CrustPenetration => 1,
            Self::NonFiniteKinematics => 2,
            Self::LethalCoreContact => 3,
        }
    }

    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::CrustPenetration),
            2 => Some(Self::NonFiniteKinematics),
            3 => Some(Self::LethalCoreContact),
            _ => None,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::CrustPenetration => "crust_penetration",
            Self::NonFiniteKinematics => "non_finite_kinematics",
            Self::LethalCoreContact => "lethal_core_contact",
        }
    }
}

/// Discrete interaction or topology milestone inscribed onto a `FlightTape`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlightEvent {
    LeftHandUsed,
    RightHandUsed,
    HandReleased,
    RigAdjusted,
    RelicCast,
    WorldLanded { blocks: u32 },
    Anomaly(AnomalyKind),
}

impl FlightEvent {
    pub fn summary(self) -> String {
        match self {
            Self::LeftHandUsed => "left_hand_used".to_string(),
            Self::RightHandUsed => "right_hand_used".to_string(),
            Self::HandReleased => "hand_released".to_string(),
            Self::RigAdjusted => "rig_adjusted".to_string(),
            Self::RelicCast => "relic_cast".to_string(),
            Self::WorldLanded { blocks } => format!("world_landed:{blocks}"),
            Self::Anomaly(kind) => format!("anomaly:{}", kind.label()),
        }
    }
}

/// Stack-allocated kinematic and control snapshot for one simulation tick (zero heap allocation).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TickTrace {
    position: Vec3,
    view_and_dt: (Vec3, f32),
    up_and_flags: (Vec3, u8),
    move_intent: (f32, f32),
}

impl TickTrace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        position: Vec3,
        view_and_dt: (Vec3, f32),
        up_and_flags: (Vec3, u8),
        move_intent: (f32, f32),
    ) -> Self {
        Self { position, view_and_dt, up_and_flags, move_intent }
    }

    pub const fn anomaly(self) -> Option<AnomalyKind> {
        AnomalyKind::from_code((self.up_and_flags.1 >> 4) & 0x0F)
    }

    pub fn climate_zone(self) -> ClimateZone {
        ClimateZone::dominant(self.position)
    }

    pub fn transfer<T>(
        self,
        f: impl FnOnce(Vec3, Vec3, f32, Vec3, u8, (f32, f32)) -> T,
    ) -> T {
        f(
            self.position,
            self.view_and_dt.0,
            self.view_and_dt.1,
            self.up_and_flags.0,
            self.up_and_flags.1,
            self.move_intent,
        )
    }
}

/// Printed-media destination receiving structured incident dossiers ("Tell, Don't Ask").
pub trait IncidentMedium {
    fn record_header(&mut self, world_blocks: u32, tick_count: usize, anomaly_count: u8);
    fn record_boundary(&mut self, section: &str, trace: TickTrace, zone: ClimateZone);
    fn record_event(&mut self, tick_offset: usize, event: FlightEvent);
    fn record_tick(&mut self, index: usize, trace: TickTrace);
}

/// Polymorphic flight black-box tape (`LiveTape` ring vs `SilentTape` Null Object).
pub trait FlightTape: Send + Sync {
    fn active(&self) -> bool;
    fn populated(&self) -> bool;
    fn summary_counts(&self) -> (usize, u8);
    fn record_tick(&mut self, world_blocks: u32, trace: TickTrace);
    fn record_event(&mut self, event: FlightEvent);
    fn toggled(&self) -> Box<dyn FlightTape>;
    fn inscribe_dossier(&self, medium: &mut dyn IncidentMedium) -> Option<u8>;
}
