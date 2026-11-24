use glam::Vec3;
use world::{CraftIdentity, CraftPose, PlanetaryWorld};
use crate::order::LocomotionOrder;

/// Zero-allocation ship-local deck footing tracking unseated crew across launchpad, Thrust Gravity, and 0.0g float.
#[derive(Clone, Copy, Debug)]
pub(super) enum DeckFooting {
    Planetary,
    Aboard {
        craft: CraftIdentity,
        local_pos: Vec3,
        cabin_drift: Vec3,
    },
}

impl DeckFooting {
    pub(super) const fn reset(&mut self) {
        *self = Self::Planetary;
    }

    pub(super) fn active_hull(&self, world: &dyn PlanetaryWorld) -> Option<(CraftIdentity, CraftPose)> {
        match *self {
            Self::Aboard { craft, .. } if craft.assigned() => {
                world.resolve_deck_pose(craft).map(|(pose, _)| (craft, pose))
            }
            _ => None,
        }
    }

    pub(super) fn step_aboard(
        &mut self,
        pos: Vec3,
        order: &LocomotionOrder,
        pace: f32,
        player_height: f32,
        dt: f32,
        world: &dyn PlanetaryWorld,
    ) -> Option<(Vec3, Vec3)> {
        let (craft, pose, thrust_g, carried_pos, prev_drift) = match *self {
            Self::Aboard { craft, local_pos, cabin_drift } => {
                if let Some((p, g)) = world.resolve_deck_pose(craft) {
                    (craft, p, g, p.world_point_at(local_pos), cabin_drift)
                } else if let Some(frame) = world.sample_deck_frame(pos, player_height) {
                    (frame.craft(), frame.pose(), frame.thrust_g(), pos, cabin_drift)
                } else {
                    *self = Self::Planetary;
                    return None;
                }
            }
            Self::Planetary => {
                let frame = world.sample_deck_frame(pos, player_height)?;
                (frame.craft(), frame.pose(), frame.thrust_g(), pos, Vec3::ZERO)
            }
        };

        let deck_up = pose.normal().normalize_or_zero();
        let on_crust = world.surface_contact(pose.origin(), 4.5).landable();
        let effective_g = if on_crust && thrust_g <= 0.2 {
            (-world.apply_pull(carried_pos, 1.0)).dot(deck_up).max(1.0)
        } else {
            thrust_g
        };

        let stride = order.surface_stride(deck_up, pace * dt);
        let (final_pos, next_drift) = if effective_g > 0.2 {
            let mut vy = prev_drift.dot(deck_up);
            let on_deck = world
                .constrain_on_craft(craft, carried_pos, player_height)
                .is_some_and(|sup| (carried_pos - sup).dot(deck_up).abs() <= 0.09);
            if order.jump_active() && on_deck {
                vy = 3.2 * (effective_g / 9.81).sqrt().clamp(0.55, 1.25);
            }
            vy = (vy - effective_g * dt).max(-18.0);
            let cand = carried_pos + stride + deck_up * (vy * dt);
            if let Some(sup) = world.constrain_on_craft(craft, cand, player_height) {
                if (sup - cand).dot(deck_up) >= -0.02 && vy <= 0.0 {
                    (sup, Vec3::ZERO)
                } else {
                    (cand, deck_up * vy)
                }
            } else {
                (cand, deck_up * vy)
            }
        } else {
            let cmd_v = if dt > 1e-6 { stride / dt } else { Vec3::ZERO }
                + if order.jump_active() { deck_up * 4.2 } else { Vec3::ZERO };
            let damped = prev_drift * (1.0 - (1.6 * dt).clamp(0.0, 1.0));
            let next_v = (damped + cmd_v * (3.5 * dt).clamp(0.0, 1.0)).clamp_length_max(10.0);
            let raw_cand = carried_pos + next_v * dt;
            if let Some(sup) = world.constrain_on_craft(craft, raw_cand, player_height) {
                if (sup - raw_cand).dot(deck_up) > 0.0 && next_v.dot(deck_up) < 0.0 {
                    (sup, next_v - deck_up * next_v.dot(deck_up))
                } else {
                    (raw_cand, next_v)
                }
            } else {
                (raw_cand, next_v)
            }
        };

        if let Some(next_frame) = world.sample_deck_frame(final_pos, player_height) {
            *self = Self::Aboard {
                craft: next_frame.craft(),
                local_pos: next_frame.local_pos(),
                cabin_drift: next_drift,
            };
        } else {
            *self = Self::Planetary;
        }
        Some((final_pos, deck_up))
    }
}
