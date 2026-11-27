mod commission;
mod fleet;
mod helm;

use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell, WedgeHeading};
use crate::brick_arena::{BrickCell, BrickStructure, CellCoord, ModuleSpec, RawAtom};
use crate::frame::SurfaceFrame;
use super::craft_entity::{CraftBlueprint, CraftIdentity};
use super::molded_structure::{MoldedStructure, WorldBlocks};
use super::natural_strata::NaturalStrata;
use super::shaft_excavation::ShaftExcavation;
use super::sight_ray::SurfaceSightRay;
use super::turf_scars::TurfScars;
use super::vehicle_assembly::{VehicleAssembly, VehicleAxes};
use self::commission::PadCommission;
pub use self::fleet::ParkedFleet;
use self::helm::ClusterHelm;

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Independent multi-block object constructed on a cantilever crane backed by `InertialBricks<BrickArena>` (`1` field).
pub struct CraftCluster {
    grid: BrickStructure,
}

impl CraftCluster {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(grid: BrickStructure) -> Self {
        Self { grid }
    }

    pub fn untouched() -> Self {
        Self::new(BrickStructure::untouched())
    }

    pub const fn structure(&self) -> &BrickStructure {
        &self.grid
    }

    pub fn insert(&mut self, u: i32, v: i32, floor_elev: f32, atom: RawAtom) {
        if self.grid.vacant() {
            let w = (floor_elev / BLOCK_STEP).round() as i32;
            let rem = floor_elev - (w as f32) * BLOCK_STEP;
            if rem.abs() > 1e-4 {
                self.grid.shift_elevation(rem);
            }
        }
        self.grid.insert_atom(u, v, floor_elev, atom);
    }

    pub fn place_module(&mut self, anchor: CellCoord, spec: ModuleSpec) -> bool {
        self.grid.place_module_at(anchor, spec)
    }

    pub fn snapshot_relative(&self, mast_u: i32, mast_v: i32) -> Vec<(i16, i16, i16, [u8; 3])> {
        let mut cells = Vec::new();
        let datum = self.grid.datum();
        self.grid.bricks().inner().visit_occupied_cells(|coord, cell| {
            let BrickCell::RawAtom(atom) = cell else { return };
            let (cu, cv) = coord.planar();
            let du = (cu - mast_u).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            let dv = (cv - mast_v).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            let fq = (coord.floor_elev(datum) * 4.0).round().clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            cells.push((du, dv, fq, atom.stamp()));
        });
        cells.sort_unstable();
        cells
    }

    pub fn from_relative_snapshot(mast_u: i32, mast_v: i32, cells: &[(i16, i16, i16, [u8; 3])]) -> Self {
        let mut out = Self::untouched();
        for &(du, dv, fq, stamp) in cells {
            out.insert(
                mast_u + i32::from(du),
                mast_v + i32::from(dv),
                (fq as f32) * 0.25,
                crate::BlockMold::atom_from_stamp(stamp),
            );
        }
        out
    }

    pub fn step_camera_turn(&mut self, u: i32, v: i32, delta_steps: i32) -> Option<(i32, i32)> {
        self.grid.step_camera_turn(u, v, delta_steps)
    }

    pub fn remove_layer(&mut self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.grid.remove_at(u, v, layer_elev).and_then(|(span, _)| span.medium())
    }

    pub fn occupies_claw(&self, claw: (i32, i32), hoist_z: f32) -> bool {
        let datum = self.grid.datum();
        self.grid
            .bricks()
            .inner()
            .any_in_column(claw.0, claw.1, |c, _| (c.floor_elev(datum) - hoist_z).abs() <= BLOCK_STEP + 0.12)
    }

    pub fn touches_placement(&self, u: i32, v: i32, floor_elev: f32) -> bool {
        let datum = self.grid.datum();
        [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)].into_iter().any(|(du, dv)| {
            self.grid
                .bricks()
                .inner()
                .any_in_column(u + du, v + dv, |c, _| (c.floor_elev(datum) - floor_elev).abs() <= BLOCK_STEP + 0.12)
        })
    }

    pub fn adjacent_hoisted_floor(&self, u: i32, v: i32, above_base: impl FnOnce(f32) -> bool) -> Option<f32> {
        let (mut min_z, mut adj_min) = (f32::MAX, None::<f32>);
        let datum = self.grid.datum();
        self.grid.bricks().inner().visit_occupied_cells(|coord, _| {
            let (cu, cv) = coord.planar();
            let f = coord.floor_elev(datum);
            min_z = min_z.min(f);
            if (cu - u).abs() + (cv - v).abs() <= 1 {
                adj_min = Some(adj_min.map_or(f, |cur| cur.min(f)));
            }
        });
        above_base(min_z).then_some(adj_min)?
    }

    pub fn clamped_vertical_delta(&self, raw_dz: f32, ground_fn: impl Fn(i32, i32) -> f32) -> f32 {
        let datum = self.grid.datum();
        let mut lift = 0.0f32;
        self.grid.bricks().inner().visit_occupied_cells(|coord, cell| {
            if let BrickCell::RawAtom(a) = cell {
                lift = lift.max(a.at_floor(coord.floor_elev(datum)).wheel_ground_lift());
            }
        });
        let mut clamped = raw_dz;
        self.grid.bricks().inner().visit_occupied_cells(|coord, _| {
            let (cu, cv) = coord.planar();
            clamped = clamped.max(ground_fn(cu, cv) + lift - coord.floor_elev(datum));
        });
        clamped
    }

    pub fn shift_elevation(&mut self, dz: f32) { self.grid.shift_elevation(dz); }
    pub fn rotate_around(&mut self, piv_u: i32, piv_v: i32) { self.grid.rotate_around(piv_u, piv_v); }
    pub(in crate::relief) fn translate_by(&mut self, du: i32, dv: i32, dz: f32) { let _ = self.grid.translate_by(du, dv, dz); }

    pub(in crate::relief) fn claw_anchor(&self) -> Option<(i32, i32, f32)> {
        if let Some((u, v, z, _)) = self.steering_unit() {
            return Some((u, v, z));
        }
        let datum = self.grid.datum();
        let mut fallback: Option<(i32, i32, f32)> = None;
        self.grid.bricks().inner().visit_occupied_cells(|coord, _| {
            let (cu, cv) = coord.planar();
            let f = coord.floor_elev(datum);
            if (cu, cv) == (0, 0) || fallback.is_none_or(|(fu, fv, _)| (fu, fv) != (0, 0) && (cu, cv) < (fu, fv)) {
                fallback = Some((cu, cv, f));
            }
        });
        fallback
    }

    pub(in crate::relief) fn min_boom_step(&self, mast_u: i32, mast_v: i32, step: (i32, i32), reach_steps: i32) -> i32 {
        let mut min_s = reach_steps;
        self.grid.bricks().inner().visit_occupied_cells(|coord, _| {
            let (cu, cv) = coord.planar();
            let (du, dv) = (cu - mast_u, cv - mast_v);
            if du * step.1 == dv * step.0 {
                let s = du * step.0 + dv * step.1;
                if s <= reach_steps {
                    min_s = min_s.min(s);
                }
            }
        });
        min_s
    }

    pub fn dismantle_into(self, strata: &NaturalStrata, shafts: &ShaftExcavation, world_blocks: &mut WorldBlocks, scars: &mut TurfScars) {
        let datum = self.grid.datum();
        self.grid.bricks().inner().visit_occupied_cells(|coord, cell| {
            if let BrickCell::RawAtom(atom) = cell {
                if !atom.medium().vehicle_engine() {
                    let (cu, cv) = coord.planar();
                    let ground_top = (shafts.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP;
                    let floor = ((coord.floor_elev(datum) / BLOCK_STEP).round()) * BLOCK_STEP;
                    world_blocks.stack_bare(cu, cv, floor.max(ground_top), ground_top, false, atom, scars);
                }
            }
        });
    }

    pub fn nearest_seat_distance(&self, eu: f32, ev: f32) -> Option<f32> {
        let (su, sv, _, _) = self.nearest_seat(eu, ev)?;
        let (cx, cy) = GridCell::new(su, sv).center();
        Some((cx - eu).hypot(cy - ev))
    }

    fn nearest_seat(&self, eu: f32, ev: f32) -> Option<(i32, i32, f32, WedgeHeading)> {
        let datum = self.grid.datum();
        let mut best: Option<(i32, i32, f32, WedgeHeading, f32)> = None;
        self.grid.bricks().inner().visit_occupied_cells(|coord, cell| {
            let seat = match cell {
                BrickCell::RawAtom(a) if a.medium() == BlockMedium::CabinSeat => Some(a.heading()),
                BrickCell::ModuleAnchor(s) if s.medium() == BlockMedium::PilotSeat => Some(WedgeHeading::from_code(s.turn().code())),
                _ => None,
            };
            if let Some(heading) = seat {
                let (cu, cv) = coord.planar();
                let (cx, cy) = GridCell::new(cu, cv).center();
                let dist = (cx - eu).hypot(cy - ev);
                if best.is_none_or(|cur| dist < cur.4) {
                    best = Some((cu, cv, coord.floor_elev(datum), heading, dist));
                }
            }
        });
        best.map(|(cu, cv, floor, heading, _)| (cu, cv, floor, heading))
    }

    fn steering_unit(&self) -> Option<(i32, i32, f32, WedgeHeading)> { ClusterHelm::new(&self.grid).unit() }

    pub fn relocate_to_claw(&mut self, claw: (i32, i32, f32), eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<(Vec<(i32, i32, f32)>, Vec3, Vec3)> {
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
        ClusterHelm::new(&self.grid).targets_helm(u, v, layer_elev, ray, frame)
    }

    pub fn steerable(&self) -> bool { self.steering_unit().is_some() }
    pub fn drivable(&self) -> bool { ClusterHelm::new(&self.grid).drivable() }

    pub fn into_vehicle(self, ground_fn: impl Fn(i32, i32) -> f32, ray: SurfaceSightRay, frame: &SurfaceFrame) -> Result<VehicleAssembly, Self> {
        self.into_vehicle_with_id(CraftIdentity::unassigned(), ground_fn, ray, frame)
    }

    pub fn capture_blueprint(&self, eu: f32, ev: f32, frame: &SurfaceFrame) -> Option<CraftBlueprint> {
        if self.grid.vacant() { return None; }
        let anchor = self.nearest_seat(eu, ev).map(|(su, sv, _, _)| (su, sv)).or_else(|| self.claw_anchor().map(|(u, v, _)| (u, v)))?;
        PadCommission::new(&self.grid).draft_blueprint(anchor, frame)
    }

    pub fn into_vehicle_with_id(self, identity: CraftIdentity, ground_fn: impl Fn(i32, i32) -> f32, ray: SurfaceSightRay, frame: &SurfaceFrame) -> Result<VehicleAssembly, Self> {
        if !self.drivable() { return Err(self); }
        let (eu, ev) = frame.decompose(ray.sample_point(0.0));
        let Some((seat_u, seat_v, _, _)) = self.nearest_seat(eu, ev) else { return Err(self) };
        Ok(PadCommission::new(&self.grid).assemble_vehicle(identity, (seat_u, seat_v), ground_fn, ray, frame))
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
        matches!(self.medium_at(u, v, layer_elev), Some(BlockMedium::CabinSeat | BlockMedium::PilotSeat))
            .then(|| self.perch_seat(eu, ev, frame))
            .flatten()
    }
    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> { self.grid.tripod_hub(frame) }
    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        self.grid.emit_at(cell, frame, colors, scars, target)
    }
}
