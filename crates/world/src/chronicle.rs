use glam::Vec3;
use crate::block::{BlockMedium, BlockMold};
use crate::relief::ExposedStratum;
use crate::tree::SurfaceFind;
use crate::PlanetaryWorld;

/// Authoritative planetary mutation or newly air-exposed subterranean stratum fact.
#[derive(Clone, Copy, Debug, PartialEq)]
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
}

impl WorldFact {
    /// Applies this authoritative fact directly to a planetary world, returning whether geometry changed.
    pub fn enact_on(&self, world: &mut dyn PlanetaryWorld) -> bool {
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
        }
    }

    pub const fn durable(&self) -> bool {
        !matches!(self, Self::Loosened { .. })
    }

    fn push_vec3(v: Vec3, out: &mut Vec<u8>) {
        out.extend_from_slice(&v.x.to_le_bytes());
        out.extend_from_slice(&v.y.to_le_bytes());
        out.extend_from_slice(&v.z.to_le_bytes());
    }

    fn read_vec3(bytes: &[u8]) -> Option<Vec3> {
        if bytes.len() < 12 {
            return None;
        }
        Some(Vec3::new(
            f32::from_le_bytes(bytes[0..4].try_into().ok()?),
            f32::from_le_bytes(bytes[4..8].try_into().ok()?),
            f32::from_le_bytes(bytes[8..12].try_into().ok()?),
        ))
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        match *self {
            Self::Loosened { eye, direction, stage } => {
                out.push(1);
                Self::push_vec3(eye, out);
                Self::push_vec3(direction, out);
                out.push(stage);
            }
            Self::Harvested { eye, direction } => {
                out.push(2);
                Self::push_vec3(eye, out);
                Self::push_vec3(direction, out);
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
                Self::push_vec3(eye, out);
                Self::push_vec3(direction, out);
                Self::push_vec3(avatar_pos, out);
                out.extend_from_slice(&mold.chronicle_bytes());
            }
            Self::TimberStruck { eye, direction } => {
                out.push(5);
                Self::push_vec3(eye, out);
                Self::push_vec3(direction, out);
            }
            Self::Foraged { eye, direction } => {
                out.push(6);
                Self::push_vec3(eye, out);
                Self::push_vec3(direction, out);
            }
            Self::RigAdjusted { eye, direction, delta_steps, rotate_quarter } => {
                out.push(7);
                Self::push_vec3(eye, out);
                Self::push_vec3(direction, out);
                out.extend_from_slice(&delta_steps.to_le_bytes());
                out.push(u8::from(rotate_quarter));
            }
            Self::RigLowered { eye, direction } => {
                out.push(8);
                Self::push_vec3(eye, out);
                Self::push_vec3(direction, out);
            }
            Self::LitterCast { eye, direction, find } => {
                out.push(9);
                Self::push_vec3(eye, out);
                Self::push_vec3(direction, out);
                out.push(find.code());
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        let tag = *bytes.first()?;
        match tag {
            1 if bytes.len() >= 26 => Some((
                Self::Loosened {
                    eye: Self::read_vec3(&bytes[1..13])?,
                    direction: Self::read_vec3(&bytes[13..25])?,
                    stage: bytes[25],
                },
                26,
            )),
            2 if bytes.len() >= 25 => Some((
                Self::Harvested {
                    eye: Self::read_vec3(&bytes[1..13])?,
                    direction: Self::read_vec3(&bytes[13..25])?,
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
                    eye: Self::read_vec3(&bytes[1..13])?,
                    direction: Self::read_vec3(&bytes[13..25])?,
                    avatar_pos: Self::read_vec3(&bytes[25..37])?,
                    mold: BlockMold::from_chronicle_bytes([bytes[37], bytes[38], bytes[39]]),
                },
                40,
            )),
            5 if bytes.len() >= 25 => Some((
                Self::TimberStruck {
                    eye: Self::read_vec3(&bytes[1..13])?,
                    direction: Self::read_vec3(&bytes[13..25])?,
                },
                25,
            )),
            6 if bytes.len() >= 25 => Some((
                Self::Foraged {
                    eye: Self::read_vec3(&bytes[1..13])?,
                    direction: Self::read_vec3(&bytes[13..25])?,
                },
                25,
            )),
            7 if bytes.len() >= 30 => Some((
                Self::RigAdjusted {
                    eye: Self::read_vec3(&bytes[1..13])?,
                    direction: Self::read_vec3(&bytes[13..25])?,
                    delta_steps: i32::from_le_bytes(bytes[25..29].try_into().ok()?),
                    rotate_quarter: bytes[29] != 0,
                },
                30,
            )),
            8 if bytes.len() >= 25 => Some((
                Self::RigLowered {
                    eye: Self::read_vec3(&bytes[1..13])?,
                    direction: Self::read_vec3(&bytes[13..25])?,
                },
                25,
            )),
            9 if bytes.len() >= 26 => {
                let find = SurfaceFind::from_code(bytes[25]);
                find.gathered().then_some((
                    Self::LitterCast {
                        eye: Self::read_vec3(&bytes[1..13])?,
                        direction: Self::read_vec3(&bytes[13..25])?,
                        find,
                    },
                    26,
                ))
            }
            _ => None,
        }
    }
}
