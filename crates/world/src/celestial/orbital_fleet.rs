use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CleaveOutcome, CompanionPresence, CraftBlueprint, CraftIdentity,
    CraftPose, DeckFrame, DrivenCraft, ExposedStratum, GravityField, MeadowRing, MeshTarget,
    PilotCraft, PlanetaryWorld, SurfaceFind, VehicleAssembly, WorldEarthworks,
};

const SURFACE_PARK_CLEARANCE: f32 = 4.5;
const ORBITAL_BOARD_REACH: f32 = 4.8;

/// `PlanetaryWorld` decorator preserving unmanned spacecraft weightlessly in 3D space during EVA spacewalks.
pub struct OrbitalFleetWorld<W> {
    origin: W,
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
}

impl<W: PlanetaryWorld> PlanetaryWorld for OrbitalFleetWorld<W> {
    fn emit_surface(&self, target: &mut dyn MeshTarget) { self.origin.emit_surface(target); }

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

    fn canopy_drag(&self, position: Vec3) -> f32 { self.origin.canopy_drag(position) }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> { self.origin.placed_camera_tripod() }
    fn step_surface(&mut self, dt: f32) -> bool { self.origin.step_surface(dt) }
    fn step_local_flora(&mut self, dt: f32) -> bool { self.origin.step_local_flora(dt) }
    fn lethal_at(&self, position: Vec3) -> bool { self.origin.lethal_at(position) }
    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 { self.origin.confine_beneath_crust(position) }
    fn face_count(&self) -> usize { self.origin.face_count() }
    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) { self.origin.emit_face(index, target); }
    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize { self.origin.emit_modified_face(target) }
    fn pending_slices(&self) -> bool { self.origin.pending_slices() }
    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> { self.origin.drain_modified_slice(target) }
    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) { self.origin.emit_meadow_around(center, view_dir, target); }
    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> { self.origin.arboreal_perch_near(center, selector) }
    fn visible_slice_mask(&self, position: Vec3) -> u32 { self.origin.visible_slice_mask(position) }
    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) { self.origin.drain_disclosures(out); }
    fn sync_presence(&mut self, presence: CompanionPresence) { self.origin.sync_presence(presence); }
    fn sync_pilot_craft(&mut self, craft: PilotCraft) { self.origin.sync_pilot_craft(craft); }
    fn active_companions(&self, out: &mut Vec<CompanionPresence>) { self.origin.active_companions(out); }
    fn active_crafts(&self, out: &mut Vec<DrivenCraft>) { self.origin.active_crafts(out); }
    fn blend_companions(&mut self, subtick_secs: f32) { self.origin.blend_companions(subtick_secs); }
    fn portal_ticket(&self) -> Option<String> { self.origin.portal_ticket() }
    fn portal_phrase(&self) -> Option<String> { self.origin.portal_phrase() }
    fn punch_nat_peer(&mut self, peer: std::net::SocketAddr) -> bool { self.origin.punch_nat_peer(peer) }
    fn pump_portal(&mut self, dt: f32) -> bool { self.origin.pump_portal(dt) }
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

    fn face_patch_count(&self, slice: usize) -> usize { self.origin.face_patch_count(slice) }
    fn emit_face_patch(&self, slice: usize, patch_slot: usize, target: &mut dyn MeshTarget) { self.origin.emit_face_patch(slice, patch_slot, target); }
    fn drain_modified_patch(&mut self, target: &mut dyn MeshTarget) -> Option<(usize, usize)> { self.origin.drain_modified_patch(target) }
    fn drain_stale_meadow_patch(&self, center: Vec3, ring: &mut MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        self.origin.drain_stale_meadow_patch(center, ring, target)
    }
}
