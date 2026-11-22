use std::collections::VecDeque;
use std::time::Duration;
use glam::Vec3;
use world::{
    BlockMedium, BlockMold, CelestialSystem, CleaveOutcome, CompanionPortal, CompanionPresence,
    CraftBlueprint, CraftIdentity, CraftPose, DeckFrame, DrivenCraft, ExposedStratum, GravityField,
    MeadowRing, MeshTarget, PilotCraft, PlanetaryWorld, SightCone, SurfaceFind, TerrainMesh,
    VehicleAssembly, VesselDeck, WoodlandLife, WorldEarthworks, WorldFact,
};
use crate::chronicled_planet::ChronicledPlanet;
use crate::guest_journal::GuestJournal;
use crate::remote_link::RemoteLink;
use crate::ticket::JoinTarget;

/// Zero-knowledge P2P client decorator owning `PlanetaryWorld` lock-free alongside `GuestJournal`.
pub struct RemotePlanet {
    world: Box<dyn PlanetaryWorld>,
    journal: GuestJournal,
}

impl RemotePlanet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_journal(world: Box<dyn PlanetaryWorld>, journal: GuestJournal) -> Self {
        Self { world, journal }
    }

    pub fn new(world: Box<dyn PlanetaryWorld>, link: RemoteLink, predicted: VecDeque<WorldFact>) -> Self {
        Self::with_journal(world, GuestJournal::new(link, predicted))
    }

    pub fn from_link(link: RemoteLink, blocks: u32, bevel_margin: f32) -> Self {
        let world = CelestialSystem::hollow(blocks, bevel_margin);
        Self::new(world, link, VecDeque::new())
    }

    pub fn from_cosmos_link(link: RemoteLink, proportions: world::SolarProportions, bevel_margin: f32) -> Self {
        let world = CelestialSystem::hollow_heliosphere(proportions, bevel_margin);
        Self::new(world, link, VecDeque::new())
    }

    /// Spawns the lock-free `SocketLoop` standby heartbeat over `StandbyLink`.
    pub fn spawning(mut self) -> Self {
        self.journal.spawn_standby();
        self
    }

    pub fn connect_cosmos(
        target: JoinTarget,
        fallback: world::SolarProportions,
        bevel_margin: f32,
    ) -> std::io::Result<Self> {
        let mut link = RemoteLink::bind(target)?;
        let wait = match target {
            JoinTarget::Wormhole(_) => Duration::from_millis(650),
            JoinTarget::Ticket(_) | JoinTarget::Direct(_) => Duration::from_millis(450),
        };
        let prop = link.try_welcome_cosmos_within(wait).unwrap_or(fallback);
        if !link.connected() || link.unresolved_wormhole() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Sanctuary handshake did not complete within welcome window",
            ));
        }
        Ok(Self::from_cosmos_link(link, prop, bevel_margin))
    }

    pub fn connect(
        target: JoinTarget,
        fallback_blocks: u32,
        bevel_margin: f32,
    ) -> std::io::Result<Self> {
        let mut link = RemoteLink::bind(target)?;
        let wait = match target {
            JoinTarget::Wormhole(_) => Duration::from_millis(650),
            JoinTarget::Ticket(_) | JoinTarget::Direct(_) => Duration::from_millis(450),
        };
        let blocks = link.await_welcome(wait, fallback_blocks);
        if !link.connected() || link.unresolved_wormhole() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Sanctuary handshake did not complete within welcome window",
            ));
        }
        Ok(Self::from_link(link, blocks, bevel_margin))
    }

    pub fn dial(
        target: JoinTarget,
        blocks_per_edge: u32,
        bevel_margin: f32,
    ) -> std::io::Result<Self> {
        let link = RemoteLink::bind(target)?;
        Ok(Self::from_link(link, blocks_per_edge, bevel_margin))
    }

    pub fn connected(&self) -> bool { self.journal.connected() }

    fn with_chronicled<R>(&mut self, op: impl FnOnce(&mut ChronicledPlanet<'_>) -> R) -> R {
        let mut decorated = ChronicledPlanet::new(self.world.as_mut(), &mut self.journal);
        op(&mut decorated)
    }
}

impl GravityField for RemotePlanet {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 { self.world.pull_direction_at(position) }
    fn surface_up_at(&self, position: Vec3) -> Vec3 { self.world.surface_up_at(position) }
    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 { self.world.apply_pull(position, dt) }
}

impl WorldEarthworks for RemotePlanet {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> { self.with_chronicled(|w| w.harvest_surface(eye, direction)) }
    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> { self.with_chronicled(|w| w.loosen_surface(eye, direction, stage)) }
    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome { self.with_chronicled(|w| w.strike_timber(eye, direction)) }
    fn strike_boar(&mut self, eye: Vec3, direction: Vec3, power: u8, max_reach: f32) -> (bool, bool) {
        if !self.journal.intercept_boar_strike(eye, direction, max_reach, self.world.as_ref()) {
            return (false, false);
        }
        let reach_cm = (max_reach.clamp(0.0, 10.0) * 100.0).round() as u16;
        use crate::chronicled_planet::FactJournal as _;
        self.journal.inscribe_fact(WorldFact::BoarStruck { eye, direction, power, reach_cm });
        (true, false)
    }
    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind { self.with_chronicled(|w| w.forage_surface(eye, direction)) }
    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool { self.with_chronicled(|w| w.cast_surface_find(eye, direction, find)) }
    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool { self.with_chronicled(|w| w.place_molded_surface_block(eye, direction, avatar_pos, mold)) }
    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool { self.with_chronicled(|w| w.adjust_rig(eye, direction, delta_steps, rotate_quarter)) }
    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool { self.with_chronicled(|w| w.lower_rig(eye, direction)) }
    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        self.with_chronicled(|w| w.build_rig_blueprint(eye, direction, blueprint))
    }
    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        if holding_block {
            self.with_chronicled(|w| w.board_vehicle(eye, direction, true))
        } else {
            self.world.board_vehicle(eye, direction, false)
        }
    }
    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool { self.with_chronicled(|w| w.park_vehicle(assembly)) }
    fn park_craft_by_id(&mut self, craft: CraftIdentity, pose: CraftPose) -> bool {
        self.journal.fleet_mut().park_by_id(craft, pose, self.world.as_mut())
    }
    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        self.world.disclose_stratum(face_index, stratum);
    }
    fn offworld_frame(&self, probe: Vec3) -> Option<(world::BodyId, Vec3)> {
        self.world.offworld_frame(probe)
    }
    fn body_center(&self, body: world::BodyId) -> Option<Vec3> {
        self.world.body_center(body)
    }
    fn enact_offworld(&mut self, body: world::BodyId, fact: &WorldFact) -> bool {
        self.world.enact_offworld(body, fact)
    }
}

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
    fn emit_cosmic_field(&self, cone: SightCone, target: &mut dyn MeshTarget) { self.world.emit_cosmic_field(cone, target); }
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
    fn sync_pilot_craft(&mut self, craft: PilotCraft) { self.journal.sync_local_craft(craft); }
    fn active_companions(&self, out: &mut Vec<CompanionPresence>) { self.journal.companions(out); }
    fn active_crafts(&self, out: &mut Vec<DrivenCraft>) { self.journal.crafts(out); }
    fn blend_companions(&mut self, subtick_secs: f32) { self.journal.blend_subtick(subtick_secs); }
    fn portal_ticket(&self) -> Option<String> { self.journal.ticket_string() }
    fn portal_phrase(&self) -> Option<String> { self.journal.phrase_string() }
    fn pump_portal(&mut self, dt: f32) -> bool { self.journal.pump(dt, self.world.as_mut()) || self.world.pending_slices() }
}

impl PlanetaryWorld for RemotePlanet {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        let ground = self.world.constrain_to_surface(position, player_height);
        if self.journal.active_craft_count() == 0 {
            return ground;
        }
        let up = self.world.surface_up_at(position);
        let Some(craft_top) = self.journal.constrain_crafts(position, player_height, up) else {
            return ground;
        };
        if craft_top.dot(up) > ground.dot(up) { craft_top } else { ground }
    }
    fn canopy_drag(&self, position: Vec3) -> f32 { self.world.canopy_drag(position) }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> { self.world.placed_camera_tripod() }
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
}
