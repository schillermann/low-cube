mod fleet;

use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, ElevatedShape, GridCell, VoxelShape, WedgeHeading};
use crate::frame::SurfaceFrame;
use super::molded_structure::{MoldedStructure, VoxelCells, WorldBlocks};
use super::sight_ray::SurfaceSightRay;
use super::turf_scars::TurfScars;
use super::vehicle_assembly::{VehicleAssembly, VehicleAxes, VehiclePart};
pub use self::fleet::ParkedFleet;

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Independent multi-block object constructed on a cantilever crane and optionally driven and parked.
pub struct CraftCluster {
    grid: VoxelCells,
}

impl CraftCluster {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(grid: VoxelCells) -> Self {
        Self { grid }
    }

    pub fn untouched() -> Self {
        Self::new(VoxelCells::untouched())
    }

    pub fn insert(&mut self, u: i32, v: i32, floor_elev: f32, shape: Box<dyn VoxelShape>) {
        self.grid.insert(u, v, floor_elev, shape);
    }

    pub fn step_camera_turn(&mut self, u: i32, v: i32, delta_steps: i32) -> Option<(i32, i32)> {
        self.grid.step_camera_turn(u, v, delta_steps)
    }

    pub fn remove_layer(&mut self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.grid.remove_at(u, v, layer_elev).map(|(b, _)| b.block_medium())
    }

    pub fn occupies_claw(&self, claw: (i32, i32), hoist_z: f32) -> bool {
        self.grid.any_at(claw.0, claw.1, |b| (b.floor() - hoist_z).abs() <= BLOCK_STEP + 0.12)
    }

    pub fn touches_placement(&self, u: i32, v: i32, floor_elev: f32) -> bool {
        [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)]
            .into_iter()
            .any(|(du, dv)| self.grid.any_at(u + du, v + dv, |b| (b.floor() - floor_elev).abs() <= BLOCK_STEP + 0.12))
    }

    pub fn adjacent_hoisted_floor(&self, u: i32, v: i32, above_base: impl FnOnce(f32) -> bool) -> Option<f32> {
        let (mut min_z, mut adj_min) = (f32::MAX, None::<f32>);
        self.grid.visit_cells(|(cu, cv), list| {
            let near = (cu - u).abs() + (cv - v).abs() <= 1;
            for b in list {
                min_z = min_z.min(b.floor());
                if near {
                    adj_min = Some(adj_min.map_or(b.floor(), |cur| cur.min(b.floor())));
                }
            }
        });
        above_base(min_z).then_some(adj_min)?
    }

    pub fn clamped_vertical_delta(&self, raw_dz: f32, ground_fn: impl Fn(i32, i32) -> f32) -> f32 {
        let mut lift = 0.0f32;
        self.grid.visit_cells(|_, list| {
            for b in list {
                lift = lift.max(b.wheel_ground_lift());
            }
        });
        let mut clamped = raw_dz;
        self.grid.visit_cells(|(cu, cv), list| {
            let ground = ground_fn(cu, cv);
            for b in list {
                clamped = clamped.max(ground + lift - b.floor());
            }
        });
        clamped
    }

    pub fn shift_elevation(&mut self, dz: f32) { self.grid.shift_elevation(dz); }
    pub fn rotate_around(&mut self, piv_u: i32, piv_v: i32) { self.grid.rotate_around(piv_u, piv_v); }

    pub fn dismantle_into(self, world_blocks: &mut WorldBlocks) {
        for ((cu, cv), shape) in self.grid.into_entries() {
            if !shape.block_medium().vehicle_engine() {
                world_blocks.insert_elevated(cu, cv, shape);
            }
        }
    }

    pub fn nearest_seat_distance(&self, eu: f32, ev: f32) -> Option<f32> {
        let (su, sv, _, _) = self.nearest_seat(eu, ev)?;
        let (cx, cy) = GridCell::new(su, sv).center();
        Some((cx - eu).hypot(cy - ev))
    }

    fn nearest_seat(&self, eu: f32, ev: f32) -> Option<(i32, i32, f32, WedgeHeading)> {
        let mut best: Option<(i32, i32, f32, WedgeHeading, f32)> = None;
        self.grid.visit_cells(|(cu, cv), list| {
            if let Some(seat) = list.iter().rfind(|b| b.block_medium() == BlockMedium::CabinSeat) {
                let (cx, cy) = GridCell::new(cu, cv).center();
                let dist = (cx - eu).hypot(cy - ev);
                if best.is_none_or(|cur| dist < cur.4) {
                    best = Some((cu, cv, seat.floor(), seat.block_heading(), dist));
                }
            }
        });
        best.map(|(cu, cv, floor, heading, _)| (cu, cv, floor, heading))
    }

    fn steering_unit(&self) -> Option<(i32, i32, f32, WedgeHeading)> {
        let mut found = None;
        self.grid.visit_cells(|(cu, cv), list| {
            if found.is_none() {
                if let Some(eng) = list.iter().rfind(|b| b.block_medium() == BlockMedium::SteeringWheel) {
                    found = Some((cu, cv, eng.floor(), eng.block_heading()));
                }
            }
        });
        found
    }

    pub fn relocate_to_claw(
        &mut self,
        claw: (i32, i32, f32),
        eu: f32,
        ev: f32,
        frame: &SurfaceFrame,
    ) -> Option<(Vec<(i32, i32, f32)>, Vec3, Vec3)> {
        let (seat_u, seat_v, seat_floor, _) = self.nearest_seat(eu, ev)?;
        let (eng_u, eng_v, eng_floor, eng_heading) = self.steering_unit()?;
        let (du, dv, dz) = (claw.0 - eng_u, claw.1 - eng_v, claw.2 - eng_floor);
        let moved_floors = self.grid.translate_by(du, dv, dz);
        let (cx, cy) = GridCell::new(seat_u + du, seat_v + dv).center();
        Some((moved_floors, frame.point_at(cx, cy, seat_floor + dz), eng_heading.forward_vector(frame)))
    }

    pub fn perch_seat(&self, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        let (su, sv, seat_floor, seat_heading) = self.nearest_seat(eu, ev)?;
        let heading = self.steering_unit().map_or_else(
            || seat_heading.forward_vector(frame),
            |(eng_u, eng_v, _, eng_h)| {
                if (eng_u, eng_v) != (su, sv) {
                    (frame.tangent() * ((eng_u - su) as f32) + frame.bitangent() * ((eng_v - sv) as f32)).normalize_or_zero()
                } else {
                    eng_h.forward_vector(frame)
                }
            },
        );
        let axes = VehicleAxes::new(frame.tangent(), frame.bitangent(), frame.normal());
        let (cx, cy) = GridCell::new(su, sv).center();
        Some(VehicleAssembly::new(frame.point_at(cx, cy, seat_floor), axes, heading, Vec::new()))
    }

    pub fn targets_steering(&self, u: i32, v: i32, layer_elev: f32, ray: SurfaceSightRay, frame: &SurfaceFrame) -> bool {
        if self.grid.any_at(u, v, |b| b.occupies_layer(layer_elev) && b.block_medium() == BlockMedium::SteeringWheel) {
            return true;
        }
        (1..=28).any(|step| {
            let sample = ray.sample_point((step as f32) * 0.06);
            let (su, sv) = frame.decompose(sample);
            let (cu, cv) = GridCell::at_planar(su, sv).feed_coordinates(|gu, gv| (gu, gv));
            let ray_elev = frame.elevation_of(sample);
            self.grid.any_at(cu, cv, |b| {
                b.block_medium() == BlockMedium::SteeringWheel && ray_elev >= b.floor() - 0.15 && ray_elev <= b.ceiling() + 1.15
            })
        })
    }

    pub fn steerable(&self) -> bool { self.steering_unit().is_some() }

    pub fn drivable(&self) -> bool {
        let (mut wheel_present, mut seat_present, mut steering_present) = (false, false, false);
        self.grid.visit_cells(|_, list| {
            for block in list {
                match block.block_medium() {
                    BlockMedium::CarWheel => wheel_present = true,
                    BlockMedium::CabinSeat => seat_present = true,
                    BlockMedium::SteeringWheel => steering_present = true,
                    _ => {}
                }
            }
        });
        wheel_present && seat_present && steering_present
    }

    pub fn into_vehicle(
        self,
        ground_fn: impl Fn(i32, i32) -> f32,
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
    ) -> Result<VehicleAssembly, Self> {
        if !self.drivable() {
            return Err(self);
        }
        let (eu, ev) = frame.decompose(ray.sample_point(0.0));
        let Some((seat_u, seat_v, _, _)) = self.nearest_seat(eu, ev) else { return Err(self) };
        let ground_top = ground_fn(seat_u, seat_v);
        let mut floor_min = f32::MAX;
        self.grid.visit_cells(|_, list| {
            for b in list {
                floor_min = floor_min.min(b.floor());
            }
        });
        let parts: Vec<VehiclePart> = self
            .grid
            .into_entries()
            .map(|((cu, cv), b): ((i32, i32), ElevatedShape)| {
                VehiclePart::new((cu - seat_u, cv - seat_v), (b.floor() - floor_min).max(0.0), b.into_shape())
            })
            .collect();
        let axes = VehicleAxes::new(frame.tangent(), frame.bitangent(), frame.normal());
        let heading = parts.iter().find_map(|p| p.steering_heading(frame)).unwrap_or_else(|| ray.align_heading(frame));
        let (cx, cy) = GridCell::new(seat_u, seat_v).center();
        Ok(VehicleAssembly::new(frame.point_at(cx, cy, ground_top), axes, heading, parts))
    }
}

impl MoldedStructure for CraftCluster {
    fn vacant(&self) -> bool { self.grid.vacant() }
    fn present_at(&self, u: i32, v: i32) -> bool { self.grid.present_at(u, v) }
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 { self.grid.top_steps_at(u, v, floor_steps) }
    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 { self.grid.constrain_support(cell, planar, h, span) }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { self.grid.layer_occupied_at(u, v, layer_elev) }
    fn locked_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { self.grid.layer_occupied_at(u, v, layer_elev) }
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> { self.grid.seat_backrest_cell(u, v, layer_elev) }
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> { self.grid.medium_at(u, v, layer_elev) }
    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> { self.grid.hit_top_at(cu, cv, u, v, ray_elev) }
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        self.layer_occupied_at(u, v, layer_elev).then(|| self.perch_seat(eu, ev, frame)).flatten()
    }
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> { self.grid.tripod_hub(frame) }
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        self.grid.emit_at(cell, frame, colors, scars, target)
    }
}
