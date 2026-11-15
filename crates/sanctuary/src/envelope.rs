use glam::Vec3;
use world::{CompanionPresence, PlanetaryWorld, WorldFact};

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

    pub fn enact_on(self, world: &mut dyn PlanetaryWorld) -> bool {
        world.loosen_surface(self.eye, self.direction, self.stage).is_some()
    }

    fn inscribe(&self, out: &mut Vec<u8>) {
        WorldFact::Loosened {
            eye: self.eye,
            direction: self.direction,
            stage: self.stage,
        }
        .inscribe(out);
    }

    fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        let (fact, used) = WorldFact::decode(bytes)?;
        match fact {
            WorldFact::Loosened { eye, direction, stage } => Some((Self::new(eye, direction, stage), used)),
            _ => None,
        }
    }
}

/// Encrypted domain message exchanged inside `SessionSeal` between `RemotePlanet` and `SanctuaryHost`.
#[derive(Clone, Debug, PartialEq)]
pub enum SanctuaryEnvelope {
    GuestPulse {
        ack_seq: u32,
        presence: CompanionPresence,
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
    },
}

impl SanctuaryEnvelope {
    pub fn inscribe(&self, out: &mut Vec<u8>) {
        out.clear();
        match self {
            Self::GuestPulse { ack_seq, presence } => {
                out.push(1);
                out.extend_from_slice(&ack_seq.to_le_bytes());
                presence.inscribe(out);
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
            Self::HostPulse { total_facts, cracks, companions } => {
                out.push(4);
                out.extend_from_slice(&total_facts.to_le_bytes());
                out.push(cracks.len().min(16) as u8);
                for crack in cracks.iter().take(16) {
                    crack.inscribe(out);
                }
                out.push(companions.len().min(64) as u8);
                for comp in companions.iter().take(64) {
                    comp.inscribe(out);
                }
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let tag = *bytes.first()?;
        match tag {
            1 if bytes.len() >= 5 => {
                let ack_seq = u32::from_le_bytes(bytes[1..5].try_into().ok()?);
                let (presence, _) = CompanionPresence::decode(&bytes[5..])?;
                Some(Self::GuestPulse { ack_seq, presence })
            }
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
            4 if bytes.len() >= 7 => {
                let total_facts = u32::from_le_bytes(bytes[1..5].try_into().ok()?);
                let crack_count = *bytes.get(5)? as usize;
                let mut cursor = 6;
                let mut cracks = Vec::with_capacity(crack_count.min(16));
                for _ in 0..crack_count {
                    let (crack, used) = CrackNotice::decode(&bytes[cursor..])?;
                    cracks.push(crack);
                    cursor += used;
                }
                let comp_count = *bytes.get(cursor)? as usize;
                cursor += 1;
                let mut companions = Vec::with_capacity(comp_count.min(64));
                for _ in 0..comp_count {
                    let (comp, used) = CompanionPresence::decode(&bytes[cursor..])?;
                    companions.push(comp);
                    cursor += used;
                }
                Some(Self::HostPulse { total_facts, cracks, companions })
            }
            _ => None,
        }
    }
}
