mod codec;
mod snapshot;

use glam::Vec3;
use crate::block::BlockMold;
use crate::celestial::BodyId;
use crate::relief::{CraftBlueprint, CraftIdentity, CraftPose, ExposedStratum};
use crate::tree::SurfaceFind;
use crate::WorldEarthworks;
use self::codec::{FactBytes, FactCodec};
pub use self::snapshot::{
    RigSnapshot, SectorColumnState, SectorSnapshot, SnapshotBlock, NATURAL_OVERHANG_STAMP,
    SNAPSHOT_SECTOR_CELLS,
};

/// Authoritative planetary mutation, vehicle ownership event, or newly air-exposed subterranean stratum fact.
#[derive(Clone, Debug, PartialEq)]
pub enum WorldFact {
    Loosened { eye: Vec3, direction: Vec3, stage: u8 },
    Harvested { eye: Vec3, direction: Vec3 },
    StratumDisclosed { face: u8, stratum: ExposedStratum },
    MoldPlaced { eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold },
    ModulePlaced { eye: Vec3, direction: Vec3, avatar_pos: Vec3, spec: crate::ModuleSpec },
    TimberStruck { eye: Vec3, direction: Vec3 },
    Foraged { eye: Vec3, direction: Vec3 },
    RigAdjusted { eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool },
    RigLowered { eye: Vec3, direction: Vec3 },
    LitterCast { eye: Vec3, direction: Vec3, find: SurfaceFind },
    CraftCommissioned { blueprint: CraftBlueprint },
    HelmClaimed { craft: CraftIdentity, pilot_slot: u32, eye: Vec3, direction: Vec3 },
    HelmReleased { craft: CraftIdentity, pose: CraftPose },
    RigBlueprintBuilt { eye: Vec3, direction: Vec3, blueprint: CraftBlueprint },
    OffWorld { body: BodyId, fact: Box<WorldFact> },
    SectorSnapshot { snapshot: SectorSnapshot },
}

impl WorldFact {
    /// Shifts spatial anchors (`eye`, `avatar_pos`, `pose`) by `shift` for body-local / world-space conversion.
    pub fn translated_by(&self, shift: Vec3) -> Self {
        match *self {
            Self::Loosened { eye, direction, stage } => Self::Loosened { eye: eye + shift, direction, stage },
            Self::Harvested { eye, direction } => Self::Harvested { eye: eye + shift, direction },
            Self::StratumDisclosed { face, stratum } => Self::StratumDisclosed { face, stratum },
            Self::MoldPlaced { eye, direction, avatar_pos, mold } => {
                Self::MoldPlaced { eye: eye + shift, direction, avatar_pos: avatar_pos + shift, mold }
            }
            Self::ModulePlaced { eye, direction, avatar_pos, spec } => {
                Self::ModulePlaced { eye: eye + shift, direction, avatar_pos: avatar_pos + shift, spec }
            }
            Self::TimberStruck { eye, direction } => Self::TimberStruck { eye: eye + shift, direction },
            Self::Foraged { eye, direction } => Self::Foraged { eye: eye + shift, direction },
            Self::RigAdjusted { eye, direction, delta_steps, rotate_quarter } => {
                Self::RigAdjusted { eye: eye + shift, direction, delta_steps, rotate_quarter }
            }
            Self::RigLowered { eye, direction } => Self::RigLowered { eye: eye + shift, direction },
            Self::LitterCast { eye, direction, find } => Self::LitterCast { eye: eye + shift, direction, find },
            Self::CraftCommissioned { ref blueprint } => Self::CraftCommissioned { blueprint: blueprint.clone() },
            Self::HelmClaimed { craft, pilot_slot, eye, direction } => {
                Self::HelmClaimed { craft, pilot_slot, eye: eye + shift, direction }
            }
            Self::HelmReleased { craft, pose } => {
                Self::HelmReleased { craft, pose: pose.with_origin(pose.origin() + shift) }
            }
            Self::RigBlueprintBuilt { eye, direction, ref blueprint } => {
                Self::RigBlueprintBuilt { eye: eye + shift, direction, blueprint: blueprint.clone() }
            }
            Self::OffWorld { body, ref fact } => Self::OffWorld { body, fact: fact.clone() },
            Self::SectorSnapshot { ref snapshot } => Self::SectorSnapshot { snapshot: snapshot.clone() },
        }
    }

    /// Scopes a world-space mutation to a body-local `OffWorld` fact when `frame` identifies an off-world cube.
    pub fn scoped_to(self, frame: Option<(BodyId, Vec3)>) -> Self {
        match frame {
            Some((body, center)) => Self::OffWorld { body, fact: Box::new(self.translated_by(-center)) },
            None => self,
        }
    }

    /// Applies this authoritative fact directly to a planetary earthworks target, returning whether geometry changed.
    pub fn enact_on<W: WorldEarthworks + ?Sized>(&self, world: &mut W) -> bool {
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
            Self::ModulePlaced { eye, direction, avatar_pos, spec } => {
                world.place_scaled_surface_module(eye, direction, avatar_pos, spec)
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
            Self::OffWorld { body, ref fact } => world.enact_offworld(body, fact),
            Self::SectorSnapshot { ref snapshot } => world.apply_sector_snapshot(snapshot),
        }
    }

    /// True when this fact mutates `TerracedRelief` cells/rigs/strata and folds into a `SectorSnapshot`.
    pub fn snapshot_foldable(&self) -> bool {
        match self {
            Self::Harvested { .. }
            | Self::StratumDisclosed { .. }
            | Self::MoldPlaced { .. }
            | Self::ModulePlaced { .. }
            | Self::RigAdjusted { .. }
            | Self::RigLowered { .. }
            | Self::RigBlueprintBuilt { .. }
            | Self::SectorSnapshot { .. } => true,
            Self::OffWorld { fact, .. } => fact.snapshot_foldable(),
            _ => false,
        }
    }

    pub fn durable(&self) -> bool {
        match self {
            Self::OffWorld { fact, .. } => fact.durable(),
            Self::Loosened { .. } => false,
            _ => true,
        }
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        FactCodec::new(self).inscribe(out);
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        FactBytes::new(bytes).decode()
    }
}
