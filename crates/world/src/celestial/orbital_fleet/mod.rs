mod traits;

use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CleaveOutcome, CraftBlueprint, CraftIdentity, CraftPose, DeckFrame,
    ExposedStratum, GravityField, MeshTarget, MiningEdge, PlanetaryWorld, PlannedMount,
    SurfaceFind, VehicleAssembly, VesselDeck, VesselSink, WorldEarthworks,
};

const SURFACE_PARK_CLEARANCE: f32 = 4.5;
const ORBITAL_BOARD_REACH: f32 = 4.8;

/// `PlanetaryWorld` decorator preserving unmanned spacecraft weightlessly in 3D space during EVA spacewalks.
pub struct OrbitalFleetWorld<W> {
    pub(super) origin: W,
    drifting: Vec<VehicleAssembly>,
}

impl<W> OrbitalFleetWorld<W> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: W, drifting: Vec<VehicleAssembly>) -> Self {
        Self { origin, drifting }
    }

    pub const fn wrapping(origin: W) -> Self {
        Self::new(origin, Vec::new())
    }

    fn aimed_orbital_index(&self, eye: Vec3, direction: Vec3) -> Option<usize> {
        let dir = direction.normalize_or_zero();
        if dir.length_squared() < 1e-4 {
            return None;
        }
        self.drifting
            .iter()
            .enumerate()
            .filter_map(|(idx, craft)| {
                let rel = craft.origin() - eye;
                let along = rel.dot(dir);
                if !(-0.6..=ORBITAL_BOARD_REACH).contains(&along) {
                    return None;
                }
                let radial_sq = (rel - dir * along.max(0.0)).length_squared();
                (radial_sq <= 4.0).then_some((idx, rel.length_squared()))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(idx, _)| idx)
    }
}

impl<W: PlanetaryWorld> OrbitalFleetWorld<W> {
    fn above_surface(&self, pos: Vec3) -> bool {
        let ground = self.origin.constrain_to_surface(pos, 0.0);
        if (pos - ground).length_squared() < 1e-6 {
            return self.origin.apply_pull(pos, 1.0).length_squared() < 0.25;
        }
        let up = self.origin.surface_up_at(pos);
        (pos - ground).dot(up) > SURFACE_PARK_CLEARANCE
    }
}

impl<W: GravityField> GravityField for OrbitalFleetWorld<W> {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 { self.origin.pull_direction_at(position) }
    fn surface_up_at(&self, position: Vec3) -> Vec3 { self.origin.surface_up_at(position) }
    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 { self.origin.apply_pull(position, dt) }
}

impl<W: PlanetaryWorld> WorldEarthworks for OrbitalFleetWorld<W> {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> { self.origin.harvest_surface(eye, direction) }
    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> { self.origin.loosen_surface(eye, direction, stage) }
    fn surface_yields_to(&self, eye: Vec3, direction: Vec3, edge: MiningEdge) -> bool { self.origin.surface_yields_to(eye, direction, edge) }
    fn aimed_surface_medium(&self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> { self.origin.aimed_surface_medium(eye, direction) }
    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome { self.origin.strike_timber(eye, direction) }
    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind { self.origin.forage_surface(eye, direction) }
    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool { self.origin.cast_surface_find(eye, direction, find) }
    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        self.origin.place_molded_surface_block(eye, direction, avatar_pos, mold)
    }
    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        self.origin.adjust_rig(eye, direction, delta_steps, rotate_quarter)
    }
    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool { self.origin.lower_rig(eye, direction) }
    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        self.origin.build_rig_blueprint(eye, direction, blueprint)
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        if let Some(idx) = self.aimed_orbital_index(eye, direction) {
            if holding_block {
                return Some(self.drifting.swap_remove(idx));
            }
            return Some(self.drifting[idx].perch_shell());
        }
        self.origin.board_vehicle(eye, direction, holding_block)
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        if self.above_surface(assembly.origin()) {
            let id = assembly.identity();
            if id.assigned() {
                self.drifting.retain(|c| c.identity() != id);
            }
            let quiet = VehicleAssembly::from_blueprint(
                assembly.blueprint().clone(),
                assembly.pose().with_motion(0.0, 0.0),
            );
            self.drifting.push(quiet);
            true
        } else {
            self.origin.park_vehicle(assembly)
        }
    }

    fn register_craft_blueprint(&mut self, blueprint: CraftBlueprint) -> bool { self.origin.register_craft_blueprint(blueprint) }
    fn claim_vehicle_helm(&mut self, craft: CraftIdentity, pilot_slot: u32, eye: Vec3, direction: Vec3) -> bool {
        self.origin.claim_vehicle_helm(craft, pilot_slot, eye, direction)
    }
    fn park_craft_by_id(&mut self, craft: CraftIdentity, pose: CraftPose) -> bool {
        self.origin.park_craft_by_id(craft, pose)
    }
    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        self.origin.disclose_stratum(face_index, stratum);
    }
    fn offworld_frame(&self, probe: Vec3) -> Option<(crate::BodyId, Vec3)> {
        self.origin.offworld_frame(probe)
    }
    fn body_center(&self, body: crate::BodyId) -> Option<Vec3> {
        self.origin.body_center(body)
    }
    fn harvest_offworld(&mut self, body: crate::BodyId, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.origin.harvest_offworld(body, eye, direction)
    }
    fn enact_offworld(&mut self, body: crate::BodyId, fact: &crate::WorldFact) -> bool {
        self.origin.enact_offworld(body, fact)
    }
    fn capture_sector_snapshot(&self, face: u8, gsu: i32, gsv: i32) -> crate::SectorSnapshot {
        self.origin.capture_sector_snapshot(face, gsu, gsv)
    }
    fn apply_sector_snapshot(&mut self, snapshot: &crate::SectorSnapshot) -> bool {
        self.origin.apply_sector_snapshot(snapshot)
    }
    fn collect_sector_snapshots(&self, out: &mut Vec<crate::SectorSnapshot>) {
        self.origin.collect_sector_snapshots(out);
    }
    fn capture_offworld_snapshot(&mut self, body: crate::BodyId, face: u8, gsu: i32, gsv: i32) -> crate::SectorSnapshot {
        self.origin.capture_offworld_snapshot(body, face, gsu, gsv)
    }
    fn collect_offworld_snapshots(&mut self, out: &mut Vec<(crate::BodyId, crate::SectorSnapshot)>) {
        self.origin.collect_offworld_snapshots(out);
    }
}

impl<W: VesselDeck> VesselDeck for OrbitalFleetWorld<W> {
    fn board_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        self.origin.board_passenger_seat(eye, direction)
    }
    fn passenger_seat_frame(&self, craft: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> {
        self.origin.passenger_seat_frame(craft, local_seat)
    }
    fn sample_deck_frame(&self, position: Vec3, player_height: f32) -> Option<DeckFrame> {
        for craft in &self.drifting {
            if let Some(frame) = DeckFrame::probe(craft.blueprint(), craft.pose(), position, player_height) {
                return Some(frame);
            }
        }
        self.origin.sample_deck_frame(position, player_height)
    }
    fn resolve_deck_pose(&self, id: CraftIdentity) -> Option<(CraftPose, f32)> {
        if let Some(craft) = self.drifting.iter().find(|c| c.identity() == id) {
            return Some((craft.pose(), DeckFrame::thrust_from_pose(craft.pose())));
        }
        self.origin.resolve_deck_pose(id)
    }
    fn constrain_on_craft(&self, id: CraftIdentity, position: Vec3, player_height: f32) -> Option<Vec3> {
        if let Some(craft) = self.drifting.iter().find(|c| c.identity() == id) {
            return craft.blueprint().constrain_at(craft.pose(), position, player_height);
        }
        self.origin.constrain_on_craft(id, position, player_height)
    }
    fn helm_revoked(&mut self, craft: CraftIdentity) -> bool { self.origin.helm_revoked(craft) }
    fn emit_active_crafts(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_active_crafts(target);
        for craft in &self.drifting {
            craft.emit(target);
        }
    }
    fn present_vessels(&self, sink: &mut dyn VesselSink) {
        self.origin.present_vessels(sink);
        for craft in &self.drifting {
            sink.receive_vessel(craft.blueprint(), craft.pose());
        }
    }
    fn present_vessels_near(&self, center: Vec3, radius: f32, sink: &mut dyn VesselSink) {
        self.origin.present_vessels_near(center, radius, sink);
        let r_sq = radius * radius;
        for craft in &self.drifting {
            if (craft.origin() - center).length_squared() <= r_sq {
                sink.receive_vessel(craft.blueprint(), craft.pose());
            }
        }
    }
    fn plan_molded_block(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<PlannedMount> {
        self.origin.plan_molded_block(eye, direction, avatar_pos, mold)
    }
    fn capture_vehicle_blueprint(&self, eye: Vec3, direction: Vec3) -> Option<CraftBlueprint> {
        if let Some(idx) = self.aimed_orbital_index(eye, direction) {
            return Some(self.drifting[idx].blueprint().clone());
        }
        self.origin.capture_vehicle_blueprint(eye, direction)
    }
    fn aimed_empty_rig(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.origin.aimed_empty_rig(eye, direction, avatar_pos)
    }
}

impl<W: PlanetaryWorld> PlanetaryWorld for OrbitalFleetWorld<W> {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        let ground = self.origin.constrain_to_surface(position, player_height);
        let up = self.origin.surface_up_at(position);
        let mut best = ground;
        for craft in &self.drifting {
            if let Some(supported) = craft.blueprint().constrain_at(craft.pose(), position, player_height) {
                if supported.dot(up) > best.dot(up) {
                    best = supported;
                }
            }
        }
        best
    }

    fn surface_contact(&self, probe: Vec3, clearance: f32) -> crate::SurfaceContact {
        self.origin.surface_contact(probe, clearance)
    }

    fn canopy_drag(&self, position: Vec3) -> f32 { self.origin.canopy_drag(position) }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> { self.origin.placed_camera_tripod() }
    fn collect_medium_sites(&self, medium: crate::BlockMedium, out: &mut Vec<(Vec3, Vec3)>) {
        self.origin.collect_medium_sites(medium, out);
    }
    fn collect_offworld_medium_sites(&self, medium: crate::BlockMedium, out: &mut Vec<(crate::BodyId, Vec3, Vec3)>) {
        self.origin.collect_offworld_medium_sites(medium, out);
    }
    fn assigned_spawn_point(&self) -> Option<Vec3> { self.origin.assigned_spawn_point() }
    fn assigned_survivor(&self) -> Option<crate::SurvivorWelcome> { self.origin.assigned_survivor() }
    fn anchor_to_frame(&mut self, frame: crate::CosmicFrame, local_pos: Vec3) {
        self.origin.anchor_to_frame(frame, local_pos);
    }
    fn step_surface(&mut self, dt: f32) -> bool { self.origin.step_surface(dt) }
    fn step_active_zones(&mut self, dt: f32, earth_zone_mask: u8, active_bodies: &[crate::BodyId]) -> bool {
        self.origin.step_active_zones(dt, earth_zone_mask, active_bodies)
    }
    fn step_local_flora(&mut self, dt: f32) -> bool { self.origin.step_local_flora(dt) }
    fn lethal_at(&self, position: Vec3) -> bool { self.origin.lethal_at(position) }
    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 { self.origin.confine_beneath_crust(position) }
    fn local_body_point(&self, position: Vec3) -> Vec3 { self.origin.local_body_point(position) }
    fn celestial_atlas(&self) -> crate::CelestialAtlas { self.origin.celestial_atlas() }
    fn cosmic_frame(&self) -> crate::CosmicFrame { self.origin.cosmic_frame() }
    fn align_epoch(&mut self, epoch_secs: f64) { self.origin.align_epoch(epoch_secs); }
    fn reconcile_cosmic_frame(&mut self, local_pos: Vec3, local_vel: Vec3) -> Option<crate::FrameHandoff> {
        let handoff = self.origin.reconcile_cosmic_frame(local_pos, local_vel)?;
        let shift = handoff.pos_shift();
        for craft in &mut self.drifting {
            craft.translate_to(craft.origin() + shift);
        }
        Some(handoff)
    }
}
