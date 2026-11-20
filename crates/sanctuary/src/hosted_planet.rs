use std::net::SocketAddr;
use glam::Vec3;
use world::{
    BlockMedium, BlockMold, CleaveOutcome, CompanionPresence, CraftIdentity, CraftPose, DeckFrame,
    DrivenCraft, ExposedStratum, GravityField, MeadowRing, MeshTarget, PilotCraft, PlanetaryWorld,
    SurfaceFind, VehicleAssembly, WorldEarthworks,
};
use crate::sanctuary_host::SanctuaryHost;

/// Decorator turning any authoritative `PlanetaryWorld` into a live multiplayer host via `ChronicledPlanet`.
pub struct HostedPlanet {
    host: SanctuaryHost,
}

impl HostedPlanet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(host: SanctuaryHost) -> Self {
        Self { host }
    }
}

impl GravityField for HostedPlanet {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        self.host.world().pull_direction_at(position)
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.host.world().surface_up_at(position)
    }

    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 {
        self.host.world().apply_pull(position, dt)
    }
}

impl PlanetaryWorld for HostedPlanet {
    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.host.world().emit_surface(target);
    }

    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.host.constrain_with_crafts(position, player_height)
    }

    fn canopy_drag(&self, position: Vec3) -> f32 {
        self.host.world().canopy_drag(position)
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.host.world().placed_camera_tripod()
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let regrown = self.host.world_mut().step_surface(dt);
        let net_changed = self.host.pump_network(dt);
        regrown || net_changed || self.host.world().pending_slices()
    }

    fn step_local_flora(&mut self, dt: f32) -> bool {
        self.host.world_mut().step_surface(dt)
    }

    fn lethal_at(&self, position: Vec3) -> bool {
        self.host.world().lethal_at(position)
    }

    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 {
        self.host.world().confine_beneath_crust(position)
    }

    fn face_count(&self) -> usize {
        self.host.world().face_count()
    }

    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) {
        self.host.world().emit_face(index, target);
    }

    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        self.host.world().emit_modified_face(target)
    }

    fn pending_slices(&self) -> bool {
        self.host.world().pending_slices()
    }

    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> {
        self.host.world_mut().drain_modified_slice(target)
    }

    fn face_patch_count(&self, slice: usize) -> usize {
        self.host.world().face_patch_count(slice)
    }

    fn emit_face_patch(&self, slice: usize, patch_slot: usize, target: &mut dyn MeshTarget) {
        self.host.world().emit_face_patch(slice, patch_slot, target);
    }

    fn drain_modified_patch(&mut self, target: &mut dyn MeshTarget) -> Option<(usize, usize)> {
        self.host.world_mut().drain_modified_patch(target)
    }

    fn drain_stale_meadow_patch(&self, center: Vec3, ring: &mut MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        self.host.world().drain_stale_meadow_patch(center, ring, target)
    }

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        self.host.world().emit_meadow_around(center, view_dir, target);
    }

    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> {
        self.host.world().arboreal_perch_near(center, selector)
    }

    fn woodland_habitat(&self, position: Vec3) -> bool {
        self.host.world().woodland_habitat(position)
    }

    fn local_body_point(&self, position: Vec3) -> Vec3 {
        self.host.world().local_body_point(position)
    }

    fn visible_slice_mask(&self, position: Vec3) -> u32 {
        self.host.world().visible_slice_mask(position)
    }

    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) {
        self.host.world_mut().drain_disclosures(out);
    }

    fn sync_presence(&mut self, presence: CompanionPresence) {
        self.host.sync_host_presence(presence);
    }

    fn sync_pilot_craft(&mut self, craft: PilotCraft) {
        self.host.sync_host_craft(craft);
    }

    fn active_companions(&self, out: &mut Vec<CompanionPresence>) {
        self.host.guest_companions(out);
    }

    fn active_crafts(&self, out: &mut Vec<DrivenCraft>) {
        self.host.guest_crafts(out);
    }

    fn emit_active_crafts(&self, target: &mut dyn MeshTarget) {
        self.host.world().emit_active_crafts(target);
        self.host.emit_guest_crafts(target);
    }

    fn present_vessels(&self, sink: &mut dyn world::VesselSink) {
        self.host.world().present_vessels(sink);
        self.host.present_guest_vessels(sink);
    }

    fn plan_molded_block(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<world::PlannedMount> {
        self.host.world().plan_molded_block(eye, direction, avatar_pos, mold)
    }

    fn board_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        self.host.board_passenger_seat(eye, direction)
    }

    fn passenger_seat_frame(&self, craft: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> {
        self.host.passenger_seat_frame(craft, local_seat)
    }

    fn sample_deck_frame(&self, position: Vec3, player_height: f32) -> Option<DeckFrame> {
        self.host.sample_deck_frame(position, player_height)
    }

    fn resolve_deck_pose(&self, craft: CraftIdentity) -> Option<(CraftPose, f32)> {
        self.host.resolve_deck_pose(craft)
    }

    fn constrain_on_craft(&self, craft: CraftIdentity, position: Vec3, player_height: f32) -> Option<Vec3> {
        self.host.constrain_on_craft(craft, position, player_height)
    }

    fn blend_companions(&mut self, subtick_secs: f32) {
        self.host.blend_companions(subtick_secs);
    }

    fn portal_ticket(&self) -> Option<String> {
        Some(self.host.ticket().encode())
    }

    fn portal_phrase(&self) -> Option<String> {
        Some(self.host.code().encode())
    }

    fn punch_nat_peer(&mut self, peer: SocketAddr) -> bool {
        self.host.punch_nat_peer(peer);
        true
    }

    fn pump_portal(&mut self, dt: f32) -> bool {
        self.host.pump_network(dt) || self.host.world().pending_slices()
    }
}

impl WorldEarthworks for HostedPlanet {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.host.with_chronicled(|w| w.harvest_surface(eye, direction))
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        self.host.with_chronicled(|w| w.loosen_surface(eye, direction, stage))
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        self.host.with_chronicled(|w| w.strike_timber(eye, direction))
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        self.host.with_chronicled(|w| w.forage_surface(eye, direction))
    }

    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        self.host.with_chronicled(|w| w.cast_surface_find(eye, direction, find))
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        self.host.with_chronicled(|w| w.place_molded_surface_block(eye, direction, avatar_pos, mold))
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        self.host.with_chronicled(|w| w.adjust_rig(eye, direction, delta_steps, rotate_quarter))
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.host.with_chronicled(|w| w.lower_rig(eye, direction))
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        if holding_block {
            self.host.with_chronicled(|w| w.board_vehicle(eye, direction, true))
        } else {
            self.host.world_mut().board_vehicle(eye, direction, false)
        }
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        self.host.with_chronicled(|w| w.park_vehicle(assembly))
    }

    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        self.host.world_mut().disclose_stratum(face_index, stratum);
    }
}
