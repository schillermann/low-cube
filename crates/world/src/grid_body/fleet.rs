use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, BlockMold, GridCell};
use crate::brick_arena::ModuleSpec;
use crate::frame::SurfaceFrame;
use crate::relief::{
    CraftBlueprint, CraftCluster, CraftIdentity, CraftPose, DeckFrame, NaturalStrata,
    ShaftExcavation, SurfaceSightRay, VehicleAssembly, VesselSink,
};
use super::fleet_docking::FleetDockyard;
use super::fleet_surface::SurfaceFleetView;
use super::motion::BodyMooring;
use super::GridBody;

const ORBITAL_BOARD_REACH: f32 = 4.8;

/// Unified collection of independent `GridBody`s (surface rovers, landed ships, orbital stations, and docked modules) (`1` field).
pub struct GridFleet {
    bodies: Vec<GridBody>,
}

impl GridFleet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(bodies: Vec<GridBody>) -> Self {
        Self { bodies }
    }

    pub const fn untouched() -> Self {
        Self::new(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.bodies.len()
    }

    pub fn empty(&self) -> bool {
        self.bodies.is_empty()
    }

    pub fn bodies(&self) -> &[GridBody] {
        &self.bodies
    }

    pub fn surface_view(&self) -> SurfaceFleetView<'_> {
        SurfaceFleetView::new(&self.bodies)
    }

    pub fn find(&self, id: CraftIdentity) -> Option<&GridBody> {
        self.bodies.iter().find(|b| b.identity() == id)
    }

    pub fn insert_body(&mut self, body: GridBody) {
        let id = body.identity();
        if id.assigned() {
            self.bodies.retain(|b| b.identity() != id);
        }
        self.bodies.push(body);
    }

    pub fn park_settled(
        &mut self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
        assembly: VehicleAssembly,
    ) -> (i32, i32) {
        let (body, seat_cell) = GridBody::settled_on_surface(strata, shafts, frame, assembly);
        if body.blueprint().drivable() {
            self.insert_body(body);
        }
        seat_cell
    }

    pub fn insert_orbital(&mut self, assembly: VehicleAssembly, linear_vel: Vec3, angular_vel: Vec3, proper_accel: Vec3) {
        let body = GridBody::from_free_assembly(assembly, linear_vel, angular_vel, proper_accel);
        self.insert_body(body);
    }

    pub fn present_vessels(&self, sink: &mut dyn VesselSink) {
        for body in &self.bodies {
            sink.receive_vessel(body.blueprint(), body.pose());
        }
    }

    pub fn emit_all(&self, target: &mut dyn MeshTarget) {
        for body in &self.bodies {
            body.blueprint().emit_at(body.pose(), target);
        }
    }

    pub fn step_six_dof(&mut self, dt: f32, pull_at: impl Fn(Vec3) -> Vec3) -> bool {
        if self.bodies.is_empty() {
            return false;
        }
        let mut moved = false;
        for body in &mut self.bodies {
            if body.mooring().free_moving() {
                let g_dv = pull_at(body.pose().origin());
                if body.motion().linear_vel().length_squared() > 1e-8
                    || body.motion().angular_vel().length_squared() > 1e-10
                    || body.motion().proper_accel().length_squared() > 1e-8
                    || g_dv.length_squared() > 1e-8
                {
                    body.step_free(dt, g_dv);
                    moved = true;
                }
            }
        }
        let mut dockyard = FleetDockyard::new(&mut self.bodies);
        if dockyard.couple_nearby() {
            moved = true;
        }
        dockyard.propagate_docked();
        moved
    }

    pub fn couple_nearby_docking_rings(&mut self) -> bool {
        let mut dockyard = FleetDockyard::new(&mut self.bodies);
        let coupled = dockyard.couple_nearby();
        dockyard.propagate_docked();
        coupled
    }

    pub fn undock_body(&mut self, guest_id: CraftIdentity) -> bool {
        FleetDockyard::new(&mut self.bodies).undock(guest_id)
    }

    pub fn root_inertial_host(&self, id: CraftIdentity) -> CraftIdentity {
        let mut cur = id;
        for _ in 0..4 {
            let Some(body) = self.find(cur) else { break };
            let Some(link) = body.mooring().docked_link() else { break };
            cur = link.host();
        }
        cur
    }

    pub fn place_molded_on_aimed(&mut self, eye: Vec3, dir: Vec3, mold: BlockMold) -> bool {
        for body in &mut self.bodies {
            if body.hull_mason().place_molded(eye, dir, mold) {
                return true;
            }
        }
        false
    }

    pub fn place_scaled_on_aimed(&mut self, eye: Vec3, dir: Vec3, spec: ModuleSpec) -> bool {
        for body in &mut self.bodies {
            if body.hull_mason().place_scaled(eye, dir, spec) {
                return true;
            }
        }
        false
    }

    pub fn harvest_on_aimed(&mut self, eye: Vec3, dir: Vec3) -> Option<BlockMedium> {
        for body in &mut self.bodies {
            if let Some(med) = body.hull_mason().harvest(eye, dir) {
                return Some(med);
            }
        }
        None
    }

    pub fn board_orbital(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        let dir = direction.normalize_or_zero();
        if dir.length_squared() < 1e-4 {
            return None;
        }
        let idx = self
            .bodies
            .iter()
            .enumerate()
            .filter_map(|(i, b)| {
                let rel = b.pose().origin() - eye;
                let along = rel.dot(dir);
                if !(-0.6..=ORBITAL_BOARD_REACH).contains(&along) {
                    return None;
                }
                let radial_sq = (rel - dir * along.max(0.0)).length_squared();
                (radial_sq <= 4.0).then_some((i, rel.length_squared()))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)?;
        if holding_block {
            Some(self.bodies.swap_remove(idx).into_assembly())
        } else {
            Some(VehicleAssembly::from_blueprint(self.bodies[idx].blueprint().clone(), self.bodies[idx].pose()).perch_shell())
        }
    }

    pub fn sample_deck_at(&self, position: Vec3, player_height: f32) -> Option<DeckFrame> {
        for body in &self.bodies {
            let orbital = !matches!(body.mooring(), BodyMooring::SurfaceSettled(_));
            if let Some(frame) = DeckFrame::probe_with_dynamics(
                body.blueprint(),
                body.pose(),
                body.deck_dynamics(),
                position,
                player_height,
                orbital,
            ) {
                return Some(frame);
            }
        }
        None
    }

    pub fn resolve_deck_pose(&self, id: CraftIdentity) -> Option<(CraftPose, f32)> {
        let body = self.find(id)?;
        Some((body.pose(), body.deck_dynamics().local_proper_accel().y.max(0.0)))
    }

    pub fn sample_body_deck(&self, id: CraftIdentity, world_pos: Vec3) -> Option<DeckFrame> {
        let body = self.find(id)?;
        let local = body.pose().local_point_of(world_pos);
        Some(DeckFrame::with_dynamics(id, body.pose(), local, body.deck_dynamics()))
    }

    pub fn constrain_craft_by_id(&self, id: CraftIdentity, position: Vec3, player_height: f32) -> Option<Vec3> {
        let body = self.find(id)?;
        body.blueprint().constrain_at(body.pose(), position, player_height)
    }

    pub fn apply_frame_shift(&mut self, pos_shift: Vec3, vel_shift: Vec3) {
        for body in &mut self.bodies {
            let m = body.motion();
            body.apply_motion(m.shifted_by(pos_shift, m.linear_vel() + vel_shift));
        }
    }

    pub fn blocks_placement_at(&self, u: i32, v: i32, floor_elev: f32) -> bool {
        self.bodies.iter().any(|b| {
            let BodyMooring::SurfaceSettled(ff) = b.mooring() else { return false };
            let (uc, vc) = GridCell::new(u, v).center();
            b.posed_hull().touches_world_floor(ff.point_at(uc, vc, floor_elev))
        })
    }

    pub fn step_camera_turn(&mut self, u: i32, v: i32, delta_steps: i32) -> Option<(i32, i32)> {
        for body in &mut self.bodies {
            if body.step_camera_turn(u, v, delta_steps).is_some() {
                body.hull_mason().sync_blueprint();
                return Some((u, v));
            }
        }
        None
    }

    pub fn take_cluster_near(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, eye_world: Vec3, max_dist: f32) -> Option<CraftCluster> {
        let idx = self
            .bodies
            .iter()
            .enumerate()
            .filter_map(|(i, b)| {
                let (_, _, d) = b.posed_hull().nearest_seat_to(eye_world)?;
                (d <= max_dist).then_some((i, d))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)?;
        Some(self.bodies.swap_remove(idx).into_face_cluster(strata, shafts))
    }

    pub fn board_drivable(&mut self, u: i32, v: i32, layer_elev: f32, ray: SurfaceSightRay) -> Option<VehicleAssembly> {
        let idx = self.bodies.iter().position(|b| {
            let BodyMooring::SurfaceSettled(ff) = b.mooring() else { return false };
            let (uc, vc) = GridCell::new(u, v).center();
            b.blueprint().drivable() && b.posed_hull().targets_helm(ff.point_at(uc, vc, layer_elev), ray)
        })?;
        Some(self.bodies.swap_remove(idx).into_assembly())
    }

    pub fn capture_blueprint_at(&self, u: i32, v: i32, layer_elev: f32, ray: SurfaceSightRay) -> Option<CraftBlueprint> {
        if let Some(b) = self.bodies.iter().find(|b| b.occupies_surface_layer(u, v, layer_elev)) {
            return Some(b.blueprint().clone());
        }
        (1..=36).find_map(|step| {
            let pt = ray.sample_point((step as f32) * 0.12);
            self.bodies.iter().find(|b| b.posed_hull().medium_at_world(pt).is_some()).map(|b| b.blueprint().clone())
        })
    }

    pub fn find_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        for body in &self.bodies {
            if let Some(local) = body.blueprint().passenger_seat_along_ray(body.pose(), eye, direction) {
                return Some((body.identity(), local, body.pose().world_seat_at(local)));
            }
        }
        None
    }

    pub fn passenger_seat_frame(&self, id: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> {
        Some(self.find(id)?.pose().world_seat_at(local_seat))
    }
}
