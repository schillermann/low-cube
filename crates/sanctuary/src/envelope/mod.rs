mod pulse_codec;

use glam::Vec3;
use world::{
    CompanionPresence, CosmicFrame, CraftIdentity, DrivenCraft, PilotCraft, PlanetaryWorld,
    ProximityUtterance, SkySignal, VesselBerth, WildlifePulse, WorldFact, MAX_UTTERANCE_BYTES,
};
use self::pulse_codec::PulseCodec;

/// Transient block-loosening crack stage broadcast across P2P companions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrackNotice {
    eye: Vec3,
    direction: Vec3,
    stage: u8,
}

impl CrackNotice {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(eye: Vec3, direction: Vec3, stage: u8) -> Self {
        Self { eye, direction, stage }
    }

    pub const fn eye_point(self) -> Vec3 {
        self.eye
    }

    pub fn enact_on(self, world: &mut dyn PlanetaryWorld) -> bool {
        world.loosen_surface(self.eye, self.direction, self.stage).is_some()
    }

    pub fn near(self, anchor: Vec3, radius: f32) -> bool {
        (anchor - self.eye).length_squared() <= radius * radius
    }

    pub(super) fn inscribe(&self, out: &mut Vec<u8>) {
        WorldFact::Loosened {
            eye: self.eye,
            direction: self.direction,
            stage: self.stage,
        }
        .inscribe(out);
    }

    pub(super) fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        let (fact, used) = WorldFact::decode(bytes)?;
        match fact {
            WorldFact::Loosened { eye, direction, stage } => Some((Self::new(eye, direction, stage), used)),
            _ => None,
        }
    }
}

/// Authoritative client-reported crafting or provision consumption notice verified against `SatchelManifest`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryNotice {
    Crafted { recipe: u8 },
    Consumed { relic_code: u8 },
}

/// Encrypted domain message exchanged inside `SessionSeal` between `RemotePlanet` and `SanctuaryHost`.
#[derive(Clone, Debug, PartialEq)]
pub enum SanctuaryEnvelope {
    GuestPulse {
        ack_seq: u32,
        presence: CompanionPresence,
        craft: PilotCraft,
        frame: CosmicFrame,
        berth: VesselBerth,
        sky_watch: bool,
    },
    GuestPetition {
        fact: WorldFact,
    },
    HostChronicle {
        base_seq: u32,
        facts: Vec<WorldFact>,
    },
    HostPulse {
        total_facts: u32,
        cracks: Vec<CrackNotice>,
        companions: Vec<CompanionPresence>,
        crafts: Vec<DrivenCraft>,
        berths: Vec<(u32, CraftIdentity)>,
        signals: Vec<SkySignal>,
        wildlife: WildlifePulse,
    },
    GuestUtterance {
        text: String,
    },
    HostUtterance {
        utterance: ProximityUtterance,
    },
    GuestInventory {
        notice: InventoryNotice,
    },
}

impl SanctuaryEnvelope {
    pub fn inscribe(&self, out: &mut Vec<u8>) {
        out.clear();
        match self {
            Self::GuestPulse { ack_seq, presence, craft, frame, berth, sky_watch } => {
                PulseCodec::inscribe_guest(*ack_seq, presence, craft, *frame, *berth, *sky_watch, out);
            }
            Self::GuestPetition { fact } => {
                out.push(2);
                fact.inscribe(out);
            }
            Self::HostChronicle { base_seq, facts } => {
                out.push(3);
                out.extend_from_slice(&base_seq.to_le_bytes());
                out.extend_from_slice(&(facts.len() as u16).to_le_bytes());
                for fact in facts {
                    fact.inscribe(out);
                }
            }
            Self::HostPulse { total_facts, cracks, companions, crafts, berths, signals, wildlife } => {
                PulseCodec::inscribe_host(*total_facts, cracks, companions, crafts, berths, signals, *wildlife, out);
            }
            Self::GuestUtterance { text } => {
                out.push(5);
                let bytes = text.as_bytes();
                let len = bytes.len().min(MAX_UTTERANCE_BYTES);
                out.push(len as u8);
                out.extend_from_slice(&bytes[..len]);
            }
            Self::HostUtterance { utterance } => {
                out.push(6);
                utterance.inscribe(out);
            }
            Self::GuestInventory { notice } => {
                out.push(7);
                match *notice {
                    InventoryNotice::Crafted { recipe } => out.extend_from_slice(&[1, recipe]),
                    InventoryNotice::Consumed { relic_code } => out.extend_from_slice(&[2, relic_code]),
                }
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let tag = *bytes.first()?;
        match tag {
            1 => PulseCodec::decode_guest(bytes),
            2 if bytes.len() >= 2 => {
                let (fact, _) = WorldFact::decode(&bytes[1..])?;
                Some(Self::GuestPetition { fact })
            }
            3 if bytes.len() >= 7 => {
                let base_seq = u32::from_le_bytes(bytes[1..5].try_into().ok()?);
                let count = u16::from_le_bytes(bytes[5..7].try_into().ok()?) as usize;
                let mut cursor = 7;
                let mut facts = Vec::with_capacity(count.min(128));
                for _ in 0..count {
                    let (fact, used) = WorldFact::decode(&bytes[cursor..])?;
                    facts.push(fact);
                    cursor += used;
                }
                Some(Self::HostChronicle { base_seq, facts })
            }
            4 => PulseCodec::decode_host(bytes),
            5 if bytes.len() >= 2 => {
                let len = usize::from(bytes[1]);
                if len == 0 || len > MAX_UTTERANCE_BYTES || bytes.len() < 2 + len {
                    return None;
                }
                let raw = std::str::from_utf8(&bytes[2..2 + len]).ok()?;
                Some(Self::GuestUtterance { text: raw.to_string() })
            }
            6 if bytes.len() >= 18 => {
                let (utterance, _) = ProximityUtterance::decode(&bytes[1..])?;
                Some(Self::HostUtterance { utterance })
            }
            7 if bytes.len() >= 3 => match bytes[1] {
                1 => Some(Self::GuestInventory { notice: InventoryNotice::Crafted { recipe: bytes[2] } }),
                2 => Some(Self::GuestInventory { notice: InventoryNotice::Consumed { relic_code: bytes[2] } }),
                _ => None,
            },
            _ => None,
        }
    }
}
