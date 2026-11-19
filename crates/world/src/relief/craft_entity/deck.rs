use glam::Vec3;
use crate::block::GridCell;
use super::blueprint::CraftBlueprint;
use super::identity::CraftIdentity;
use super::pose::CraftPose;

/// Ship-local deck reference frame for unseated astronauts experiencing thrust gravity or 0.0g float.
#[derive(Clone, Copy, Debug)]
pub struct DeckFrame {
    craft: CraftIdentity,
    pose: CraftPose,
    local_pos: Vec3,
    thrust_g: f32,
}

impl DeckFrame {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(craft: CraftIdentity, pose: CraftPose, local_pos: Vec3, thrust_g: f32) -> Self {
        Self { craft, pose, local_pos, thrust_g }
    }

    pub const fn craft(self) -> CraftIdentity {
        self.craft
    }

    pub const fn pose(self) -> CraftPose {
        self.pose
    }

    pub const fn local_pos(self) -> Vec3 {
        self.local_pos
    }

    pub const fn thrust_g(self) -> f32 {
        self.thrust_g
    }

    pub fn thrust_active(self) -> bool {
        self.thrust_g > 0.2
    }

    pub fn deck_up(self) -> Vec3 {
        self.pose.normal().normalize_or_zero()
    }

    /// Evaluates artificial thrust gravity (`m/s^2`) from `CraftPose` vertical main-thruster channel.
    pub fn thrust_from_pose(pose: CraftPose) -> f32 {
        let burn = pose.wheel_spin();
        if burn > 0.05 {
            (burn * 9.81).clamp(0.0, 14.0)
        } else {
            0.0
        }
    }

    /// Probes whether `world_pos` lies inside `blueprint`'s multi-deck cabin envelope at `pose`.
    pub fn probe(
        blueprint: &CraftBlueprint,
        pose: CraftPose,
        world_pos: Vec3,
        player_height: f32,
    ) -> Option<Self> {
        if !blueprint.spacefaring() {
            return None;
        }
        let local = pose.local_point_of(world_pos);
        let (u, v) = (local.x + 0.25, local.z + 0.25);
        let (cu, cv) = GridCell::at_planar(u, v).feed_coordinates(|gu, gv| (gu, gv));
        let foot_y = local.y - player_height;
        let lift = blueprint.wheel_ground_lift();
        let mut deck_below = false;
        let mut max_ceil = f32::NEG_INFINITY;
        for part in blueprint.parts() {
            if let Some((_, ceil_h)) = part.any_ceiling(lift) {
                max_ceil = max_ceil.max(ceil_h);
            }
            if let Some((floor_h, _)) = part.cell_top_at(cu, cv, lift) {
                if floor_h <= foot_y + 0.75 {
                    deck_below = true;
                }
            }
        }
        if !deck_below || foot_y > max_ceil + 3.0 {
            return None;
        }
        Some(Self::new(blueprint.identity(), pose, local, Self::thrust_from_pose(pose)))
    }
}
