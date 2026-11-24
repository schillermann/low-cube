mod facets;

use std::collections::VecDeque;
use std::time::Duration;
use glam::Vec3;
use world::{
    BlockMedium, BlockMold, CelestialSystem, CleaveOutcome, CraftBlueprint, CraftIdentity,
    CraftPose, ExposedStratum, GravityField, MiningEdge, PlanetaryWorld, SurfaceFind,
    VehicleAssembly, WorldEarthworks, WorldFact,
};
use crate::chronicled_planet::{ChronicledPlanet, FactJournal as _};
use crate::guest_journal::GuestJournal;
use crate::remote_link::RemoteLink;
use crate::ticket::JoinTarget;

/// Zero-knowledge P2P client decorator owning `PlanetaryWorld` lock-free alongside `GuestJournal`.
pub struct RemotePlanet {
    pub(super) world: Box<dyn PlanetaryWorld>,
    pub(super) journal: GuestJournal,
}

impl RemotePlanet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_journal(world: Box<dyn PlanetaryWorld>, journal: GuestJournal) -> Self {
        Self { world, journal }
    }

    pub fn new(mut world: Box<dyn PlanetaryWorld>, mut link: RemoteLink, predicted: VecDeque<WorldFact>) -> Self {
        if let Some(ep) = link.take_pending_epoch() {
            world.align_epoch(ep);
        }
        if let Some((frame, pos)) = link.take_pending_anchor() {
            world.anchor_to_frame(frame, pos);
        }
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
        Self::connect_cosmos_with_vault(
            target,
            Box::new(crate::passport::FixedPassport::unassigned()),
            fallback,
            bevel_margin,
        )
    }

    pub fn connect_cosmos_with_identity(
        target: JoinTarget,
        keys: crate::seal::HandshakeIdentity,
        fallback: world::SolarProportions,
        bevel_margin: f32,
    ) -> std::io::Result<Self> {
        Self::connect_cosmos_with_vault(
            target,
            Box::new(crate::passport::FixedPassport::pinned(&keys)),
            fallback,
            bevel_margin,
        )
    }

    pub fn connect_cosmos_with_vault(
        target: JoinTarget,
        vault: Box<dyn crate::passport::PassportVault>,
        fallback: world::SolarProportions,
        bevel_margin: f32,
    ) -> std::io::Result<Self> {
        let mut link = RemoteLink::bind_with_vault(target, vault)?;
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

    pub fn connected(&self) -> bool {
        self.journal.connected()
    }

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
    fn surface_yields_to(&self, eye: Vec3, direction: Vec3, edge: MiningEdge) -> bool { self.world.surface_yields_to(eye, direction, edge) }
    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome { self.with_chronicled(|w| w.strike_timber(eye, direction)) }
    fn strike_boar(&mut self, eye: Vec3, direction: Vec3, power: u8, max_reach: f32) -> (bool, bool) {
        if !self.journal.intercept_boar_strike(eye, direction, max_reach, self.world.as_ref()) {
            return (false, false);
        }
        let reach_cm = (max_reach.clamp(0.0, 10.0) * 100.0).round() as u16;
        self.journal.inscribe_fact(WorldFact::BoarStruck { eye, direction, power, reach_cm });
        (true, false)
    }
    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind { self.with_chronicled(|w| w.forage_surface(eye, direction)) }
    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool { self.with_chronicled(|w| w.cast_surface_find(eye, direction, find)) }
    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        self.with_chronicled(|w| w.place_molded_surface_block(eye, direction, avatar_pos, mold))
    }
    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        self.with_chronicled(|w| w.adjust_rig(eye, direction, delta_steps, rotate_quarter))
    }
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
        self.with_chronicled(|w| w.enact_offworld(body, fact))
    }
}
