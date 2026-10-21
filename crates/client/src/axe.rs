use glam::{Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{HeldRelic, LimbFrame};

/// Interface for an interactive world relic site that can be picked up into both hands and thrown by an avatar.
pub trait RelicSite {
    fn register_key(&mut self, code: KeyCode) -> bool;
    fn step_near_avatar(&mut self, avatar_pos: Vec3, surface_rot: Quat, view_rot: Quat, dt: f32, world: &dyn PlanetaryWorld) -> Option<(HeldRelic, HeldRelic)>;
    fn emit(&self, target: &mut dyn MeshTarget);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RelicCommand { Idle, PickupRight, PickupLeft, Throw }

#[derive(Clone, Copy, Debug)]
enum RelicPhase {
    Grounded { position: Vec3, surface_rot: Quat, yaw: f32, cooldown: f32 },
    Airborne { position: Vec3, velocity: Vec3, surface_rot: Quat, yaw: f32, spin: f32 },
    Wielded { side: f32 },
}

/// Single interactive medieval weapon (axe or sword) resting on the planetary surface.
#[derive(Clone, Copy, Debug)]
struct FieldRelic {
    relic: HeldRelic,
    phase: RelicPhase,
}

impl FieldRelic {
    fn new(relic: HeldRelic, phase: RelicPhase) -> Self {
        Self { relic, phase }
    }

    fn grounded_near_spawn(
        relic: HeldRelic,
        spawn_position: Vec3,
        lateral_offset: f32,
        forward_offset: f32,
        yaw: f32,
        world: &dyn PlanetaryWorld,
    ) -> Self {
        let up = world.surface_up_at(spawn_position);
        let rot = Quat::from_rotation_arc(Vec3::Y, up);
        let candidate = spawn_position + (rot * Vec3::X) * lateral_offset + (rot * Vec3::NEG_Z) * forward_offset;
        let grounded = world.constrain_to_surface(candidate, 0.0);
        let ground_up = world.surface_up_at(grounded);
        let surface_rot = Quat::from_rotation_arc(Vec3::Y, ground_up);
        Self::new(relic, RelicPhase::Grounded { position: grounded, surface_rot, yaw, cooldown: 0.0 })
    }

    fn emit_at(&self, position: Vec3, surface_rot: Quat, yaw: f32, pitch: f32, target: &mut dyn MeshTarget) {
        let (up, right, forward) = ((surface_rot * Vec3::Y).normalize(), (surface_rot * Vec3::X).normalize(), (surface_rot * Vec3::Z).normalize());
        let frame = LimbFrame::new(position + up * 0.18, right, up, forward).yawed(yaw).pitched(pitch);
        self.relic.emit_in_palm(1.0, &frame, target);
    }

    fn emit(&self, target: &mut dyn MeshTarget) {
        let ground_pitch = if matches!(self.relic, HeldRelic::KnightlySword) { -1.22 } else { -0.60 };
        match self.phase {
            RelicPhase::Grounded { position, surface_rot, yaw, .. } => self.emit_at(position, surface_rot, yaw, ground_pitch, target),
            RelicPhase::Airborne { position, surface_rot, yaw, spin, .. } => self.emit_at(position, surface_rot, yaw, spin, target),
            RelicPhase::Wielded { .. } => {}
        }
    }
}

/// Interactive medieval woodman's axe, knightly sword, and iron spade stationed near spawn.
pub struct FieldAxe {
    relics: [FieldRelic; 3],
    command: RelicCommand,
}

impl FieldAxe {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(relics: [FieldRelic; 3], command: RelicCommand) -> Self {
        Self { relics, command }
    }

    /// Secondary constructor placing the medieval axe, knightly sword, and iron spade on the ground in front of spawn.
    pub fn near_spawn(spawn_position: Vec3, world: &dyn PlanetaryWorld) -> Self {
        Self::new(
            [
                FieldRelic::grounded_near_spawn(HeldRelic::WoodmanAxe, spawn_position, 0.48, 1.55, 0.55, world),
                FieldRelic::grounded_near_spawn(HeldRelic::KnightlySword, spawn_position, -0.52, 1.62, -0.45, world),
                FieldRelic::grounded_near_spawn(HeldRelic::IronSpade, spawn_position, 0.0, 1.68, 0.15, world),
            ],
            RelicCommand::Idle,
        )
    }

    fn held_pair(&self) -> (HeldRelic, HeldRelic) {
        let right = self.relics.iter().find_map(|r| match r.phase { RelicPhase::Wielded { side } if side > 0.0 => Some(r.relic), _ => None }).unwrap_or(HeldRelic::EmptyPalm);
        let left = self.relics.iter().find_map(|r| match r.phase { RelicPhase::Wielded { side } if side < 0.0 => Some(r.relic), _ => None }).unwrap_or(HeldRelic::EmptyPalm);
        (right, left)
    }
}

impl RelicSite for FieldAxe {
    fn register_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::KeyE => {
                if self.relics.iter().any(|r| matches!(r.phase, RelicPhase::Grounded { .. })) {
                    self.command = RelicCommand::PickupRight;
                    return true;
                }
                false
            }
            KeyCode::KeyQ => {
                if self.relics.iter().any(|r| matches!(r.phase, RelicPhase::Grounded { .. })) {
                    self.command = RelicCommand::PickupLeft;
                    return true;
                }
                false
            }
            KeyCode::KeyX => {
                if self.relics.iter().any(|r| matches!(r.phase, RelicPhase::Wielded { .. })) {
                    self.command = RelicCommand::Throw;
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn step_near_avatar(
        &mut self,
        avatar_pos: Vec3,
        surface_rot: Quat,
        view_rot: Quat,
        dt: f32,
        world: &dyn PlanetaryWorld,
    ) -> Option<(HeldRelic, HeldRelic)> {
        let cmd = std::mem::replace(&mut self.command, RelicCommand::Idle);
        let up = (surface_rot * Vec3::Y).normalize();
        let (mut right_free, mut left_free) = (self.held_pair().0 == HeldRelic::EmptyPalm, self.held_pair().1 == HeldRelic::EmptyPalm);
        let mut changed = false;

        for idx in 0..self.relics.len() {
            if let RelicPhase::Grounded { position, surface_rot: r_rot, yaw, cooldown } = self.relics[idx].phase {
                let next_cd = (cooldown - dt).max(0.0);
                let delta = avatar_pos - position;
                let dist = (delta - up * delta.dot(up)).length();
                let interact = matches!(cmd, RelicCommand::PickupRight | RelicCommand::PickupLeft) && dist <= 2.60;
                if next_cd <= 0.0 && (dist <= 1.15 || interact) && !changed && (right_free || left_free) {
                    let side = if cmd == RelicCommand::PickupLeft && left_free {
                        left_free = false; -1.0
                    } else if right_free {
                        right_free = false; 1.0
                    } else {
                        left_free = false; -1.0
                    };
                    self.relics[idx].phase = RelicPhase::Wielded { side };
                    changed = true;
                } else {
                    self.relics[idx].phase = RelicPhase::Grounded { position, surface_rot: r_rot, yaw, cooldown: next_cd };
                }
            }
        }

        if changed {
            return Some(self.held_pair());
        }

        for item in &mut self.relics {
            match item.phase {
                RelicPhase::Airborne { position, velocity, surface_rot: r_rot, yaw, spin } => {
                    let next_vel = velocity + world.pull_direction_at(position) * (14.0 * dt);
                    let next_pos = position + next_vel * dt;
                    let ground = world.constrain_to_surface(next_pos, 0.0);
                    let g_up = world.surface_up_at(ground);
                    if (next_pos - ground).dot(g_up) <= 0.05 {
                        item.phase = RelicPhase::Grounded { position: ground, surface_rot: Quat::from_rotation_arc(Vec3::Y, g_up), yaw, cooldown: 0.55 };
                    } else {
                        item.phase = RelicPhase::Airborne { position: next_pos, velocity: next_vel, surface_rot: r_rot, yaw, spin: spin + 11.0 * dt };
                    }
                }
                RelicPhase::Wielded { side } if cmd == RelicCommand::Throw && !changed => {
                    let (view_fwd, right) = ((view_rot * Vec3::NEG_Z).normalize(), (view_rot * Vec3::X).normalize());
                    let launch_pos = avatar_pos + up * 0.25 + up.cross(right).normalize() * 0.65 + right * (0.18 * side);
                    item.phase = RelicPhase::Airborne { position: launch_pos, velocity: view_fwd * 5.8 + up * 2.1, surface_rot, yaw: 0.2, spin: 0.0 };
                    changed = true;
                }
                _ => {}
            }
        }
        if changed { Some(self.held_pair()) } else { None }
    }

    fn emit(&self, target: &mut dyn MeshTarget) {
        for item in &self.relics {
            item.emit(target);
        }
    }
}
