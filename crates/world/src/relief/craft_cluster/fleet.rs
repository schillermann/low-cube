use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell};
use crate::frame::SurfaceFrame;
use super::super::craft_entity::{
    CraftBlueprint, CraftIdentity, CraftPose, DeckFrame, PosedHull, VesselSink,
};
use super::super::molded_structure::MoldedStructure;
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;
use super::super::sight_ray::SurfaceSightRay;
use super::super::turf_scars::TurfScars;
use super::super::vehicle_assembly::VehicleAssembly;
use super::CraftCluster;

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Living 6-DOF vehicle resting on a planetary face without 90-degree terrain-sector grid snapping.
pub struct ParkedCraft {
    blueprint: CraftBlueprint,
    pose: CraftPose,
    face_frame: SurfaceFrame,
}

impl ParkedCraft {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(blueprint: CraftBlueprint, pose: CraftPose, face_frame: SurfaceFrame) -> Self {
        Self { blueprint, pose, face_frame }
    }

    pub fn settled(
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
        assembly: VehicleAssembly,
    ) -> (Self, (i32, i32)) {
        let (raw_bp, raw_pose) = assembly.into_blueprint_pose();
        let bp = if raw_bp.identity().assigned() || !raw_bp.drivable() {
            raw_bp
        } else {
            CraftBlueprint::commission(raw_pose.origin(), raw_bp.into_parts())
        };
        let (u0, v0) = frame.decompose(raw_pose.origin());
        let half_cells = (frame.height_of(frame.point_at(0.0, 0.0, 0.0)) * 2.0).round() as i32;
        let (u, v) = if half_cells >= 8 { frame.clamp_interior(u0, v0, 0.35) } else { (u0, v0) };
        let seat_cell = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        let ground_top = (shafts.elevation_at(strata, seat_cell.0, seat_cell.1) as f32) * BLOCK_STEP;
        let cur_elev = frame.elevation_of(raw_pose.origin());
        let settled_elev = if (cur_elev - ground_top).abs() <= 0.65 {
            ground_top
        } else {
            cur_elev.max(ground_top)
        };
        let settled_origin = frame.point_at(u, v, settled_elev);
        let settled_pose = raw_pose.relocated(settled_origin, frame.normal()).with_motion(0.0, 0.0);
        (Self::new(bp, settled_pose, *frame), seat_cell)
    }

    pub const fn identity(&self) -> CraftIdentity { self.blueprint.identity() }
    pub const fn blueprint(&self) -> &CraftBlueprint { &self.blueprint }
    pub const fn pose(&self) -> CraftPose { self.pose }
    const fn hull(&self) -> PosedHull<'_> { PosedHull::new(&self.blueprint, self.pose) }
    pub fn into_assembly(self) -> VehicleAssembly { VehicleAssembly::from_blueprint(self.blueprint, self.pose) }

    pub fn into_cluster(self, strata: &NaturalStrata, shafts: &ShaftExcavation) -> CraftCluster {
        let lift = self.blueprint.wheel_ground_lift();
        let raw_seat = self.face_frame.cell_at(self.pose.origin()).feed_coordinates(|cu, cv| (cu, cv));
        let basis = self.pose.axes().grid_basis(&self.face_frame);
        let half_cells = (self.face_frame.height_of(self.face_frame.point_at(0.0, 0.0, 0.0)) * 2.0).round() as i32;
        let seat = self.blueprint.clamped_seat(raw_seat, basis, half_cells);
        let mut craft = CraftCluster::untouched();
        for part in self.blueprint.into_parts() {
            part.park_into(seat, basis, lift, &mut craft, strata, shafts);
        }
        craft
    }

    fn world_cell_center(&self, u: i32, v: i32, elev: f32) -> Vec3 {
        let (uc, vc) = GridCell::new(u, v).center();
        self.face_frame.point_at(uc, vc, elev)
    }

    fn occupies_cell_layer(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        let (uc, vc) = GridCell::new(u, v).center();
        [(0.0, 0.0), (0.14, 0.14), (-0.14, 0.14), (0.14, -0.14), (-0.14, -0.14)]
            .into_iter()
            .any(|(du, dv)| {
                let pt = self.face_frame.point_at(uc + du, vc + dv, layer_elev);
                self.hull().medium_at_world(pt).is_some()
            })
    }
}

/// Registry of independent 6-DOF vehicle entities parked on a planetary face, decoupled from terrain sectors.
pub struct ParkedFleet {
    crafts: Vec<ParkedCraft>,
}

impl ParkedFleet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(crafts: Vec<ParkedCraft>) -> Self {
        Self { crafts }
    }

    pub const fn untouched() -> Self {
        Self::new(Vec::new())
    }

    pub fn park_settled(
        &mut self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
        assembly: VehicleAssembly,
    ) -> (i32, i32) {
        let (parked, seat_cell) = ParkedCraft::settled(strata, shafts, frame, assembly);
        if parked.blueprint().drivable() {
            let id = parked.identity();
            if id.assigned() {
                self.crafts.retain(|c| c.identity() != id);
            }
            self.crafts.push(parked);
        }
        seat_cell
    }

    pub fn present_vessels(&self, sink: &mut dyn VesselSink) {
        for craft in &self.crafts {
            sink.receive_vessel(craft.blueprint(), craft.pose());
        }
    }

    pub fn blocks_placement_at(&self, u: i32, v: i32, floor_elev: f32) -> bool {
        self.crafts.iter().any(|c| c.hull().touches_world_floor(c.world_cell_center(u, v, floor_elev)))
    }

    pub fn step_camera_turn(&mut self, u: i32, v: i32, delta_steps: i32) -> Option<(i32, i32)> {
        for craft in &mut self.crafts {
            if craft.blueprint.step_camera_turn(delta_steps).is_some() {
                return Some((u, v));
            }
        }
        None
    }

    pub fn take_cluster_near(
        &mut self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        eye_world: Vec3,
        max_dist: f32,
    ) -> Option<CraftCluster> {
        let idx = self
            .crafts
            .iter()
            .enumerate()
            .filter_map(|(i, c)| {
                let (_, _, d) = c.hull().nearest_seat_to(eye_world)?;
                (d <= max_dist).then_some((i, d))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)?;
        Some(self.crafts.swap_remove(idx).into_cluster(strata, shafts))
    }

    pub fn board_drivable(&mut self, u: i32, v: i32, layer_elev: f32, ray: SurfaceSightRay) -> Option<VehicleAssembly> {
        let idx = self.crafts.iter().position(|c| {
            let hit = c.world_cell_center(u, v, layer_elev);
            c.blueprint().drivable() && c.hull().targets_helm(hit, ray)
        })?;
        Some(self.crafts.swap_remove(idx).into_assembly())
    }

    pub fn find_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        for craft in &self.crafts {
            if let Some(local) = craft.blueprint().passenger_seat_along_ray(craft.pose(), eye, direction) {
                return Some((craft.identity(), local, craft.pose().world_seat_at(local)));
            }
        }
        None
    }

    pub fn passenger_seat_frame(&self, id: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> {
        let craft = self.crafts.iter().find(|c| c.identity() == id)?;
        Some(craft.pose().world_seat_at(local_seat))
    }

    pub fn sample_deck_at(&self, position: Vec3, player_height: f32) -> Option<DeckFrame> {
        for craft in &self.crafts {
            if let Some(frame) = DeckFrame::probe(craft.blueprint(), craft.pose(), position, player_height) {
                return Some(frame);
            }
        }
        None
    }

    pub fn resolve_deck_pose(&self, id: CraftIdentity) -> Option<(CraftPose, f32)> {
        let craft = self.crafts.iter().find(|c| c.identity() == id)?;
        Some((craft.pose(), DeckFrame::thrust_from_pose(craft.pose())))
    }

    pub fn constrain_craft_by_id(&self, id: CraftIdentity, position: Vec3, player_height: f32) -> Option<Vec3> {
        let craft = self.crafts.iter().find(|c| c.identity() == id)?;
        craft.blueprint().constrain_at(craft.pose(), position, player_height)
    }
}

impl MoldedStructure for ParkedFleet {
    fn vacant(&self) -> bool { self.crafts.is_empty() }
    fn present_at(&self, u: i32, v: i32) -> bool {
        self.crafts.iter().any(|c| {
            let (uc, vc) = GridCell::new(u, v).center();
            let probe = c.face_frame.point_at(uc, vc, 64.0);
            c.blueprint().constrain_at(c.pose(), probe, 0.0).is_some()
        })
    }
    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 {
        let (uc, vc) = GridCell::new(u, v).center();
        self.crafts.iter().fold(floor_steps, |acc, c| {
            let probe = c.face_frame.point_at(uc, vc, 64.0);
            match c.blueprint().constrain_at(c.pose(), probe, 0.0) {
                Some(sup) => acc.max((c.face_frame.elevation_of(sup) / BLOCK_STEP).ceil() as i32),
                None => acc,
            }
        })
    }
    fn constrain_support(&self, _cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 {
        self.crafts.iter().fold(h, |acc, c| {
            let probe = c.face_frame.point_at(planar.0, planar.1, acc + span.0);
            match c.blueprint().constrain_at(c.pose(), probe, span.0) {
                Some(sup) => acc.max(c.face_frame.elevation_of(sup) - span.0),
                None => acc,
            }
        })
    }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.crafts.iter().any(|c| c.occupies_cell_layer(u, v, layer_elev))
    }
    fn locked_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { self.layer_occupied_at(u, v, layer_elev) }
    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        self.crafts.iter().find_map(|c| {
            let back = c.hull().backrest_world_at(c.world_cell_center(u, v, layer_elev))?;
            Some(c.face_frame.cell_at(back).feed_coordinates(|gu, gv| (gu, gv)))
        })
    }
    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.crafts.iter().find_map(|c| c.hull().medium_at_world(c.world_cell_center(u, v, layer_elev)))
    }
    fn hit_top_at(&self, _cu: i32, _cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.crafts.iter().find_map(|c| {
            let pt = c.face_frame.point_at(u, v, ray_elev);
            c.hull().hit_ceiling_at(pt).map(|top| c.face_frame.elevation_of(top))
        })
    }
    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, eu: f32, ev: f32, _frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        self.crafts.iter().find_map(|c| {
            let hit = c.world_cell_center(u, v, layer_elev);
            let eye = c.face_frame.point_at(eu, ev, layer_elev + 1.0);
            c.hull().perch_at(hit, eye)
        })
    }
    fn tripod_hub(&self, _frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.crafts.iter().find_map(|c| c.hull().tripod_hub())
    }
    fn emit_at(&self, _cell: GridCell, _frame: &SurfaceFrame, _colors: (Vec3, Vec3), _scars: &TurfScars, _target: &mut dyn MeshTarget) -> bool {
        false
    }
}
