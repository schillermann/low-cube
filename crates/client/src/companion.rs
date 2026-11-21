use world::{CompanionPortal, CompanionPresence, MeshTarget};
#[cfg(test)]
use world::PlanetaryWorld;
use crate::attire::{BodyPose, MoldedWardrobe, SightMode, StrideRhythm, Wardrobe, YouthPhysique};

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
        self.presence.transfer(|_, kinematics, rhythm_raw, presence_attire| {
            let wardrobe = MoldedWardrobe::from_presence(presence_attire);
            let rhythm = StrideRhythm::with_dual_cleave(
                rhythm_raw.0,
                rhythm_raw.1,
                rhythm_raw.2,
                rhythm_raw.3,
            );
            let physique = presence_attire.transfer(|_, regions, _| {
                YouthPhysique::from_region_bytes(regions)
            });
            kinematics.transfer(|pos, s_rot, v_rot, eye_h, leap| {
                let pose = BodyPose::with_leap_pose(pos, s_rot, v_rot, eye_h, leap, rhythm, physique);
                wardrobe.emit(&pose, SightMode::LookingGlassSight, target);
            });
        });
    }

    pub fn emit_figures(portal: &dyn CompanionPortal, target: &mut dyn MeshTarget) {
        let mut roster = Vec::new();
        portal.active_companions(&mut roster);
        for companion in roster {
            Self::new(companion).emit(target);
        }
    }

    #[cfg(test)]
    pub fn emit_all(world: &dyn PlanetaryWorld, target: &mut dyn MeshTarget) {
        world.emit_active_crafts(target);
        Self::emit_figures(world, target);
    }
}
