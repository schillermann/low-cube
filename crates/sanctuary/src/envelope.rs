use glam::Vec3;
use world::{CompanionPresence, WorldFact};

/// Polymorphic transient block-loosening crack stage without nullable fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrackNotice {
    Quiet,
    Active { eye: Vec3, direction: Vec3, stage: u8 },
}

impl CrackNotice {
    fn inscribe(&self, out: &mut Vec<u8>) {
        match *self {
            Self::Quiet => out.push(0),
            Self::Active { eye, direction, stage } => {
                out.push(1);
                WorldFact::Loosened { eye, direction, stage }.inscribe(out);
            }
        }
    }

    fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        match *bytes.first()? {
            0 => Some((Self::Quiet, 1)),
            1 => {
                let (fact, used) = WorldFact::decode(&bytes[1..])?;
                match fact {
                    WorldFact::Loosened { eye, direction, stage } => {
                        Some((Self::Active { eye, direction, stage }, 1 + used))
                    }
                    _ => None,
                }
            }
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
        crack: CrackNotice,
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
            Self::HostPulse { total_facts, crack, companions } => {
                out.push(4);
                out.extend_from_slice(&total_facts.to_le_bytes());
                crack.inscribe(out);
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
            4 if bytes.len() >= 6 => {
                let total_facts = u32::from_le_bytes(bytes[1..5].try_into().ok()?);
                let (crack, used_crack) = CrackNotice::decode(&bytes[5..])?;
                let mut cursor = 5 + used_crack;
                let count = *bytes.get(cursor)? as usize;
                cursor += 1;
                let mut companions = Vec::with_capacity(count);
                for _ in 0..count {
                    let (comp, used) = CompanionPresence::decode(&bytes[cursor..])?;
                    companions.push(comp);
                    cursor += used;
                }
                Some(Self::HostPulse { total_facts, crack, companions })
            }
            _ => None,
        }
    }
}
