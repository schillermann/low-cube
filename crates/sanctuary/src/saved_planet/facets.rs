use glam::Vec3;
use world::{
    BlockMedium, BlockMold, CompanionPortal, CompanionPresence, CraftBlueprint, CraftIdentity,
    CraftPose, DeckFrame, DrivenCraft, ExposedStratum, GravityField, MeadowRing, MeshTarget,
    PilotCraft, PlanetaryWorld, SatchelManifest, SightCone, TerrainMesh, VesselDeck, WoodlandLife,
};
use super::SavedPlanet;

impl GravityField for SavedPlanet {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        self.world.pull_direction_at(position)
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.world.surface_up_at(position)
    }

    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 {
        self.world.apply_pull(position, dt)
    }
}

impl TerrainMesh for SavedPlanet {
    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.world.emit_surface(target);
    }

    fn face_count(&self) -> usize {
        self.world.face_count()
    }

    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) {
        self.world.emit_face(index, target);
    }

    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        self.world.emit_modified_face(target)
    }

    fn pending_slices(&self) -> bool {
        self.world.pending_slices()
    }

    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> {
        self.world.drain_modified_slice(target)
    }

    fn face_patch_count(&self, slice: usize) -> usize {
        self.world.face_patch_count(slice)
    }

    fn emit_face_patch(&self, slice: usize, patch_slot: usize, target: &mut dyn MeshTarget) {
        self.world.emit_face_patch(slice, patch_slot, target);
    }

    fn drain_modified_patch(&mut self, target: &mut dyn MeshTarget) -> Option<(usize, usize)> {
        self.world.drain_modified_patch(target)
    }

    fn drain_stale_meadow_patch(
        &self,
        center: Vec3,
        ring: &mut MeadowRing,
        target: &mut dyn MeshTarget,
    ) -> Option<usize> {
        self.world.drain_stale_meadow_patch(center, ring, target)
    }

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        self.world.emit_meadow_around(center, view_dir, target);
    }

    fn emit_cosmic_field(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        self.world.emit_cosmic_field(cone, target);
    }

    fn visible_slice_mask(&self, position: Vec3) -> u32 {
        self.world.visible_slice_mask(position)
    }

    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) {
        self.world.drain_disclosures(out);
    }
}

impl WoodlandLife for SavedPlanet {
    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> {
        self.world.arboreal_perch_near(center, selector)
    }

    fn woodland_habitat(&self, position: Vec3) -> bool {
        self.world.woodland_habitat(position)
    }

    fn step_wildlife(&mut self, dt: f32, listener: Vec3, view_dir: Vec3, crouching: bool) -> Option<Vec3> {
        self.world.step_wildlife(dt, listener, view_dir, crouching)
    }

    fn emit_wildlife(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        self.world.emit_wildlife(cone, target);
    }
}

impl VesselDeck for SavedPlanet {
    fn board_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        self.world.board_passenger_seat(eye, direction)
    }

    fn passenger_seat_frame(&self, craft: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> {
        self.world.passenger_seat_frame(craft, local_seat)
    }

    fn sample_deck_frame(&self, position: Vec3, player_height: f32) -> Option<DeckFrame> {
        self.world.sample_deck_frame(position, player_height)
    }

    fn resolve_deck_pose(&self, craft: CraftIdentity) -> Option<(CraftPose, f32)> {
        self.world.resolve_deck_pose(craft)
    }

    fn constrain_on_craft(&self, craft: CraftIdentity, position: Vec3, player_height: f32) -> Option<Vec3> {
        self.world.constrain_on_craft(craft, position, player_height)
    }

    fn emit_active_crafts(&self, target: &mut dyn MeshTarget) {
        self.world.emit_active_crafts(target);
    }

    fn present_vessels(&self, sink: &mut dyn world::VesselSink) {
        self.world.present_vessels(sink);
    }

    fn present_vessels_near(&self, center: Vec3, radius: f32, sink: &mut dyn world::VesselSink) {
        self.world.present_vessels_near(center, radius, sink);
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

impl CompanionPortal for SavedPlanet {
    fn sync_pilot_craft(&mut self, craft: PilotCraft) {
        self.piloted = craft;
    }

    fn active_companions(&self, out: &mut Vec<CompanionPresence>) {
        self.world.active_companions(out);
    }

    fn active_crafts(&self, out: &mut Vec<DrivenCraft>) {
        self.world.active_crafts(out);
    }

    fn commit_local_survivor(&mut self, frame: world::CosmicFrame, position: Vec3, manifest: SatchelManifest) {
        self.owner.note_state(frame, position, manifest, self.ledger.survivors_mut());
    }
}

impl PlanetaryWorld for SavedPlanet {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.world.constrain_to_surface(position, player_height)
    }

    fn surface_contact(&self, probe: Vec3, clearance: f32) -> world::SurfaceContact {
        self.world.surface_contact(probe, clearance)
    }

    fn canopy_drag(&self, position: Vec3) -> f32 {
        self.world.canopy_drag(position)
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.world.placed_camera_tripod()
    }

    fn collect_medium_sites(&self, medium: BlockMedium, out: &mut Vec<(Vec3, Vec3)>) {
        self.world.collect_medium_sites(medium, out);
    }

    fn collect_offworld_medium_sites(&self, medium: BlockMedium, out: &mut Vec<(world::BodyId, Vec3, Vec3)>) {
        self.world.collect_offworld_medium_sites(medium, out);
    }

    fn assigned_spawn_point(&self) -> Option<Vec3> {
        self.world.assigned_spawn_point()
    }

    fn assigned_survivor(&self) -> Option<world::SurvivorWelcome> {
        Some(self.owner.welcome())
    }

    fn anchor_to_frame(&mut self, frame: world::CosmicFrame, local_pos: Vec3) {
        self.world.anchor_to_frame(frame, local_pos);
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let stepped = self.world.step_surface(dt);
        self.ledger.flush_tick();
        stepped
    }

    fn step_active_zones(&mut self, dt: f32, earth_zone_mask: u8, active_bodies: &[world::BodyId]) -> bool {
        let stepped = self.world.step_active_zones(dt, earth_zone_mask, active_bodies);
        self.ledger.flush_tick();
        stepped
    }

    fn step_local_flora(&mut self, dt: f32) -> bool {
        self.world.step_local_flora(dt)
    }

    fn lethal_at(&self, position: Vec3) -> bool {
        self.world.lethal_at(position)
    }

    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 {
        self.world.confine_beneath_crust(position)
    }

    fn local_body_point(&self, position: Vec3) -> Vec3 {
        self.world.local_body_point(position)
    }

    fn celestial_atlas(&self) -> world::CelestialAtlas {
        self.world.celestial_atlas()
    }

    fn cosmic_frame(&self) -> world::CosmicFrame {
        self.world.cosmic_frame()
    }

    fn align_epoch(&mut self, epoch_secs: f64) {
        self.world.align_epoch(epoch_secs);
    }

    fn reconcile_cosmic_frame(&mut self, local_pos: Vec3, local_vel: Vec3) -> Option<world::FrameHandoff> {
        self.world.reconcile_cosmic_frame(local_pos, local_vel)
    }
}
