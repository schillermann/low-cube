mod ambient;

use std::net::SocketAddr;
use glam::Vec3;
use world::{
    BlockMedium, BlockMold, CleaveOutcome, CompanionPortal, CompanionPresence, CraftBlueprint,
    CraftIdentity, CraftPose, DeckFrame, DrivenCraft, ExposedStratum, GravityField, MeadowRing,
    MeshTarget, PilotCraft, PlannedMount, PlanetaryWorld, SightCone, SurfaceFind, TerrainMesh,
    VehicleAssembly, VesselDeck, VesselSink, WoodlandLife, WorldEarthworks,
};
use crate::WoodlandFauna;
pub use self::ambient::{AmbientRealm, FaunaRealm};

/// Multiplayer `PlanetaryWorld` decorator layering `AmbientRealm` over synchronized world boars.
pub type AmbientWoodland = WoodlandPlanet<AmbientRealm>;

/// `PlanetaryWorld` decorator uniting planetary terrain with a polymorphic `FaunaRealm`.
pub struct WoodlandPlanet<R = WoodlandFauna> {
    origin: Box<dyn PlanetaryWorld>,
    fauna: R,
}

impl<R> WoodlandPlanet<R> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn PlanetaryWorld>, fauna: R) -> Self {
        Self { origin, fauna }
    }

    pub const fn fauna(&self) -> &R {
        &self.fauna
    }
}

impl WoodlandPlanet<WoodlandFauna> {
    /// Secondary constructor seeding woodland sparrows, Temperate roe deer, and Temperate wild boars onto `origin`.
    pub fn wrapping(origin: Box<dyn PlanetaryWorld>) -> Self {
        Self::new(origin, WoodlandFauna::seeded())
    }
}

impl WoodlandPlanet<AmbientRealm> {
    /// Secondary constructor layering seeded ambient woodland birds and roe deer over `origin`.
    pub fn ambient(origin: Box<dyn PlanetaryWorld>) -> Self {
        Self::new(origin, AmbientRealm::seeded())
    }
}

impl<R> GravityField for WoodlandPlanet<R> {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 { self.origin.pull_direction_at(position) }
    fn surface_up_at(&self, position: Vec3) -> Vec3 { self.origin.surface_up_at(position) }
    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 { self.origin.apply_pull(position, dt) }
}

impl<R: FaunaRealm> WorldEarthworks for WoodlandPlanet<R> {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> { self.origin.harvest_surface(eye, direction) }
    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> { self.origin.loosen_surface(eye, direction, stage) }
    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome { self.origin.strike_timber(eye, direction) }
    fn strike_boar(&mut self, eye: Vec3, direction: Vec3, power: u8, max_reach: f32) -> (bool, bool) {
        self.fauna.strike_realm(eye, direction, power, max_reach, self.origin.as_mut())
    }
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
        self.origin.board_vehicle(eye, direction, holding_block)
    }
    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool { self.origin.park_vehicle(assembly) }
    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        self.origin.build_rig_blueprint(eye, direction, blueprint)
    }
    fn register_craft_blueprint(&mut self, blueprint: CraftBlueprint) -> bool { self.origin.register_craft_blueprint(blueprint) }
    fn claim_vehicle_helm(&mut self, craft: CraftIdentity, pilot_slot: u32, eye: Vec3, direction: Vec3) -> bool {
        self.origin.claim_vehicle_helm(craft, pilot_slot, eye, direction)
    }
    fn park_craft_by_id(&mut self, craft: CraftIdentity, pose: CraftPose) -> bool { self.origin.park_craft_by_id(craft, pose) }
    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) { self.origin.disclose_stratum(face_index, stratum); }
    fn offworld_frame(&self, probe: Vec3) -> Option<(world::BodyId, Vec3)> { self.origin.offworld_frame(probe) }
    fn body_center(&self, body: world::BodyId) -> Option<Vec3> { self.origin.body_center(body) }
    fn enact_offworld(&mut self, body: world::BodyId, fact: &world::WorldFact) -> bool {
        self.origin.enact_offworld(body, fact)
    }
}

impl<R> TerrainMesh for WoodlandPlanet<R> {
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
}

impl<R: FaunaRealm> WoodlandLife for WoodlandPlanet<R> {
    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> { self.origin.arboreal_perch_near(center, selector) }
    fn woodland_habitat(&self, position: Vec3) -> bool { self.origin.woodland_habitat(position) }
    fn step_wildlife(&mut self, dt: f32, listener: Vec3, view_dir: Vec3, crouching: bool) -> Option<Vec3> {
        self.fauna.step_realm(dt, listener, view_dir, crouching, self.origin.as_mut())
    }
    fn emit_wildlife(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        self.fauna.emit_realm(cone, self.origin.as_ref(), target);
    }
}

impl<R> VesselDeck for WoodlandPlanet<R> {
    fn board_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        self.origin.board_passenger_seat(eye, direction)
    }
    fn passenger_seat_frame(&self, craft: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> {
        self.origin.passenger_seat_frame(craft, local_seat)
    }
    fn sample_deck_frame(&self, position: Vec3, player_height: f32) -> Option<DeckFrame> { self.origin.sample_deck_frame(position, player_height) }
    fn resolve_deck_pose(&self, craft: CraftIdentity) -> Option<(CraftPose, f32)> { self.origin.resolve_deck_pose(craft) }
    fn constrain_on_craft(&self, craft: CraftIdentity, position: Vec3, player_height: f32) -> Option<Vec3> {
        self.origin.constrain_on_craft(craft, position, player_height)
    }
    fn helm_revoked(&mut self, craft: CraftIdentity) -> bool { self.origin.helm_revoked(craft) }
    fn emit_active_crafts(&self, target: &mut dyn MeshTarget) { self.origin.emit_active_crafts(target); }
    fn present_vessels(&self, sink: &mut dyn VesselSink) { self.origin.present_vessels(sink); }
    fn plan_molded_block(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<PlannedMount> {
        self.origin.plan_molded_block(eye, direction, avatar_pos, mold)
    }
    fn capture_vehicle_blueprint(&self, eye: Vec3, direction: Vec3) -> Option<CraftBlueprint> { self.origin.capture_vehicle_blueprint(eye, direction) }
    fn aimed_empty_rig(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool { self.origin.aimed_empty_rig(eye, direction, avatar_pos) }
}

impl<R> CompanionPortal for WoodlandPlanet<R> {
    fn sync_presence(&mut self, presence: CompanionPresence) { self.origin.sync_presence(presence); }
    fn sync_pilot_craft(&mut self, craft: PilotCraft) { self.origin.sync_pilot_craft(craft); }
    fn active_companions(&self, out: &mut Vec<CompanionPresence>) { self.origin.active_companions(out); }
    fn active_crafts(&self, out: &mut Vec<DrivenCraft>) { self.origin.active_crafts(out); }
    fn blend_companions(&mut self, subtick_secs: f32) { self.origin.blend_companions(subtick_secs); }
    fn portal_ticket(&self) -> Option<String> { self.origin.portal_ticket() }
    fn portal_phrase(&self) -> Option<String> { self.origin.portal_phrase() }
    fn punch_nat_peer(&mut self, peer: SocketAddr) -> bool { self.origin.punch_nat_peer(peer) }
    fn pump_portal(&mut self, dt: f32) -> bool { self.origin.pump_portal(dt) }
    fn utter_nearby(&mut self, text: &str) { self.origin.utter_nearby(text); }
    fn drain_utterances(&mut self, out: &mut Vec<world::ProximityUtterance>) { self.origin.drain_utterances(out); }
}

impl<R: FaunaRealm> PlanetaryWorld for WoodlandPlanet<R> {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 { self.origin.constrain_to_surface(position, player_height) }
    fn surface_contact(&self, probe: Vec3, clearance: f32) -> world::SurfaceContact { self.origin.surface_contact(probe, clearance) }
    fn canopy_drag(&self, position: Vec3) -> f32 { self.origin.canopy_drag(position) }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> { self.origin.placed_camera_tripod() }
    fn collect_spawn_points(&self, out: &mut Vec<Vec3>) { self.origin.collect_spawn_points(out); }
    fn step_surface(&mut self, dt: f32) -> bool { self.origin.step_surface(dt) }
    fn step_local_flora(&mut self, dt: f32) -> bool { self.origin.step_local_flora(dt) }
    fn lethal_at(&self, position: Vec3) -> bool { self.origin.lethal_at(position) }
    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 { self.origin.confine_beneath_crust(position) }
    fn local_body_point(&self, position: Vec3) -> Vec3 { self.origin.local_body_point(position) }
    fn celestial_atlas(&self) -> world::CelestialAtlas { self.origin.celestial_atlas() }
}
