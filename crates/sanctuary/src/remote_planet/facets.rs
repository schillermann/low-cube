use glam::Vec3;
use world::{
    BlockMold, CompanionPortal, CompanionPresence, CraftBlueprint, CraftIdentity, CraftPose,
    DeckFrame, DrivenCraft, ExposedStratum, MeadowRing, MeshTarget, PilotCraft, PlanetaryWorld,
    SightCone, TerrainMesh, VesselDeck, WoodlandLife,
};
use super::RemotePlanet;

impl TerrainMesh for RemotePlanet {
    fn emit_surface(&self, target: &mut dyn MeshTarget) { self.world.emit_surface(target); }
    fn face_count(&self) -> usize { self.world.face_count() }
    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) { self.world.emit_face(index, target); }
    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize { self.world.emit_modified_face(target) }
    fn pending_slices(&self) -> bool { self.world.pending_slices() }
    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> { self.world.drain_modified_slice(target) }
    fn face_patch_count(&self, slice: usize) -> usize { self.world.face_patch_count(slice) }
    fn emit_face_patch(&self, slice: usize, patch_slot: usize, target: &mut dyn MeshTarget) { self.world.emit_face_patch(slice, patch_slot, target); }
    fn drain_modified_patch(&mut self, target: &mut dyn MeshTarget) -> Option<(usize, usize)> { self.world.drain_modified_patch(target) }
    fn drain_stale_meadow_patch(&self, center: Vec3, ring: &mut MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        self.world.drain_stale_meadow_patch(center, ring, target)
    }
    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) { self.world.emit_meadow_around(center, view_dir, target); }
    fn emit_cosmic_field(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        self.world.emit_cosmic_field(cone, target);
        self.journal.emit_sky_signals(cone, target);
    }
    fn visible_slice_mask(&self, position: Vec3) -> u32 { self.world.visible_slice_mask(position) }
    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) { self.world.drain_disclosures(out); }
}

impl WoodlandLife for RemotePlanet {
    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> { self.world.arboreal_perch_near(center, selector) }
    fn woodland_habitat(&self, position: Vec3) -> bool { self.world.woodland_habitat(position) }
    fn step_wildlife(&mut self, dt: f32, listener: Vec3, view_dir: Vec3, _crouching: bool) -> Option<Vec3> {
        self.journal.step_wildlife(dt, listener, view_dir, self.world.as_ref())
    }
    fn emit_wildlife(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        self.journal.emit_wildlife(cone, self.world.as_ref(), target);
    }
}

impl VesselDeck for RemotePlanet {
    fn helm_revoked(&mut self, craft: CraftIdentity) -> bool { self.journal.fleet_mut().take_revoked(craft) }
    fn board_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        let mut crafts = Vec::new();
        self.journal.crafts(&mut crafts);
        self.journal.fleet().find_passenger_seat(&crafts, eye, direction)
    }
    fn passenger_seat_frame(&self, craft: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> {
        let mut crafts = Vec::new();
        self.journal.crafts(&mut crafts);
        crafts.into_iter().find(|c| c.identity() == craft).map(|c| c.pose().world_seat_at(local_seat))
    }
    fn sample_deck_frame(&self, position: Vec3, player_height: f32) -> Option<DeckFrame> {
        self.journal
            .sample_deck_at(position, player_height)
            .or_else(|| self.world.sample_deck_frame(position, player_height))
    }
    fn resolve_deck_pose(&self, craft: CraftIdentity) -> Option<(CraftPose, f32)> {
        let mut crafts = Vec::new();
        self.journal.crafts(&mut crafts);
        self.journal
            .fleet()
            .resolve_deck_pose(&crafts, craft)
            .or_else(|| self.world.resolve_deck_pose(craft))
    }
    fn constrain_on_craft(&self, craft: CraftIdentity, position: Vec3, player_height: f32) -> Option<Vec3> {
        let mut crafts = Vec::new();
        self.journal.crafts(&mut crafts);
        self.journal
            .fleet()
            .constrain_craft_by_id(&crafts, craft, position, player_height)
            .or_else(|| self.world.constrain_on_craft(craft, position, player_height))
    }
    fn emit_active_crafts(&self, target: &mut dyn MeshTarget) {
        self.world.emit_active_crafts(target);
        if self.journal.active_craft_count() > 0 {
            let mut crafts = Vec::new();
            self.journal.crafts(&mut crafts);
            self.journal.fleet().emit_crafts(&crafts, target);
        }
    }
    fn present_vessels(&self, sink: &mut dyn world::VesselSink) {
        self.world.present_vessels(sink);
        if self.journal.active_craft_count() > 0 {
            let mut crafts = Vec::new();
            self.journal.crafts(&mut crafts);
            self.journal.fleet().present_vessels(&crafts, sink);
        }
    }
    fn present_vessels_near(&self, center: Vec3, radius: f32, sink: &mut dyn world::VesselSink) {
        self.world.present_vessels_near(center, radius, sink);
        if self.journal.active_craft_count() > 0 {
            let mut crafts = Vec::new();
            self.journal.crafts(&mut crafts);
            crafts.retain(|c| c.within_reach(center, radius));
            self.journal.fleet().present_vessels(&crafts, sink);
        }
    }
    fn plan_molded_block(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<world::PlannedMount> {
        self.world.plan_molded_block(eye, direction, avatar_pos, mold)
    }
    fn capture_vehicle_blueprint(&self, eye: Vec3, direction: Vec3) -> Option<CraftBlueprint> {
        self.world.capture_vehicle_blueprint(eye, direction)
    }
    fn aimed_empty_rig(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.world.aimed_empty_rig(eye, direction, avatar_pos)
    }
}

impl CompanionPortal for RemotePlanet {
    fn sync_presence(&mut self, presence: CompanionPresence) { self.journal.sync_local_presence(presence); }
    fn sync_berth_presence(&mut self, presence: CompanionPresence, berth: world::VesselBerth) {
        self.journal.sync_local_berth_presence(presence, berth);
    }
    fn sync_pilot_craft(&mut self, craft: PilotCraft) { self.journal.sync_local_craft(craft); }
    fn sync_sky_watch(&mut self, active: bool) { self.journal.sync_sky_watch(active); }
    fn active_companions(&self, out: &mut Vec<CompanionPresence>) { self.journal.companions(out); }
    fn active_crafts(&self, out: &mut Vec<DrivenCraft>) { self.journal.crafts(out); }
    fn blend_companions(&mut self, subtick_secs: f32) { self.journal.blend_subtick(subtick_secs); }
    fn portal_ticket(&self) -> Option<String> { self.journal.ticket_string() }
    fn portal_phrase(&self) -> Option<String> { self.journal.phrase_string() }
    fn pump_portal(&mut self, dt: f32) -> bool { self.journal.pump(dt, self.world.as_mut()) || self.world.pending_slices() }
    fn utter_nearby(&mut self, text: &str) { self.journal.utter_nearby(text); }
    fn drain_utterances(&mut self, out: &mut Vec<world::ProximityUtterance>) { self.journal.drain_utterances(out); }
    fn note_crafted_recipe(&mut self, recipe_index: u8) {
        self.journal.send_inventory_notice(crate::envelope::InventoryNotice::Crafted { recipe: recipe_index });
    }
    fn note_consumed_provision(&mut self, relic_code: u8) {
        self.journal.send_inventory_notice(crate::envelope::InventoryNotice::Consumed { relic_code });
    }
}

impl PlanetaryWorld for RemotePlanet {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        let raw_ground = self.world.constrain_to_surface(position, player_height);
        let ground = self.journal.constrain_arrival(
            position,
            player_height,
            raw_ground,
            self.world.cosmic_frame().earth_centered(),
        );
        if self.journal.active_craft_count() == 0 {
            return ground;
        }
        let up = self.world.surface_up_at(position);
        let Some(craft_top) = self.journal.constrain_crafts(position, player_height, up) else {
            return ground;
        };
        if craft_top.dot(up) > ground.dot(up) { craft_top } else { ground }
    }
    fn surface_contact(&self, probe: Vec3, clearance: f32) -> world::SurfaceContact {
        self.world.surface_contact(probe, clearance)
    }
    fn canopy_drag(&self, position: Vec3) -> f32 { self.world.canopy_drag(position) }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> { self.world.placed_camera_tripod() }
    fn collect_medium_sites(&self, medium: world::BlockMedium, out: &mut Vec<(Vec3, Vec3)>) {
        self.world.collect_medium_sites(medium, out);
    }
    fn collect_offworld_medium_sites(&self, medium: world::BlockMedium, out: &mut Vec<(world::BodyId, Vec3, Vec3)>) {
        self.world.collect_offworld_medium_sites(medium, out);
    }
    fn assigned_spawn_point(&self) -> Option<Vec3> {
        self.journal.assigned_spawn().or_else(|| self.world.assigned_spawn_point())
    }
    fn assigned_survivor(&self) -> Option<world::SurvivorWelcome> {
        self.journal.assigned_survivor()
    }
    fn anchor_to_frame(&mut self, frame: world::CosmicFrame, local_pos: Vec3) {
        self.world.anchor_to_frame(frame, local_pos);
        self.journal.anchor_local_frame(frame);
    }
    fn step_surface(&mut self, dt: f32) -> bool {
        let regrown = self.world.step_surface(dt);
        let net_changed = self.journal.pump(dt, self.world.as_mut());
        regrown || net_changed || self.world.pending_slices()
    }
    fn step_local_flora(&mut self, dt: f32) -> bool { self.world.step_surface(dt) }
    fn lethal_at(&self, position: Vec3) -> bool { self.world.lethal_at(position) }
    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 { self.world.confine_beneath_crust(position) }
    fn local_body_point(&self, position: Vec3) -> Vec3 { self.world.local_body_point(position) }
    fn celestial_atlas(&self) -> world::CelestialAtlas { self.world.celestial_atlas() }
    fn cosmic_frame(&self) -> world::CosmicFrame { self.world.cosmic_frame() }
    fn align_epoch(&mut self, epoch_secs: f64) { self.world.align_epoch(epoch_secs); }
    fn reconcile_cosmic_frame(&mut self, local_pos: Vec3, local_vel: Vec3) -> Option<world::FrameHandoff> {
        let handoff = self.world.reconcile_cosmic_frame(local_pos, local_vel)?;
        self.journal.apply_frame_handoff(handoff);
        Some(handoff)
    }
}
