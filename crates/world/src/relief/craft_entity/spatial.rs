use glam::Vec3;
use crate::block::{BlockMedium, ConsoleOrrery, GridCell, WedgeHeading};
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::super::sight_ray::SurfaceSightRay;
use super::super::vehicle_assembly::VehicleAssembly;
use super::{CraftBlueprint, CraftPose};

/// 6-DOF spatial hull view uniting a `CraftBlueprint` with a world-space `CraftPose`.
#[derive(Clone, Copy)]
pub(in crate::relief) struct PosedHull<'a> {
    blueprint: &'a CraftBlueprint,
    pose: CraftPose,
}

impl<'a> PosedHull<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(in crate::relief) const fn new(blueprint: &'a CraftBlueprint, pose: CraftPose) -> Self {
        Self { blueprint, pose }
    }

    pub(in crate::relief) fn emit_local(blueprint: &CraftBlueprint, target: &mut dyn MeshTarget) {
        let local_frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let lift = blueprint.wheel_ground_lift();
        let (top_col, wall_col) = (Vec3::new(0.45, 0.30, 0.17), Vec3::new(0.41, 0.27, 0.15));
        for part in blueprint.parts() {
            if !part.kinematic_part() {
                part.emit_driven(lift, 0.0, 0.0, &local_frame, top_col, wall_col, target);
            }
        }
    }

    pub(in crate::relief) fn emit_kinematic(self, target: &mut dyn MeshTarget) {
        let car_frame = self.pose.surface_frame();
        let shift = self.pose.origin() - car_frame.point_at(0.25, 0.25, 0.0);
        let lift = self.blueprint.wheel_ground_lift();
        let space = self.blueprint.spacefaring();
        let com = if space { self.blueprint.mass_center(lift, &car_frame) } else { Vec3::ZERO };
        let order = super::super::vehicle_assembly::PilotOrder::from_pose_motion(
            self.pose.wheel_spin(),
            self.pose.steer_turn(),
        );
        let (rcs_lin, rcs_torque) = if space {
            self.blueprint.rcs_demands(&order, self.pose, &car_frame)
        } else {
            (Vec3::ZERO, Vec3::ZERO)
        };
        {
            let mut shifted = super::blueprint::ShiftedMeshTarget::new(shift, target);
            let (top_col, wall_col) = (Vec3::new(0.45, 0.30, 0.17), Vec3::new(0.41, 0.27, 0.15));
            for part in self.blueprint.parts() {
                if part.kinematic_part()
                    && (!space
                        || !part.emit_space_kinematic(
                            lift,
                            &car_frame,
                            com,
                            &order,
                            rcs_lin,
                            rcs_torque,
                            &mut shifted,
                        ))
                {
                    part.emit_driven(
                        lift,
                        self.pose.wheel_spin(),
                        self.pose.steer_turn(),
                        &car_frame,
                        top_col,
                        wall_col,
                        &mut shifted,
                    );
                }
            }
        }
        for part in self.blueprint.parts() {
            if let Some(base) = part.console_base_at(lift, &car_frame) {
                ConsoleOrrery::new(self.pose.origin(), shift + base, (self.pose.heading(), self.pose.normal())).emit(target);
            }
        }
    }

    pub(in crate::relief) fn hit_ceiling_at(self, world_pt: Vec3) -> Option<Vec3> {
        let local = self.pose.local_point_of(world_pt);
        let (car_u, car_v, car_elev) = (local.x + 0.25, local.z + 0.25, local.y);
        let (cu, cv) = GridCell::at_planar(car_u, car_v).feed_coordinates(|gu, gv| (gu, gv));
        let lift = self.blueprint.wheel_ground_lift();
        let mut best: Option<f32> = None;
        for part in self.blueprint.parts() {
            if let Some((floor_h, ceil_h)) = part.cell_top_at(cu, cv, lift) {
                if car_elev >= floor_h - 0.02 && car_elev <= ceil_h + 0.02 {
                    best = Some(best.map_or(ceil_h, |cur| cur.max(ceil_h)));
                }
            }
        }
        let ceil = best?;
        let car_frame = self.pose.surface_frame();
        let shift = self.pose.origin() - car_frame.point_at(0.25, 0.25, 0.0);
        Some(shift + car_frame.point_at(car_u, car_v, ceil))
    }

    pub(in crate::relief) fn medium_at_world(self, world_pt: Vec3) -> Option<BlockMedium> {
        let local = self.pose.local_point_of(world_pt);
        let (cu, cv) = GridCell::at_planar(local.x + 0.25, local.z + 0.25).feed_coordinates(|gu, gv| (gu, gv));
        let lift = self.blueprint.wheel_ground_lift();
        self.blueprint.parts().iter().rev().find_map(|part| {
            let (floor_h, ceil_h) = part.cell_top_at(cu, cv, lift)?;
            (local.y >= floor_h - 0.02 && local.y <= ceil_h + 0.02).then(|| part.block_medium())
        })
    }

    pub(in crate::relief) fn touches_world_floor(self, cell_floor_world: Vec3) -> bool {
        let local = self.pose.local_point_of(cell_floor_world);
        let (car_u, car_v, car_floor) = (local.x + 0.25, local.z + 0.25, local.y);
        let lift = self.blueprint.wheel_ground_lift();
        self.blueprint.parts().iter().any(|part| {
            let Some((floor_h, _)) = part.any_ceiling(lift) else { return false };
            let mut b = (0, 0, 0, 0);
            part.expand_span((1, 0, 0, 1), &mut b);
            let (px, py) = GridCell::new(b.0 + b.1, b.2 + b.3).center();
            (car_u - px).abs() + (car_v - py).abs() <= 0.65 && (car_floor - floor_h).abs() <= 0.62
        })
    }

    pub(in crate::relief) fn backrest_world_at(self, world_pt: Vec3) -> Option<Vec3> {
        let local = self.pose.local_point_of(world_pt);
        let (cu, cv) = GridCell::at_planar(local.x + 0.25, local.z + 0.25).feed_coordinates(|gu, gv| (gu, gv));
        let lift = self.blueprint.wheel_ground_lift();
        let heading = self.blueprint.parts().iter().rev().find_map(|part| {
            let (off, _, h) = part.seat_slot()?;
            let (floor_h, ceil_h) = part.cell_top_at(cu, cv, lift)?;
            (off == (cu, cv) && local.y >= floor_h - 0.02 && local.y <= ceil_h + 0.02).then_some(h)
        })?;
        let (du, dv) = match heading {
            WedgeHeading::East => (-1, 0),
            WedgeHeading::West => (1, 0),
            WedgeHeading::North => (0, -1),
            WedgeHeading::South => (0, 1),
        };
        let (bx, by) = GridCell::new(cu + du, cv + dv).center();
        let car_frame = self.pose.surface_frame();
        let shift = self.pose.origin() - car_frame.point_at(0.25, 0.25, 0.0);
        Some(shift + car_frame.point_at(bx, by, local.y))
    }

    pub(in crate::relief) fn targets_helm(self, hit_world: Vec3, ray: SurfaceSightRay) -> bool {
        let target_med = if self.blueprint.spacefaring() {
            BlockMedium::FlightConsole
        } else if self.blueprint.parts().iter().any(|p| p.block_medium() == BlockMedium::SteeringWheel) {
            BlockMedium::SteeringWheel
        } else {
            return false;
        };
        if self.medium_at_world(hit_world) == Some(target_med) {
            return true;
        }
        let lift = self.blueprint.wheel_ground_lift();
        (1..=28).any(|step| {
            let local = self.pose.local_point_of(ray.sample_point((step as f32) * 0.06));
            let (cu, cv) = GridCell::at_planar(local.x + 0.25, local.z + 0.25).feed_coordinates(|gu, gv| (gu, gv));
            self.blueprint.parts().iter().any(|p| {
                p.block_medium() == target_med
                    && p.cell_top_at(cu, cv, lift)
                        .is_some_and(|(fl, ce)| local.y >= fl - 0.15 && local.y <= ce + 1.15)
            })
        })
    }

    pub(in crate::relief) fn perch_at(self, hit_world: Vec3, eye_world: Vec3) -> Option<VehicleAssembly> {
        if self.medium_at_world(hit_world) != Some(BlockMedium::CabinSeat) {
            return None;
        }
        let (_, seat_world, _) = self.nearest_seat_to(eye_world)?;
        Some(VehicleAssembly::new(seat_world, self.pose.axes(), self.pose.heading(), Vec::new()))
    }

    pub(in crate::relief) fn nearest_seat_to(self, eye_world: Vec3) -> Option<((i32, i32), Vec3, f32)> {
        let lift = self.blueprint.wheel_ground_lift();
        let car_frame = self.pose.surface_frame();
        let shift = self.pose.origin() - car_frame.point_at(0.25, 0.25, 0.0);
        let mut best: Option<((i32, i32), Vec3, f32)> = None;
        for part in self.blueprint.parts() {
            let Some((off, floor_off, _)) = part.seat_slot() else { continue };
            let (cx, cy) = GridCell::new(off.0, off.1).center();
            let seat_world = shift + car_frame.point_at(cx, cy, floor_off + lift);
            let dist = (seat_world - eye_world).length();
            if best.is_none_or(|(_, _, cur)| dist < cur) {
                best = Some((off, seat_world, dist));
            }
        }
        best
    }

    pub(in crate::relief) fn tripod_hub(self) -> Option<(Vec3, Vec3, Vec3)> {
        let car_frame = self.pose.surface_frame();
        let shift = self.pose.origin() - car_frame.point_at(0.25, 0.25, 0.0);
        let lift = self.blueprint.wheel_ground_lift();
        self.blueprint.parts().iter().find_map(|p| {
            let (hub, fwd, up) = p.tripod_hub_at(lift, &car_frame)?;
            Some((shift + hub, fwd, up))
        })
    }

    pub(in crate::relief) fn visit_soles(self, mut visitor: impl FnMut(Vec3)) {
        let car_frame = self.pose.surface_frame();
        let shift = self.pose.origin() - car_frame.point_at(0.25, 0.25, 0.0);
        let lift = self.blueprint.wheel_ground_lift();
        for part in self.blueprint.parts() {
            visitor(shift + part.world_sole_center(lift, &car_frame));
        }
    }

    pub(in crate::relief) fn passenger_seat_along_ray(self, eye: Vec3, direction: Vec3) -> Option<Vec3> {
        let dir = direction.normalize_or_zero();
        if dir.length_squared() < 1e-4 {
            return None;
        }
        let lift = self.blueprint.wheel_ground_lift();
        let mut best: Option<(Vec3, f32)> = None;
        for part in self.blueprint.parts() {
            let Some((ox, oy, floor_off)) = part.passenger_seat_slot() else { continue };
            let local = Vec3::new((ox as f32) * 0.5, floor_off + lift, (oy as f32) * 0.5);
            let (world_seat, _, up) = self.pose.world_seat_at(local);
            let target_pt = world_seat + up * 0.25;
            let to_seat = target_pt - eye;
            let along = to_seat.dot(dir);
            if !(0.15..=4.2).contains(&along) {
                continue;
            }
            let radial_sq = (to_seat - dir * along).length_squared();
            if radial_sq <= 0.45 && best.is_none_or(|(_, d)| along < d) {
                best = Some((local, along));
            }
        }
        best.map(|(local, _)| local)
    }

    pub(in crate::relief) fn constrain_at(self, position: Vec3, player_height: f32) -> Option<Vec3> {
        if self.blueprint.parts().is_empty() {
            return None;
        }
        let car_frame = self.pose.surface_frame();
        let shift = self.pose.origin() - car_frame.point_at(0.25, 0.25, 0.0);
        let rel = position - shift;
        let (u, v) = car_frame.decompose(rel);
        let elev = car_frame.elevation_of(rel);
        let (cu, cv) = GridCell::at_planar(u, v).feed_coordinates(|gu, gv| (gu, gv));
        let lift = self.blueprint.wheel_ground_lift();
        let foot_elev = elev - player_height;
        let mut top_support: Option<f32> = None;
        for part in self.blueprint.parts() {
            let Some((floor_h, ceil_h)) = part.cell_top_at(cu, cv, lift) else { continue };
            let passes_under = player_height > 1e-4 && foot_elev + 0.08 <= floor_h && floor_h >= 1.77;
            if !passes_under {
                top_support = Some(top_support.map_or(ceil_h, |cur| cur.max(ceil_h)));
            }
        }
        top_support.map(|h| shift + car_frame.point_at(u, v, h + player_height))
    }
}
