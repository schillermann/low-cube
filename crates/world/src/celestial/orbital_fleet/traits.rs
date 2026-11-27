use glam::Vec3;
use crate::{
    BlockMold, CompanionPortal, CompanionPresence, CraftBlueprint, CraftIdentity, CraftPose,
    DeckDynamics, DeckFrame, DrivenCraft, ExposedStratum, MeadowRing, MeshTarget, PilotCraft,
    PlanetaryWorld, PlannedMount, SightCone, TerrainMesh, VesselBerth, VesselDeck, VesselSink,
};
use super::OrbitalFleetWorld;

impl<W: TerrainMesh> TerrainMesh for OrbitalFleetWorld<W> {
    fn emit_surface(&self, target: &mut dyn MeshTarget) { self.inner().emit_surface(target); }
    fn face_count(&self) -> usize { self.inner().face_count() }
    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) { self.inner().emit_face(index, target); }
    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize { self.inner().emit_modified_face(target) }
    fn pending_slices(&self) -> bool { self.inner().pending_slices() }
    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> { self.inner_mut().drain_modified_slice(target) }
    fn face_patch_count(&self, slice: usize) -> usize { self.inner().face_patch_count(slice) }
    fn emit_face_patch(&self, slice: usize, patch_slot: usize, target: &mut dyn MeshTarget) { self.inner().emit_face_patch(slice, patch_slot, target); }
    fn drain_modified_patch(&mut self, target: &mut dyn MeshTarget) -> Option<(usize, usize)> { self.inner_mut().drain_modified_patch(target) }
    fn drain_stale_meadow_patch(&self, center: Vec3, ring: &mut MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        self.inner().drain_stale_meadow_patch(center, ring, target)
    }
    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) { self.inner().emit_meadow_around(center, view_dir, target); }
    fn emit_cosmic_field(&self, cone: SightCone, target: &mut dyn MeshTarget) { self.inner().emit_cosmic_field(cone, target); }
    fn visible_slice_mask(&self, position: Vec3) -> u32 { self.inner().visible_slice_mask(position) }
    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) { self.inner_mut().drain_disclosures(out); }
    fn allocated_brick_bytes(&self) -> usize { self.inner().allocated_brick_bytes() }
}

impl<W: CompanionPortal> CompanionPortal for OrbitalFleetWorld<W> {
    fn sync_presence(&mut self, presence: CompanionPresence) { self.inner_mut().sync_presence(presence); }
    fn sync_berth_presence(&mut self, presence: CompanionPresence, berth: crate::VesselBerth) {
        self.inner_mut().sync_berth_presence(presence, berth);
    }
    fn sync_pilot_craft(&mut self, craft: PilotCraft) { self.inner_mut().sync_pilot_craft(craft); }
    fn sync_sky_watch(&mut self, active: bool) { self.inner_mut().sync_sky_watch(active); }
    fn active_companions(&self, out: &mut Vec<CompanionPresence>) { self.inner().active_companions(out); }
    fn active_crafts(&self, out: &mut Vec<DrivenCraft>) { self.inner().active_crafts(out); }
    fn blend_companions(&mut self, subtick_secs: f32) { self.inner_mut().blend_companions(subtick_secs); }
    fn portal_ticket(&self) -> Option<String> { self.inner().portal_ticket() }
    fn portal_phrase(&self) -> Option<String> { self.inner().portal_phrase() }
    fn punch_nat_peer(&mut self, peer: std::net::SocketAddr) -> bool { self.inner_mut().punch_nat_peer(peer) }
    fn pump_portal(&mut self, dt: f32) -> bool { self.inner_mut().pump_portal(dt) }
    fn utter_nearby(&mut self, text: &str) { self.inner_mut().utter_nearby(text); }
    fn drain_utterances(&mut self, out: &mut Vec<crate::ProximityUtterance>) { self.inner_mut().drain_utterances(out); }
    fn note_crafted_recipe(&mut self, recipe_index: u8) { self.inner_mut().note_crafted_recipe(recipe_index); }
    fn note_consumed_provision(&mut self, relic_code: u8) { self.inner_mut().note_consumed_provision(relic_code); }
}

impl<W: VesselDeck> VesselDeck for OrbitalFleetWorld<W> {
    fn board_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        self.fleet()
            .find_passenger_seat(eye, direction)
            .or_else(|| self.inner().board_passenger_seat(eye, direction))
    }

    fn passenger_seat_frame(&self, craft: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> {
        self.fleet()
            .passenger_seat_frame(craft, local_seat)
            .or_else(|| self.inner().passenger_seat_frame(craft, local_seat))
    }

    fn sample_deck_frame(&self, position: Vec3, player_height: f32) -> Option<DeckFrame> {
        self.fleet()
            .sample_deck_at(position, player_height)
            .or_else(|| self.inner().sample_deck_frame(position, player_height))
    }

    fn resolve_deck_pose(&self, id: CraftIdentity) -> Option<(CraftPose, f32)> {
        self.fleet()
            .resolve_deck_pose(id)
            .or_else(|| self.inner().resolve_deck_pose(id))
    }

    fn resolve_deck_dynamics(&self, id: CraftIdentity) -> Option<(CraftPose, DeckDynamics, Vec3)> {
        if let Some(body) = self.fleet().find(id) {
            return Some((body.pose(), body.deck_dynamics(), body.motion().linear_vel()));
        }
        self.inner().resolve_deck_dynamics(id)
    }

    fn resolve_nested_berth(&self, berth: VesselBerth) -> VesselBerth {
        let Some(id) = berth.craft_id() else { return VesselBerth::planetary() };
        let root = self.fleet().root_inertial_host(id);
        if root != id {
            VesselBerth::nested(id, root)
        } else {
            self.inner().resolve_nested_berth(berth)
        }
    }

    fn couple_docking_rings(&mut self) -> bool {
        self.fleet_mut().couple_nearby_docking_rings()
    }

    fn undock_craft(&mut self, craft: CraftIdentity) -> bool {
        self.fleet_mut().undock_body(craft)
    }

    fn constrain_on_craft(&self, id: CraftIdentity, position: Vec3, player_height: f32) -> Option<Vec3> {
        self.fleet()
            .constrain_craft_by_id(id, position, player_height)
            .or_else(|| self.inner().constrain_on_craft(id, position, player_height))
    }

    fn helm_revoked(&mut self, craft: CraftIdentity) -> bool { self.inner_mut().helm_revoked(craft) }

    fn emit_active_crafts(&self, target: &mut dyn MeshTarget) {
        self.inner().emit_active_crafts(target);
        self.fleet().emit_all(target);
    }

    fn present_vessels(&self, sink: &mut dyn VesselSink) {
        self.inner().present_vessels(sink);
        self.fleet().present_vessels(sink);
    }

    fn present_vessels_near(&self, center: Vec3, radius: f32, sink: &mut dyn VesselSink) {
        self.inner().present_vessels_near(center, radius, sink);
        let r_sq = radius * radius;
        for body in self.fleet().bodies() {
            if (body.pose().origin() - center).length_squared() <= r_sq {
                sink.receive_vessel(body.blueprint(), body.pose());
            }
        }
    }

    fn plan_molded_block(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<PlannedMount> {
        self.inner().plan_molded_block(eye, direction, avatar_pos, mold)
    }

    fn capture_vehicle_blueprint(&self, eye: Vec3, direction: Vec3) -> Option<CraftBlueprint> {
        self.inner().capture_vehicle_blueprint(eye, direction)
    }

    fn aimed_empty_rig(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.inner().aimed_empty_rig(eye, direction, avatar_pos)
    }
}

impl<W: PlanetaryWorld> PlanetaryWorld for OrbitalFleetWorld<W> {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        let ground = self.inner().constrain_to_surface(position, player_height);
        let up = self.inner().surface_up_at(position);
        let mut best = ground;
        for body in self.fleet().bodies() {
            if let Some(supported) = body.blueprint().constrain_at(body.pose(), position, player_height) {
                if supported.dot(up) > best.dot(up) {
                    best = supported;
                }
            }
        }
        best
    }

    fn surface_contact(&self, probe: Vec3, clearance: f32) -> crate::SurfaceContact {
        self.inner().surface_contact(probe, clearance)
    }

    fn canopy_drag(&self, position: Vec3) -> f32 { self.inner().canopy_drag(position) }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> { self.inner().placed_camera_tripod() }
    fn collect_medium_sites(&self, medium: crate::BlockMedium, out: &mut Vec<(Vec3, Vec3)>) {
        self.inner().collect_medium_sites(medium, out);
    }
    fn collect_offworld_medium_sites(&self, medium: crate::BlockMedium, out: &mut Vec<(crate::BodyId, Vec3, Vec3)>) {
        self.inner().collect_offworld_medium_sites(medium, out);
    }
    fn assigned_spawn_point(&self) -> Option<Vec3> { self.inner().assigned_spawn_point() }
    fn assigned_survivor(&self) -> Option<crate::SurvivorWelcome> { self.inner().assigned_survivor() }
    fn anchor_to_frame(&mut self, frame: crate::CosmicFrame, local_pos: Vec3) {
        self.inner_mut().anchor_to_frame(frame, local_pos);
    }
    fn step_surface(&mut self, dt: f32) -> bool {
        let origin_moved = self.inner_mut().step_surface(dt);
        let fleet_moved = self.step_fleet_six_dof(dt);
        origin_moved || fleet_moved
    }
    fn step_active_zones(&mut self, dt: f32, earth_zone_mask: u8, active_bodies: &[crate::BodyId]) -> bool {
        let origin_moved = self.inner_mut().step_active_zones(dt, earth_zone_mask, active_bodies);
        let fleet_moved = self.step_fleet_six_dof(dt);
        origin_moved || fleet_moved
    }
    fn step_local_flora(&mut self, dt: f32) -> bool { self.inner_mut().step_local_flora(dt) }
    fn lethal_at(&self, position: Vec3) -> bool { self.inner().lethal_at(position) }
    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 { self.inner().confine_beneath_crust(position) }
    fn local_body_point(&self, position: Vec3) -> Vec3 { self.inner().local_body_point(position) }
    fn celestial_atlas(&self) -> crate::CelestialAtlas { self.inner().celestial_atlas() }
    fn cosmic_frame(&self) -> crate::CosmicFrame { self.inner().cosmic_frame() }
    fn align_epoch(&mut self, epoch_secs: f64) { self.inner_mut().align_epoch(epoch_secs); }
    fn reconcile_cosmic_frame(&mut self, local_pos: Vec3, local_vel: Vec3) -> Option<crate::FrameHandoff> {
        let handoff = self.inner_mut().reconcile_cosmic_frame(local_pos, local_vel)?;
        self.fleet_mut().apply_frame_shift(handoff.pos_shift(), Vec3::ZERO);
        Some(handoff)
    }
}
