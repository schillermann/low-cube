use std::net::SocketAddr;
use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CelestialAtlas, CleaveOutcome, CompanionPresence, CraftBlueprint,
    CraftIdentity, CraftPose, DeckFrame, DrivenCraft, ExposedStratum, GravityField, MeadowRing,
    MeshTarget, PilotCraft, PlannedMount, ProximityUtterance, SightCone, SurfaceContact,
    SurfaceFind, VehicleAssembly, VesselSink,
};

/// Focused surface earthworks, harvesting, building, and vehicle docking contract.
pub trait WorldEarthworks {
    fn excavate_surface(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.harvest_surface(eye, direction).is_some()
    }
    fn harvest_surface(&mut self, _eye: Vec3, _direction: Vec3) -> Option<BlockMedium> { None }
    fn loosen_surface(&mut self, _eye: Vec3, _direction: Vec3, _stage: u8) -> Option<BlockMedium> { None }
    fn strike_timber(&mut self, _eye: Vec3, _direction: Vec3) -> CleaveOutcome { CleaveOutcome::Missed }
    fn strike_boar(&mut self, _eye: Vec3, _direction: Vec3, _power: u8, _max_reach: f32) -> (bool, bool) { (false, false) }
    fn forage_surface(&mut self, _eye: Vec3, _direction: Vec3) -> SurfaceFind { SurfaceFind::EmptyHanded }
    fn cast_surface_find(&mut self, _eye: Vec3, _direction: Vec3, _find: SurfaceFind) -> bool { false }
    fn place_molded_surface_block(&mut self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3, _mold: BlockMold) -> bool { false }
    fn adjust_rig(&mut self, _eye: Vec3, _direction: Vec3, _delta_steps: i32, _rotate_quarter: bool) -> bool { false }
    fn lower_rig(&mut self, _eye: Vec3, _direction: Vec3) -> bool { false }
    fn board_vehicle(&mut self, _eye: Vec3, _direction: Vec3, _holding_block: bool) -> Option<VehicleAssembly> { None }
    fn park_vehicle(&mut self, _assembly: VehicleAssembly) -> bool { false }
    fn build_rig_blueprint(&mut self, _eye: Vec3, _direction: Vec3, _blueprint: &CraftBlueprint) -> bool { false }
    fn register_craft_blueprint(&mut self, _blueprint: CraftBlueprint) -> bool { false }
    fn claim_vehicle_helm(&mut self, _craft: CraftIdentity, _pilot_slot: u32, eye: Vec3, direction: Vec3) -> bool {
        self.board_vehicle(eye, direction, true).is_some()
    }
    fn park_craft_by_id(&mut self, _craft: CraftIdentity, _pose: CraftPose) -> bool { false }
    fn disclose_stratum(&mut self, _face_index: usize, _stratum: ExposedStratum) {}
    fn offworld_frame(&self, _probe: Vec3) -> Option<(crate::BodyId, Vec3)> { None }
    fn body_center(&self, _body: crate::BodyId) -> Option<Vec3> { None }
    fn enact_offworld(&mut self, _body: crate::BodyId, fact: &crate::WorldFact) -> bool {
        fact.enact_on(self)
    }
}

/// Focused planetary terrain, `16×16` sector grid, and toroidal `MeadowRing` mesh-streaming contract.
pub trait TerrainMesh {
    fn emit_surface(&self, target: &mut dyn MeshTarget);
    fn face_count(&self) -> usize { 1 }
    fn emit_face(&self, _index: usize, target: &mut dyn MeshTarget) { self.emit_surface(target); }
    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        self.emit_surface(target);
        0
    }
    fn pending_slices(&self) -> bool { false }
    fn drain_modified_slice(&mut self, _target: &mut dyn MeshTarget) -> Option<usize> { None }
    fn face_patch_count(&self, _slice: usize) -> usize { 0 }
    fn emit_face_patch(&self, _slice: usize, _patch_slot: usize, _target: &mut dyn MeshTarget) {}
    fn drain_modified_patch(&mut self, _target: &mut dyn MeshTarget) -> Option<(usize, usize)> { None }
    fn drain_stale_meadow_patch(&self, _center: Vec3, _ring: &mut MeadowRing, _target: &mut dyn MeshTarget) -> Option<usize> { None }
    fn emit_meadow_around(&self, _center: Vec3, _view_dir: Vec3, _target: &mut dyn MeshTarget) {}
    fn emit_cosmic_field(&self, _cone: SightCone, _target: &mut dyn MeshTarget) {}
    fn visible_slice_mask(&self, _position: Vec3) -> u32 { u32::MAX }
    fn drain_disclosures(&mut self, _out: &mut Vec<(u8, ExposedStratum)>) {}
}

/// Focused woodland habitat query and living fauna simulation/emission contract.
pub trait WoodlandLife {
    fn arboreal_perch_near(&self, _center: Vec3, _selector: usize) -> Option<Vec3> { None }
    fn woodland_habitat(&self, _position: Vec3) -> bool { true }
    fn step_wildlife(&mut self, _dt: f32, _listener: Vec3, _view_dir: Vec3, _crouching: bool) -> Option<Vec3> { None }
    fn emit_wildlife(&self, _cone: SightCone, _target: &mut dyn MeshTarget) {}
}

/// Focused active vehicle deck, passenger seating, hull sink, and mount-hologram contract.
pub trait VesselDeck {
    fn board_passenger_seat(&self, _eye: Vec3, _direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> { None }
    fn passenger_seat_frame(&self, _craft: CraftIdentity, _local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> { None }
    fn sample_deck_frame(&self, _position: Vec3, _player_height: f32) -> Option<DeckFrame> { None }
    fn resolve_deck_pose(&self, _craft: CraftIdentity) -> Option<(CraftPose, f32)> { None }
    fn constrain_on_craft(&self, _craft: CraftIdentity, _position: Vec3, _player_height: f32) -> Option<Vec3> { None }
    fn helm_revoked(&mut self, _craft: CraftIdentity) -> bool { false }
    fn emit_active_crafts(&self, _target: &mut dyn MeshTarget) {}
    fn present_vessels(&self, _sink: &mut dyn VesselSink) {}
    fn present_vessels_near(&self, _center: Vec3, _radius: f32, sink: &mut dyn VesselSink) {
        self.present_vessels(sink);
    }
    fn plan_molded_block(&self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3, _mold: BlockMold) -> Option<PlannedMount> { None }
    fn capture_vehicle_blueprint(&self, _eye: Vec3, _direction: Vec3) -> Option<CraftBlueprint> { None }
    fn aimed_empty_rig(&self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3) -> bool { false }
}

/// Focused multiplayer companion presence, active fleet, and P2P portal session contract.
pub trait CompanionPortal {
    fn sync_presence(&mut self, _presence: CompanionPresence) {}
    fn sync_berth_presence(&mut self, presence: CompanionPresence, _berth: crate::VesselBerth) {
        self.sync_presence(presence);
    }
    fn sync_pilot_craft(&mut self, _craft: PilotCraft) {}
    fn active_companions(&self, _out: &mut Vec<CompanionPresence>) {}
    fn active_crafts(&self, _out: &mut Vec<DrivenCraft>) {}
    fn blend_companions(&mut self, _subtick_secs: f32) {}
    fn portal_ticket(&self) -> Option<String> { None }
    fn portal_phrase(&self) -> Option<String> { None }
    fn punch_nat_peer(&mut self, _peer: SocketAddr) -> bool { false }
    fn pump_portal(&mut self, _dt: f32) -> bool { false }
    fn utter_nearby(&mut self, _text: &str) {}
    fn drain_utterances(&mut self, _out: &mut Vec<ProximityUtterance>) {}
}

/// Composite planetary body interface uniting physical relief with segregated mesh, fauna, vessel, and portal contracts.
pub trait PlanetaryWorld:
    GravityField + WorldEarthworks + TerrainMesh + WoodlandLife + VesselDeck + CompanionPortal + Send
{
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3;
    fn surface_contact(&self, probe: Vec3, clearance: f32) -> SurfaceContact {
        SurfaceContact::SolidCrust {
            ground: self.constrain_to_surface(probe, clearance),
            normal: self.surface_up_at(probe),
        }
    }
    fn canopy_drag(&self, _position: Vec3) -> f32 { 0.0 }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> { None }
    fn collect_spawn_points(&self, _out: &mut Vec<Vec3>) {}
    fn assigned_spawn_point(&self) -> Option<Vec3> {
        let mut pts = Vec::new();
        self.collect_spawn_points(&mut pts);
        pts.into_iter().next().map(|p| p + self.surface_up_at(p))
    }
    fn step_surface(&mut self, _dt: f32) -> bool { false }
    fn step_local_flora(&mut self, dt: f32) -> bool { self.step_surface(dt) }
    fn lethal_at(&self, _position: Vec3) -> bool { false }
    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 { position }
    fn local_body_point(&self, position: Vec3) -> Vec3 { position }
    fn celestial_atlas(&self) -> CelestialAtlas { CelestialAtlas::solitary(16.0) }
    fn cosmic_frame(&self) -> crate::CosmicFrame { crate::CosmicFrame::earth() }
    fn align_epoch(&mut self, _epoch_secs: f64) {}
    fn reconcile_cosmic_frame(&mut self, _local_pos: Vec3, _local_vel: Vec3) -> Option<crate::FrameHandoff> {
        None
    }
}
