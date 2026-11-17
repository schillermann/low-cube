use std::collections::VecDeque;
use std::time::Duration;
use glam::Vec3;
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, CleaveOutcome, CompanionPresence, ExposedStratum,
    GravityField, MeshTarget, PlanetaryWorld, SurfaceFind, VehicleAssembly, WorldFact,
};
use crate::chronicled_planet::{ChronicledPlanet, FactJournal};
use crate::remote_link::RemoteLink;
use crate::ticket::JoinTarget;

/// Optimistic guest petition journal pairing `RemoteLink` with a bounded prediction ring.
pub struct GuestJournal {
    link: RemoteLink,
    predicted: VecDeque<WorldFact>,
}

impl GuestJournal {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(link: RemoteLink, predicted: VecDeque<WorldFact>) -> Self {
        Self { link, predicted }
    }

    pub fn connected(&self) -> bool { self.link.connected() }
    pub fn sync_local_presence(&mut self, presence: CompanionPresence) { self.link.sync_local_presence(presence); }
    pub fn companions(&self, out: &mut Vec<CompanionPresence>) { self.link.companions(out); }
    pub fn ticket_string(&self) -> Option<String> { self.link.ticket_string() }
    pub fn phrase_string(&self) -> Option<String> { self.link.phrase_string() }

    pub fn pump(&mut self, dt: f32, world: &mut dyn PlanetaryWorld) -> bool {
        self.link.pump(dt, world, &mut self.predicted)
    }
}

impl FactJournal for GuestJournal {
    fn inscribe_fact(&mut self, fact: WorldFact) {
        if fact.durable() {
            if self.predicted.len() >= 64 {
                let _ = self.predicted.pop_front();
            }
            self.predicted.push_back(fact);
        }
        self.link.send_petition(fact);
    }
}

/// Zero-knowledge P2P client decorator implementing `PlanetaryWorld` over `HollowVeins` and `ChronicledPlanet`.
pub struct RemotePlanet {
    world: Box<dyn PlanetaryWorld>,
    journal: GuestJournal,
}

impl RemotePlanet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_journal(world: Box<dyn PlanetaryWorld>, journal: GuestJournal) -> Self {
        Self { world, journal }
    }

    /// Secondary constructor assembling `RemotePlanet` from an inner world, link, and prediction queue.
    pub fn new(world: Box<dyn PlanetaryWorld>, link: RemoteLink, predicted: VecDeque<WorldFact>) -> Self {
        Self::with_journal(world, GuestJournal::new(link, predicted))
    }

    pub fn from_link(link: RemoteLink, blocks: u32, tron: bool, bevel_margin: f32) -> Self {
        let world: Box<dyn PlanetaryWorld> = if tron {
            Box::new(BiomedCubeWorld::hollow_beveled_tron(blocks, bevel_margin))
        } else {
            Box::new(BiomedCubeWorld::hollow_beveled(blocks, bevel_margin))
        };
        Self::new(world, link, VecDeque::new())
    }

    /// Connects to a `SanctuaryHost` via `PortalCode`, `PortalTicket`, or `host:port`.
    pub fn connect(
        target: JoinTarget,
        fallback_blocks: u32,
        fallback_tron: bool,
        bevel_margin: f32,
    ) -> std::io::Result<Self> {
        let mut link = RemoteLink::bind(target)?;
        let wait = match target {
            JoinTarget::Wormhole(_) => Duration::from_millis(650),
            JoinTarget::Ticket(_) | JoinTarget::Direct(_) => Duration::from_millis(450),
        };
        let (blocks, tron) = link.await_welcome(wait, fallback_blocks, fallback_tron);
        if !link.connected() || link.unresolved_wormhole() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Sanctuary handshake did not complete within welcome window",
            ));
        }
        Ok(Self::from_link(link, blocks, tron, bevel_margin))
    }

    /// Non-blocking connection constructor firing the initial knock immediately without sleeping.
    pub fn dial(
        target: JoinTarget,
        blocks_per_edge: u32,
        tron_mode: bool,
        bevel_margin: f32,
    ) -> std::io::Result<Self> {
        let link = RemoteLink::bind(target)?;
        Ok(Self::from_link(link, blocks_per_edge, tron_mode, bevel_margin))
    }

    pub fn connected(&self) -> bool {
        self.journal.connected()
    }

    fn with_chronicled<R>(&mut self, op: impl FnOnce(&mut ChronicledPlanet<'_>) -> R) -> R {
        let mut decorated = ChronicledPlanet::new(self.world.as_mut(), &mut self.journal);
        op(&mut decorated)
    }
}

impl GravityField for RemotePlanet {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        self.world.pull_direction_at(position)
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.world.surface_up_at(position)
    }
}

impl PlanetaryWorld for RemotePlanet {
    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.world.emit_surface(target);
    }

    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.world.constrain_to_surface(position, player_height)
    }

    fn canopy_drag(&self, position: Vec3) -> f32 {
        self.world.canopy_drag(position)
    }

    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.with_chronicled(|w| w.harvest_surface(eye, direction))
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        self.with_chronicled(|w| w.loosen_surface(eye, direction, stage))
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        self.with_chronicled(|w| w.strike_timber(eye, direction))
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        self.with_chronicled(|w| w.forage_surface(eye, direction))
    }

    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        self.with_chronicled(|w| w.cast_surface_find(eye, direction, find))
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        self.with_chronicled(|w| w.place_molded_surface_block(eye, direction, avatar_pos, mold))
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        self.with_chronicled(|w| w.adjust_rig(eye, direction, delta_steps, rotate_quarter))
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.with_chronicled(|w| w.lower_rig(eye, direction))
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        self.world.board_vehicle(eye, direction, holding_block)
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        self.world.park_vehicle(assembly)
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.world.placed_camera_tripod()
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let regrown = self.world.step_surface(dt);
        let net_changed = self.journal.pump(dt, self.world.as_mut());
        regrown || net_changed || self.world.pending_slices()
    }

    fn lethal_at(&self, position: Vec3) -> bool {
        self.world.lethal_at(position)
    }

    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 {
        self.world.confine_beneath_crust(position)
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

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        self.world.emit_meadow_around(center, view_dir, target);
    }

    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> {
        self.world.arboreal_perch_near(center, selector)
    }

    fn visible_slice_mask(&self, position: Vec3) -> u32 {
        self.world.visible_slice_mask(position)
    }

    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        self.world.disclose_stratum(face_index, stratum);
    }

    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) {
        self.world.drain_disclosures(out);
    }

    fn sync_presence(&mut self, presence: CompanionPresence) {
        self.journal.sync_local_presence(presence);
    }

    fn active_companions(&self, out: &mut Vec<CompanionPresence>) {
        self.journal.companions(out);
    }

    fn portal_ticket(&self) -> Option<String> {
        self.journal.ticket_string()
    }

    fn portal_phrase(&self) -> Option<String> {
        self.journal.phrase_string()
    }

    fn pump_portal(&mut self, dt: f32) -> bool {
        self.journal.pump(dt, self.world.as_mut()) || self.world.pending_slices()
    }
}
