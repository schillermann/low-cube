use glam::{Quat, Vec3};
use crate::block::{BlockMedium, CardinalTurn};
use crate::brick_arena::{BrickCell, BrickStructure, BrickVolume, CellCoord};
use crate::relief::{CraftIdentity, CraftPose};
use super::motion::{BodyMotion, DockedLink};

/// Physical `2×2×1` `DockingRing` collar exposed on the exterior of a `GridBody` (`4` strictly private fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DockingPort {
    body: CraftIdentity,
    world_center: Vec3,
    outward_normal: Vec3,
    local_center: Vec3,
}

impl DockingPort {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        body: CraftIdentity,
        world_center: Vec3,
        outward_normal: Vec3,
        local_center: Vec3,
    ) -> Self {
        Self { body, world_center, outward_normal, local_center }
    }

    pub const fn body(self) -> CraftIdentity {
        self.body
    }

    pub const fn world_center(self) -> Vec3 {
        self.world_center
    }

    pub const fn outward_normal(self) -> Vec3 {
        self.outward_normal
    }

    pub const fn local_center(self) -> Vec3 {
        self.local_center
    }

    /// Collects all `2×2×1` `DockingRing` ports mounted on `grid` transformed by `pose`.
    pub fn collect_from(
        body: CraftIdentity,
        grid: &BrickStructure,
        pose: CraftPose,
        out: &mut Vec<Self>,
    ) {
        let datum = grid.datum();
        let hull_rot = pose.hull_rotation();
        grid.bricks().inner().visit_occupied_cells(|coord, cell| {
            let (turn_code, is_ring) = match cell {
                BrickCell::ModuleAnchor(spec) if spec.medium() == BlockMedium::DockingRing => {
                    (spec.turn().code(), true)
                }
                BrickCell::RawAtom(atom) if atom.medium() == BlockMedium::DockingRing => {
                    (atom.heading().code(), true)
                }
                _ => (0, false),
            };
            if !is_ring {
                return;
            }
            let local_pt = Self::local_port_center(coord, datum, cell);
            let local_n = Self::local_outward_normal(grid, coord, CardinalTurn::from_code(turn_code));
            let world_pt = pose.world_point_at(local_pt);
            let world_n = (hull_rot * local_n).normalize_or_zero();
            out.push(Self::new(body, world_pt, world_n, local_pt));
        });
    }

    fn local_port_center(coord: CellCoord, datum: f32, cell: BrickCell) -> Vec3 {
        let (cu, cv) = coord.planar();
        let floor = coord.floor_elev(datum);
        match cell {
            BrickCell::ModuleAnchor(spec) => {
                let (du, dv, dh) = spec.span().dimensions();
                Vec3::new(
                    (cu as f32) * 0.5 + (du as f32) * 0.25 - 0.25,
                    floor + (dh as f32) * 0.25,
                    (cv as f32) * 0.5 + (dv as f32) * 0.25 - 0.25,
                )
            }
            _ => Vec3::new((cu as f32) * 0.5, floor + 0.25, (cv as f32) * 0.5),
        }
    }

    fn local_outward_normal(grid: &BrickStructure, coord: CellCoord, turn: CardinalTurn) -> Vec3 {
        let above_free = !grid.bricks().inner().cell_at(coord.shifted(0, 0, 1)).occupied();
        let below_free = !grid.bricks().inner().cell_at(coord.shifted(0, 0, -1)).occupied();
        if above_free && !below_free {
            Vec3::Y
        } else if below_free && !above_free {
            Vec3::NEG_Y
        } else {
            match turn {
                CardinalTurn::Forward => Vec3::NEG_Z,
                CardinalTurn::Right => Vec3::X,
                CardinalTurn::Backward => Vec3::Z,
                CardinalTurn::Left => Vec3::NEG_X,
            }
        }
    }
}

/// Magnetic `2×2×1` `DockingRing` capture clamp conserving combined Steiner momentum (`2` fields).
#[derive(Clone, Copy, Debug)]
pub struct DockingClamp {
    capture_range: f32,
    max_closing_speed: f32,
}

impl DockingClamp {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(capture_range: f32, max_closing_speed: f32) -> Self {
        Self { capture_range, max_closing_speed }
    }

    pub const fn orbital() -> Self {
        Self::new(1.50, 5.0)
    }

    pub fn try_couple(
        self,
        host_port: DockingPort,
        host_motion: BodyMotion,
        host_mass: f32,
        guest_port: DockingPort,
        guest_motion: BodyMotion,
        guest_mass: f32,
    ) -> Option<(BodyMotion, DockedLink, BodyMotion)> {
        if !host_port.body().assigned()
            || !guest_port.body().assigned()
            || host_port.body() == guest_port.body()
        {
            return None;
        }
        let gap = (guest_port.world_center() - host_port.world_center()).length();
        let alignment = host_port.outward_normal().dot(guest_port.outward_normal());
        let rel_speed = (guest_motion.linear_vel() - host_motion.linear_vel()).length();
        if gap > self.capture_range || alignment > -0.72 || rel_speed > self.max_closing_speed {
            return None;
        }

        let m_h = host_mass.max(100.0);
        let m_g = guest_mass.max(100.0);
        let inv_total = 1.0 / (m_h + m_g);
        let merged_vel = (host_motion.linear_vel() * m_h + guest_motion.linear_vel() * m_g) * inv_total;
        let merged_omega = (host_motion.angular_vel() * m_h + guest_motion.angular_vel() * m_g) * inv_total;
        let next_host_motion = host_motion.with_velocity(merged_vel, merged_omega);

        let align_rot = Quat::from_rotation_arc(guest_port.outward_normal(), -host_port.outward_normal());
        let rotated_guest_pose = guest_motion.pose().rotated(align_rot);
        let target_port_world = host_port.world_center() + host_port.outward_normal() * 0.25;
        let guest_port_world_after_rot = rotated_guest_pose.world_point_at(guest_port.local_center());
        let snapped_guest_origin = rotated_guest_pose.origin() + (target_port_world - guest_port_world_after_rot);
        let snapped_guest_pose = rotated_guest_pose.with_origin(snapped_guest_origin);

        let host_pose = next_host_motion.pose();
        let local_offset = host_pose.local_point_of(snapped_guest_origin);
        let local_rot = (host_pose.hull_rotation().inverse() * snapped_guest_pose.hull_rotation()).normalize();
        let link = DockedLink::new(host_port.body(), local_offset, local_rot);
        let synced_guest = link.synced_motion(next_host_motion, snapped_guest_pose);
        Some((next_host_motion, link, synced_guest))
    }
}
