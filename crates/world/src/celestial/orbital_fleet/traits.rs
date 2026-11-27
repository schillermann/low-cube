use glam::Vec3;
use crate::{
    CompanionPortal, CompanionPresence, DrivenCraft, ExposedStratum, MeadowRing, MeshTarget,
    PilotCraft, SightCone, TerrainMesh,
};
use super::OrbitalFleetWorld;

impl<W: TerrainMesh> TerrainMesh for OrbitalFleetWorld<W> {
    fn emit_surface(&self, target: &mut dyn MeshTarget) { self.origin.emit_surface(target); }
    fn face_count(&self) -> usize { self.origin.face_count() }
    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) { self.origin.emit_face(index, target); }
    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize { self.origin.emit_modified_face(target) }
    fn pending_slices(&self) -> bool { self.origin.pending_slices() }
    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> { self.origin.drain_modified_slice(target) }
    fn face_patch_count(&self, slice: usize) -> usize { self.origin.face_patch_count(slice) }
    fn emit_face_patch(&self, slice: usize, patch_slot: usize, target: &mut dyn MeshTarget) { self.origin.emit_face_patch(slice, patch_slot, target); }
    fn drain_modified_patch(&mut self, target: &mut dyn MeshTarget) -> Option<(usize, usize)> { self.origin.drain_modified_patch(target) }
    fn drain_stale_meadow_patch(&self, center: Vec3, ring: &mut MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        self.origin.drain_stale_meadow_patch(center, ring, target)
    }
    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) { self.origin.emit_meadow_around(center, view_dir, target); }
    fn emit_cosmic_field(&self, cone: SightCone, target: &mut dyn MeshTarget) { self.origin.emit_cosmic_field(cone, target); }
    fn visible_slice_mask(&self, position: Vec3) -> u32 { self.origin.visible_slice_mask(position) }
    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) { self.origin.drain_disclosures(out); }
    fn allocated_brick_bytes(&self) -> usize { self.origin.allocated_brick_bytes() }
}

impl<W: CompanionPortal> CompanionPortal for OrbitalFleetWorld<W> {
    fn sync_presence(&mut self, presence: CompanionPresence) { self.origin.sync_presence(presence); }
    fn sync_berth_presence(&mut self, presence: CompanionPresence, berth: crate::VesselBerth) {
        self.origin.sync_berth_presence(presence, berth);
    }
    fn sync_pilot_craft(&mut self, craft: PilotCraft) { self.origin.sync_pilot_craft(craft); }
    fn sync_sky_watch(&mut self, active: bool) { self.origin.sync_sky_watch(active); }
    fn active_companions(&self, out: &mut Vec<CompanionPresence>) { self.origin.active_companions(out); }
    fn active_crafts(&self, out: &mut Vec<DrivenCraft>) { self.origin.active_crafts(out); }
    fn blend_companions(&mut self, subtick_secs: f32) { self.origin.blend_companions(subtick_secs); }
    fn portal_ticket(&self) -> Option<String> { self.origin.portal_ticket() }
    fn portal_phrase(&self) -> Option<String> { self.origin.portal_phrase() }
    fn punch_nat_peer(&mut self, peer: std::net::SocketAddr) -> bool { self.origin.punch_nat_peer(peer) }
    fn pump_portal(&mut self, dt: f32) -> bool { self.origin.pump_portal(dt) }
    fn utter_nearby(&mut self, text: &str) { self.origin.utter_nearby(text); }
    fn drain_utterances(&mut self, out: &mut Vec<crate::ProximityUtterance>) { self.origin.drain_utterances(out); }
    fn note_crafted_recipe(&mut self, recipe_index: u8) { self.origin.note_crafted_recipe(recipe_index); }
    fn note_consumed_provision(&mut self, relic_code: u8) { self.origin.note_consumed_provision(relic_code); }
}
