use glam::Vec3;
use crate::block::{BlockMedium, BlockMold};
use crate::relief::{CraftBlueprint, CraftIdentity, CraftPose, ExposedStratum};
use crate::tree::SurfaceFind;
use crate::WorldEarthworks;

/// Authoritative planetary mutation, vehicle ownership event, or newly air-exposed subterranean stratum fact.
#[derive(Clone, Debug, PartialEq)]
pub enum WorldFact {
    Loosened { eye: Vec3, direction: Vec3, stage: u8 },
    Harvested { eye: Vec3, direction: Vec3 },
    StratumDisclosed { face: u8, stratum: ExposedStratum },
    MoldPlaced { eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold },
    TimberStruck { eye: Vec3, direction: Vec3 },
    Foraged { eye: Vec3, direction: Vec3 },
    RigAdjusted { eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool },
    RigLowered { eye: Vec3, direction: Vec3 },
    LitterCast { eye: Vec3, direction: Vec3, find: SurfaceFind },
    CraftCommissioned { blueprint: CraftBlueprint },
    HelmClaimed { craft: CraftIdentity, pilot_slot: u32, eye: Vec3, direction: Vec3 },
    HelmReleased { craft: CraftIdentity, pose: CraftPose },
    RigBlueprintBuilt { eye: Vec3, direction: Vec3, blueprint: CraftBlueprint },
}

impl WorldFact {
    /// Applies this authoritative fact directly to a planetary earthworks target, returning whether geometry changed.
    pub fn enact_on(&self, world: &mut dyn WorldEarthworks) -> bool {
        match *self {
            Self::Loosened { eye, direction, stage } => world.loosen_surface(eye, direction, stage).is_some(),
            Self::Harvested { eye, direction } => world.harvest_surface(eye, direction).is_some(),
            Self::StratumDisclosed { face, stratum } => {
                world.disclose_stratum(face as usize, stratum);
                true
            }
            Self::MoldPlaced { eye, direction, avatar_pos, mold } => {
                world.place_molded_surface_block(eye, direction, avatar_pos, mold)
            }
            Self::TimberStruck { eye, direction } => world.strike_timber(eye, direction).struck(),
            Self::Foraged { eye, direction } => world.forage_surface(eye, direction).gathered(),
            Self::RigAdjusted { eye, direction, delta_steps, rotate_quarter } => {
                world.adjust_rig(eye, direction, delta_steps, rotate_quarter)
            }
            Self::RigLowered { eye, direction } => world.lower_rig(eye, direction),
            Self::LitterCast { eye, direction, find } => world.cast_surface_find(eye, direction, find),
            Self::CraftCommissioned { ref blueprint } => world.register_craft_blueprint(blueprint.clone()),
            Self::HelmClaimed { craft, pilot_slot, eye, direction } => {
                world.claim_vehicle_helm(craft, pilot_slot, eye, direction)
            }
            Self::HelmReleased { craft, pose } => world.park_craft_by_id(craft, pose),
            Self::RigBlueprintBuilt { eye, direction, ref blueprint } => {
                world.build_rig_blueprint(eye, direction, blueprint)
            }
        }
    }

    pub const fn durable(&self) -> bool {
        !matches!(self, Self::Loosened { .. })
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        use crate::wire_vector::WireVector;
        match *self {
            Self::Loosened { eye, direction, stage } => {
                out.push(1);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
                out.push(stage);
            }
            Self::Harvested { eye, direction } => {
                out.push(2);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
            }
            Self::StratumDisclosed { face, stratum } => {
                out.push(3);
                out.push(face);
                stratum.transfer(|u, v, step, med| {
                    out.extend_from_slice(&u.to_le_bytes());
                    out.extend_from_slice(&v.to_le_bytes());
                    out.extend_from_slice(&step.to_le_bytes());
                    out.push(med.code());
                });
            }
            Self::MoldPlaced { eye, direction, avatar_pos, mold } => {
                out.push(4);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
                WireVector::new(avatar_pos).inscribe(out);
                out.extend_from_slice(&mold.chronicle_bytes());
            }
            Self::TimberStruck { eye, direction } => {
                out.push(5);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
            }
            Self::Foraged { eye, direction } => {
                out.push(6);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
            }
            Self::RigAdjusted { eye, direction, delta_steps, rotate_quarter } => {
                out.push(7);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
                out.extend_from_slice(&delta_steps.to_le_bytes());
                out.push(u8::from(rotate_quarter));
            }
            Self::RigLowered { eye, direction } => {
                out.push(8);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
            }
            Self::LitterCast { eye, direction, find } => {
                out.push(9);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
                out.push(find.code());
            }
            Self::CraftCommissioned { ref blueprint } => {
                out.push(10);
                blueprint.inscribe(out);
            }
            Self::HelmClaimed { craft, pilot_slot, eye, direction } => {
                out.push(11);
                craft.inscribe(out);
                out.extend_from_slice(&pilot_slot.to_le_bytes());
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
            }
            Self::HelmReleased { craft, pose } => {
                out.push(12);
                craft.inscribe(out);
                pose.inscribe(out);
            }
            Self::RigBlueprintBuilt { eye, direction, ref blueprint } => {
                out.push(13);
                WireVector::new(eye).inscribe(out);
                WireVector::new(direction).inscribe(out);
                blueprint.inscribe(out);
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        use crate::wire_vector::WireSlice;
        let tag = *bytes.first()?;
        match tag {
            1 if bytes.len() >= 26 => Some((
                Self::Loosened {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                    stage: bytes[25],
                },
                26,
            )),
            2 if bytes.len() >= 25 => Some((
                Self::Harvested {
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
                Some((Self::StratumDisclosed { face, stratum: ExposedStratum::new((u, v, step), med) }, 15))
            }
            4 if bytes.len() >= 40 => Some((
                Self::MoldPlaced {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                    avatar_pos: WireSlice::new(&bytes[25..37]).vec3()?,
                    mold: BlockMold::from_chronicle_bytes([bytes[37], bytes[38], bytes[39]]),
                },
                40,
            )),
            5 if bytes.len() >= 25 => Some((
                Self::TimberStruck {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                },
                25,
            )),
            6 if bytes.len() >= 25 => Some((
                Self::Foraged {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                },
                25,
            )),
            7 if bytes.len() >= 30 => Some((
                Self::RigAdjusted {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                    delta_steps: i32::from_le_bytes(bytes[25..29].try_into().ok()?),
                    rotate_quarter: bytes[29] != 0,
                },
                30,
            )),
            8 if bytes.len() >= 25 => Some((
                Self::RigLowered {
                    eye: WireSlice::new(&bytes[1..13]).vec3()?,
                    direction: WireSlice::new(&bytes[13..25]).vec3()?,
                },
                25,
            )),
            9 if bytes.len() >= 26 => {
                let find = SurfaceFind::from_code(bytes[25]);
                find.gathered().then_some((
                    Self::LitterCast {
                        eye: WireSlice::new(&bytes[1..13]).vec3()?,
                        direction: WireSlice::new(&bytes[13..25]).vec3()?,
                        find,
                    },
                    26,
                ))
            }
            10 if bytes.len() >= 11 => {
                let (blueprint, used) = CraftBlueprint::decode(&bytes[1..])?;
                Some((Self::CraftCommissioned { blueprint }, 1 + used))
            }
            11 if bytes.len() >= 37 => {
                let (craft, _) = CraftIdentity::decode(&bytes[1..9])?;
                let pilot_slot = u32::from_le_bytes(bytes[9..13].try_into().ok()?);
                let eye = WireSlice::new(&bytes[13..25]).vec3()?;
                let direction = WireSlice::new(&bytes[25..37]).vec3()?;
                Some((Self::HelmClaimed { craft, pilot_slot, eye, direction }, 37))
            }
            12 if bytes.len() >= 77 => {
                let (craft, _) = CraftIdentity::decode(&bytes[1..9])?;
                let (pose, _) = CraftPose::decode(&bytes[9..77])?;
                Some((Self::HelmReleased { craft, pose }, 77))
            }
            13 if bytes.len() >= 35 => {
                let eye = WireSlice::new(&bytes[1..13]).vec3()?;
                let direction = WireSlice::new(&bytes[13..25]).vec3()?;
                let (blueprint, used) = CraftBlueprint::decode(&bytes[25..])?;
                Some((Self::RigBlueprintBuilt { eye, direction, blueprint }, 25 + used))
            }
            _ => None,
        }
    }
}
