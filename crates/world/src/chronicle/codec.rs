use crate::block::{BlockMedium, BlockMold};
use crate::relief::{CraftBlueprint, CraftIdentity, CraftPose, ExposedStratum};
use crate::tree::SurfaceFind;
use crate::wire_vector::{WireSlice, WireVector};
use super::WorldFact;

/// Binary serializer encoding an authoritative `WorldFact` into compact little-endian bytes.
pub(super) struct FactCodec<'a> {
    fact: &'a WorldFact,
}

impl<'a> FactCodec<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(fact: &'a WorldFact) -> Self {
        Self { fact }
    }

    pub(super) fn inscribe(&self, out: &mut Vec<u8>) {
        match *self.fact {
            WorldFact::Loosened { eye, direction, stage } => {
                out.push(1);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
                out.push(stage);
            }
            WorldFact::Harvested { eye, direction } => {
                out.push(2);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
            }
            WorldFact::StratumDisclosed { face, stratum } => {
                out.push(3);
                out.push(face);
                stratum.transfer(|u, v, step, med| {
                    out.extend_from_slice(&u.to_le_bytes());
                    out.extend_from_slice(&v.to_le_bytes());
                    out.extend_from_slice(&step.to_le_bytes());
                    out.push(med.code());
                });
            }
            WorldFact::MoldPlaced { eye, direction, avatar_pos, mold } => {
                out.push(4);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
                WireVector::new(avatar_pos).inscribe(out);
                out.extend_from_slice(&mold.chronicle_bytes());
            }
            WorldFact::TimberStruck { eye, direction } => {
                out.push(5);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
            }
            WorldFact::Foraged { eye, direction } => {
                out.push(6);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
            }
            WorldFact::RigAdjusted { eye, direction, delta_steps, rotate_quarter } => {
                out.push(7);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
                out.extend_from_slice(&delta_steps.to_le_bytes());
                out.push(u8::from(rotate_quarter));
            }
            WorldFact::RigLowered { eye, direction } => {
                out.push(8);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
            }
            WorldFact::LitterCast { eye, direction, find } => {
                out.push(9);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
                out.push(find.code());
            }
            WorldFact::CraftCommissioned { ref blueprint } => {
                out.push(10);
                blueprint.inscribe(out);
            }
            WorldFact::HelmClaimed { craft, pilot_slot, eye, direction } => {
                out.push(11);
                craft.inscribe(out);
                out.extend_from_slice(&pilot_slot.to_le_bytes());
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
            }
            WorldFact::HelmReleased { craft, pose } => {
                out.push(12);
                craft.inscribe(out);
                pose.inscribe(out);
            }
            WorldFact::RigBlueprintBuilt { eye, direction, ref blueprint } => {
                out.push(13);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
                blueprint.inscribe(out);
            }
            WorldFact::BoarStruck { eye, direction, power, reach_cm } => {
                out.push(14);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
                out.push(power);
                out.extend_from_slice(&reach_cm.to_le_bytes());
            }
        }
    }
}

/// Binary slice parser reconstructing a `WorldFact` and its consumed byte span.
pub(super) struct FactBytes<'a> {
    bytes: &'a [u8],
}

impl<'a> FactBytes<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub(super) fn decode(&self) -> Option<(WorldFact, usize)> {
        let bytes = self.bytes;
        let tag = *bytes.first()?;
        match tag {
            1 if bytes.len() >= 26 => Some((
                WorldFact::Loosened {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                    stage: bytes[25],
                },
                26,
            )),
            2 if bytes.len() >= 25 => Some((
                WorldFact::Harvested {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                },
                25,
            )),
            3 if bytes.len() >= 15 => {
                let face = bytes[1];
                let u = i32::from_le_bytes(bytes[2..6].try_into().ok()?);
                let v = i32::from_le_bytes(bytes[6..10].try_into().ok()?);
                let step = i32::from_le_bytes(bytes[10..14].try_into().ok()?);
                let med = BlockMedium::from_code(bytes[14]);
                Some((WorldFact::StratumDisclosed { face, stratum: ExposedStratum::new((u, v, step), med) }, 15))
            }
            4 if bytes.len() >= 40 => Some((
                WorldFact::MoldPlaced {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                    avatar_pos: WireSlice::new(&bytes[25..37]).vec3()?,
                    mold: BlockMold::from_chronicle_bytes([bytes[37], bytes[38], bytes[39]]),
                },
                40,
            )),
            5 if bytes.len() >= 25 => Some((
                WorldFact::TimberStruck {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                },
                25,
            )),
            6 if bytes.len() >= 25 => Some((
                WorldFact::Foraged {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                },
                25,
            )),
            7 if bytes.len() >= 30 => Some((
                WorldFact::RigAdjusted {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                    delta_steps: i32::from_le_bytes(bytes[25..29].try_into().ok()?),
                    rotate_quarter: bytes[29] != 0,
                },
                30,
            )),
            8 if bytes.len() >= 25 => Some((
                WorldFact::RigLowered {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                },
                25,
            )),
            9 if bytes.len() >= 26 => {
                let find = SurfaceFind::from_code(bytes[25]);
                find.gathered().then_some((
                    WorldFact::LitterCast {
                        eye: WireSlice::new(&bytes[1..13]).vec3()?,
                        direction: WireSlice::new(&bytes[13..25]).vec3()?,
                        find,
                    },
                    26,
                ))
            }
            10 if bytes.len() >= 11 => {
                let (blueprint, used) = CraftBlueprint::decode(&bytes[1..])?;
                Some((WorldFact::CraftCommissioned { blueprint }, 1 + used))
            }
            11 if bytes.len() >= 37 => {
                let (craft, _) = CraftIdentity::decode(&bytes[1..9])?;
                let pilot_slot = u32::from_le_bytes(bytes[9..13].try_into().ok()?);
                let eye = WireSlice::new(&bytes[13..25]).vec3()?;
                let direction = WireSlice::new(&bytes[25..37]).vec3()?;
                Some((WorldFact::HelmClaimed { craft, pilot_slot, eye, direction }, 37))
            }
            12 if bytes.len() >= 77 => {
                let (craft, _) = CraftIdentity::decode(&bytes[1..9])?;
                let (pose, _) = CraftPose::decode(&bytes[9..77])?;
                Some((WorldFact::HelmReleased { craft, pose }, 77))
            }
            13 if bytes.len() >= 35 => {
                let eye = WireSlice::new(&bytes[1..13]).vec3()?;
                let direction = WireSlice::new(&bytes[13..25]).vec3()?;
                let (blueprint, used) = CraftBlueprint::decode(&bytes[25..])?;
                Some((WorldFact::RigBlueprintBuilt { eye, direction, blueprint }, 25 + used))
            }
            14 if bytes.len() >= 28 => {
                let eye = WireSlice::new(&bytes[1..13]).vec3()?;
                let direction = WireSlice::new(&bytes[13..25]).vec3()?;
                let power = bytes[25];
                let reach_cm = u16::from_le_bytes([bytes[26], bytes[27]]);
                Some((WorldFact::BoarStruck { eye, direction, power, reach_cm }, 28))
            }
            _ => None,
        }
    }
}
