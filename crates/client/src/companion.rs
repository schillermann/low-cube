use world::{CompanionPresence, MeshTarget, PlanetaryWorld};
use crate::attire::{BodyPose, StrideRhythm, YouthAttire, YouthPhysique};

/// 3D figure decorator rendering active multiplayer companions on the World-Cube and in Polaroids.
#[derive(Clone, Copy)]
pub struct CompanionFigure {
    presence: CompanionPresence,
}

impl CompanionFigure {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(presence: CompanionPresence) -> Self {
        Self { presence }
    }

    pub fn emit(self, target: &mut dyn MeshTarget) {
        self.presence.transfer(|_, kinematics, rhythm_raw, attire_bytes, region_bytes| {
            let attire = YouthAttire::from_multiplayer_wire(attire_bytes, region_bytes);
            let rhythm = StrideRhythm::with_dual_cleave(
                rhythm_raw.0,
                rhythm_raw.1,
                rhythm_raw.2,
                rhythm_raw.3,
            );
            let physique = YouthPhysique::from_region_bytes(region_bytes);
            kinematics.transfer(|pos, s_rot, v_rot, eye_h, leap| {
                let pose = BodyPose::with_leap_pose(pos, s_rot, v_rot, eye_h, leap, rhythm, physique);
                attire.emit_companion(&pose, target);
            });
        });
    }

    pub fn emit_all(world: &dyn PlanetaryWorld, target: &mut dyn MeshTarget) {
        let mut roster = Vec::new();
        world.active_companions(&mut roster);
        for companion in roster {
            Self::new(companion).emit(target);
        }
    }
}
