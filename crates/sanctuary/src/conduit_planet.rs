use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use glam::Vec3;
use world::{
    BlockMedium, BlockMold, CleaveOutcome, CompanionPresence, CraftBlueprint, CraftIdentity,
    CraftPose, DeckFrame, DrivenCraft, ExposedStratum, GravityField, MeadowRing, MeshTarget,
    PilotCraft, PlanetaryWorld, SurfaceFind, VehicleAssembly, WorldEarthworks,
};
use crate::mailbox::{PortalMailbox, SocketLoop};

/// `PlanetaryWorld` decorator decoupling P2P UDP heartbeats from the render loop via `SocketLoop`.
pub struct ConduitPlanet {
    shared: Arc<Mutex<Box<dyn PlanetaryWorld>>>,
    mailbox: Arc<PortalMailbox>,
    socket_loop: SocketLoop,
}

impl ConduitPlanet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        shared: Arc<Mutex<Box<dyn PlanetaryWorld>>>,
        mailbox: Arc<PortalMailbox>,
        socket_loop: SocketLoop,
    ) -> Self {
        Self { shared, mailbox, socket_loop }
    }

    /// Secondary constructor wrapping an authoritative or remote planet with a 120 Hz `SocketLoop`.
    pub fn spawning(world: Box<dyn PlanetaryWorld>) -> Self {
        let shared = Arc::new(Mutex::new(world));
        let mailbox = Arc::new(PortalMailbox::active());
        let socket_loop = SocketLoop::spawn(Arc::clone(&shared), Arc::clone(&mailbox));
        Self::new(shared, mailbox, socket_loop)
    }

    pub fn active(&self) -> bool {
        self.mailbox.running() && std::mem::size_of_val(&self.socket_loop) > 0
    }

    fn read_world<R>(&self, op: impl FnOnce(&dyn PlanetaryWorld) -> R) -> R {
        let guard = self.shared.lock().expect("ConduitPlanet mutex poisoned");
        op(guard.as_ref())
    }

    fn write_world<R>(&mut self, op: impl FnOnce(&mut dyn PlanetaryWorld) -> R) -> R {
        let mut guard = self.shared.lock().expect("ConduitPlanet mutex poisoned");
        op(guard.as_mut())
    }
}

impl GravityField for ConduitPlanet {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        self.read_world(|w| w.pull_direction_at(position))
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.read_world(|w| w.surface_up_at(position))
    }

    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 {
        self.read_world(|w| w.apply_pull(position, dt))
    }
}

impl WorldEarthworks for ConduitPlanet {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> { self.write_world(|w| w.harvest_surface(eye, direction)) }
    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> { self.write_world(|w| w.loosen_surface(eye, direction, stage)) }
    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome { self.write_world(|w| w.strike_timber(eye, direction)) }
    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind { self.write_world(|w| w.forage_surface(eye, direction)) }
    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool { self.write_world(|w| w.cast_surface_find(eye, direction, find)) }
    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool { self.write_world(|w| w.place_molded_surface_block(eye, direction, avatar_pos, mold)) }
    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool { self.write_world(|w| w.adjust_rig(eye, direction, delta_steps, rotate_quarter)) }
    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool { self.write_world(|w| w.lower_rig(eye, direction)) }
    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> { self.write_world(|w| w.board_vehicle(eye, direction, holding_block)) }
    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool { self.write_world(|w| w.park_vehicle(assembly)) }
    fn register_craft_blueprint(&mut self, blueprint: CraftBlueprint) -> bool { self.write_world(|w| w.register_craft_blueprint(blueprint)) }
    fn claim_vehicle_helm(&mut self, craft: CraftIdentity, pilot_slot: u32, eye: Vec3, direction: Vec3) -> bool { self.write_world(|w| w.claim_vehicle_helm(craft, pilot_slot, eye, direction)) }
    fn park_craft_by_id(&mut self, craft: CraftIdentity, pose: CraftPose) -> bool { self.write_world(|w| w.park_craft_by_id(craft, pose)) }
    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) { self.write_world(|w| w.disclose_stratum(face_index, stratum)); }
}

impl PlanetaryWorld for ConduitPlanet {
    fn emit_surface(&self, target: &mut dyn MeshTarget) { self.read_world(|w| w.emit_surface(target)); }
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 { self.read_world(|w| w.constrain_to_surface(position, player_height)) }
    fn canopy_drag(&self, position: Vec3) -> f32 { self.read_world(|w| w.canopy_drag(position)) }
    fn board_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> { self.read_world(|w| w.board_passenger_seat(eye, direction)) }
    fn passenger_seat_frame(&self, craft: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> { self.read_world(|w| w.passenger_seat_frame(craft, local_seat)) }
    fn sample_deck_frame(&self, position: Vec3, player_height: f32) -> Option<DeckFrame> { self.read_world(|w| w.sample_deck_frame(position, player_height)) }
    fn resolve_deck_pose(&self, craft: CraftIdentity) -> Option<(CraftPose, f32)> { self.read_world(|w| w.resolve_deck_pose(craft)) }
    fn constrain_on_craft(&self, craft: CraftIdentity, position: Vec3, player_height: f32) -> Option<Vec3> { self.read_world(|w| w.constrain_on_craft(craft, position, player_height)) }
    fn helm_revoked(&mut self, craft: CraftIdentity) -> bool { self.write_world(|w| w.helm_revoked(craft)) }
    fn emit_active_crafts(&self, target: &mut dyn MeshTarget) { self.read_world(|w| w.emit_active_crafts(target)); }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> { self.read_world(|w| w.placed_camera_tripod()) }

    fn step_surface(&mut self, dt: f32) -> bool {
        let net_changed = self.mailbox.take_surface_change();
        self.write_world(|w| {
            let regrown = w.step_local_flora(dt);
            regrown || net_changed || w.pending_slices()
        })
    }

    fn step_local_flora(&mut self, dt: f32) -> bool { self.write_world(|w| w.step_local_flora(dt)) }
    fn lethal_at(&self, position: Vec3) -> bool { self.read_world(|w| w.lethal_at(position)) }
    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 { self.read_world(|w| w.confine_beneath_crust(position)) }
    fn face_count(&self) -> usize { self.read_world(|w| w.face_count()) }
    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) { self.read_world(|w| w.emit_face(index, target)); }
    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize { self.read_world(|w| w.emit_modified_face(target)) }
    fn pending_slices(&self) -> bool { self.read_world(|w| w.pending_slices()) }
    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> { self.write_world(|w| w.drain_modified_slice(target)) }
    fn face_patch_count(&self, slice: usize) -> usize { self.read_world(|w| w.face_patch_count(slice)) }
    fn emit_face_patch(&self, slice: usize, patch_slot: usize, target: &mut dyn MeshTarget) { self.read_world(|w| w.emit_face_patch(slice, patch_slot, target)); }
    fn drain_modified_patch(&mut self, target: &mut dyn MeshTarget) -> Option<(usize, usize)> { self.write_world(|w| w.drain_modified_patch(target)) }
    fn drain_stale_meadow_patch(&self, center: Vec3, ring: &mut MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> { self.read_world(|w| w.drain_stale_meadow_patch(center, ring, target)) }
    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) { self.read_world(|w| w.emit_meadow_around(center, view_dir, target)); }
    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> { self.read_world(|w| w.arboreal_perch_near(center, selector)) }
    fn woodland_habitat(&self, position: Vec3) -> bool { self.read_world(|w| w.woodland_habitat(position)) }
    fn local_body_point(&self, position: Vec3) -> Vec3 { self.read_world(|w| w.local_body_point(position)) }
    fn visible_slice_mask(&self, position: Vec3) -> u32 { self.read_world(|w| w.visible_slice_mask(position)) }
    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) { self.write_world(|w| w.drain_disclosures(out)); }
    fn sync_presence(&mut self, presence: CompanionPresence) { self.write_world(|w| w.sync_presence(presence)); }
    fn sync_pilot_craft(&mut self, craft: PilotCraft) { self.write_world(|w| w.sync_pilot_craft(craft)); }
    fn active_companions(&self, out: &mut Vec<CompanionPresence>) { self.read_world(|w| w.active_companions(out)); }
    fn active_crafts(&self, out: &mut Vec<DrivenCraft>) { self.read_world(|w| w.active_crafts(out)); }
    fn blend_companions(&mut self, subtick_secs: f32) { self.write_world(|w| w.blend_companions(subtick_secs)); }
    fn portal_ticket(&self) -> Option<String> { self.read_world(|w| w.portal_ticket()) }
    fn portal_phrase(&self) -> Option<String> { self.read_world(|w| w.portal_phrase()) }
    fn punch_nat_peer(&mut self, peer: SocketAddr) -> bool { self.write_world(|w| w.punch_nat_peer(peer)) }
    fn pump_portal(&mut self, _dt: f32) -> bool {
        self.mailbox.take_surface_change() || self.read_world(|w| w.pending_slices())
    }
}
