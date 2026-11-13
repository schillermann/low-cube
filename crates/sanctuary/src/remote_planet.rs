use std::collections::VecDeque;
use std::time::Duration;
use glam::Vec3;
use world::{
    BiomedCubeWorld, BlockMedium, BlockMold, CleaveOutcome, CompanionPresence, ExposedStratum,
    GravityField, MeshTarget, PlanetaryWorld, SurfaceFind, VehicleAssembly, WorldFact,
};
use crate::chronicle_log::DirtySlices;
use crate::remote_link::RemoteLink;
use crate::ticket::JoinTarget;

/// Zero-knowledge multiplayer client decorator implementing `PlanetaryWorld` over `HollowVeins`.
pub struct RemotePlanet {
    world: Box<dyn PlanetaryWorld>,
    link: RemoteLink,
    predicted: VecDeque<WorldFact>,
    slices: DirtySlices,
}

impl RemotePlanet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        world: Box<dyn PlanetaryWorld>,
        link: RemoteLink,
        predicted: VecDeque<WorldFact>,
        slices: DirtySlices,
    ) -> Self {
        Self { world, link, predicted, slices }
    }

    /// Connects to a `SanctuaryHost` via `PortalTicket` or `host:port` and builds a zero-knowledge cube.
    pub fn connect(
        target: JoinTarget,
        fallback_blocks: u32,
        fallback_tron: bool,
        bevel_margin: f32,
    ) -> std::io::Result<Self> {
        let mut link = RemoteLink::bind(target)?;
        let (blocks, tron) = link.await_welcome(Duration::from_millis(450), fallback_blocks, fallback_tron);
        let world: Box<dyn PlanetaryWorld> = if tron {
            Box::new(BiomedCubeWorld::hollow_beveled_tron(blocks, bevel_margin))
        } else {
            Box::new(BiomedCubeWorld::hollow_beveled(blocks, bevel_margin))
        };
        Ok(Self::new(world, link, VecDeque::new(), DirtySlices::clean()))
    }

    /// Non-blocking connection constructor firing the initial knock immediately without sleeping.
    pub fn dial(
        target: JoinTarget,
        blocks_per_edge: u32,
        tron_mode: bool,
        bevel_margin: f32,
    ) -> std::io::Result<Self> {
        let link = RemoteLink::bind(target)?;
        let world: Box<dyn PlanetaryWorld> = if tron_mode {
            Box::new(BiomedCubeWorld::hollow_beveled_tron(blocks_per_edge, bevel_margin))
        } else {
            Box::new(BiomedCubeWorld::hollow_beveled(blocks_per_edge, bevel_margin))
        };
        Ok(Self::new(world, link, VecDeque::new(), DirtySlices::clean()))
    }

    pub fn connected(&self) -> bool {
        self.link.connected()
    }

    fn record_prediction(&mut self, fact: WorldFact) {
        if self.predicted.len() >= 64 {
            let _ = self.predicted.pop_front();
        }
        self.predicted.push_back(fact);
        self.link.send_petition(fact);
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
        let medium = self.world.harvest_surface(eye, direction)?;
        self.record_prediction(WorldFact::Harvested { eye, direction });
        Some(medium)
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        let medium = self.world.loosen_surface(eye, direction, stage)?;
        self.link.send_petition(WorldFact::Loosened { eye, direction, stage });
        Some(medium)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        let out = self.world.strike_timber(eye, direction);
        if out.struck() {
            self.record_prediction(WorldFact::TimberStruck { eye, direction });
        }
        out
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        let find = self.world.forage_surface(eye, direction);
        if find.gathered() {
            self.record_prediction(WorldFact::Foraged { eye, direction });
        }
        find
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        let ok = self.world.place_molded_surface_block(eye, direction, avatar_pos, mold);
        if ok {
            self.record_prediction(WorldFact::MoldPlaced { eye, direction, avatar_pos, mold });
        }
        ok
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        let ok = self.world.adjust_rig(eye, direction, delta_steps, rotate_quarter);
        if ok {
            self.record_prediction(WorldFact::RigAdjusted { eye, direction, delta_steps, rotate_quarter });
        }
        ok
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        let ok = self.world.lower_rig(eye, direction);
        if ok {
            self.record_prediction(WorldFact::RigLowered { eye, direction });
        }
        ok
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
        let net_changed = self.link.pump(dt, self.world.as_mut(), &mut self.predicted, &self.slices);
        regrown || net_changed || self.slices.pending()
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
        if let Some(idx) = self.slices.pop_next() {
            self.world.emit_face(idx, target);
            idx
        } else {
            self.world.emit_modified_face(target)
        }
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
        self.link.sync_local_presence(presence);
    }

    fn active_companions(&self, out: &mut Vec<CompanionPresence>) {
        self.link.companions(out);
    }

    fn portal_ticket(&self) -> Option<String> {
        self.link.ticket_string()
    }

    fn pump_portal(&mut self, dt: f32) -> bool {
        self.link.pump(dt, self.world.as_mut(), &mut self.predicted, &self.slices) || self.slices.pending()
    }
}
