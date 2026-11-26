use glam::Vec3;
use world::{
    BlockMedium, BlockMold, CleaveOutcome, CraftBlueprint, CraftIdentity, CraftPose,
    ExposedStratum, MiningEdge, SurfaceFind, VehicleAssembly, WorldEarthworks, WorldFact,
};
use crate::chronicled_planet::ChronicledPlanet;
use super::SavedPlanet;

impl WorldEarthworks for SavedPlanet {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.with_chronicled(|w| w.harvest_surface(eye, direction))
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        self.with_chronicled(|w| w.loosen_surface(eye, direction, stage))
    }

    fn surface_yields_to(&self, eye: Vec3, direction: Vec3, edge: MiningEdge) -> bool {
        self.world.surface_yields_to(eye, direction, edge)
    }

    fn aimed_surface_medium(&self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.world.aimed_surface_medium(eye, direction)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        self.with_chronicled(|w| w.strike_timber(eye, direction))
    }

    fn strike_boar(&mut self, eye: Vec3, direction: Vec3, power: u8, max_reach: f32) -> (bool, bool) {
        self.with_chronicled(|w| w.strike_boar(eye, direction, power, max_reach))
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

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        self.with_chronicled(|w| w.park_vehicle(assembly))
    }

    fn park_craft_by_id(&mut self, craft: CraftIdentity, pose: CraftPose) -> bool {
        let mut decorated = ChronicledPlanet::new(self.world.as_mut(), &mut self.ledger);
        let ok = decorated.park_craft_by_id(craft, pose);
        self.ledger.flush_tick();
        ok
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

    fn harvest_offworld(&mut self, body: world::BodyId, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.with_chronicled(|w| w.harvest_offworld(body, eye, direction))
    }

    fn enact_offworld(&mut self, body: world::BodyId, fact: &WorldFact) -> bool {
        self.with_chronicled(|w| w.enact_offworld(body, fact))
    }

    fn capture_sector_snapshot(&self, face: u8, gsu: i32, gsv: i32) -> world::SectorSnapshot {
        self.world.capture_sector_snapshot(face, gsu, gsv)
    }

    fn apply_sector_snapshot(&mut self, snapshot: &world::SectorSnapshot) -> bool {
        self.with_chronicled(|w| w.apply_sector_snapshot(snapshot))
    }

    fn collect_sector_snapshots(&self, out: &mut Vec<world::SectorSnapshot>) {
        self.world.collect_sector_snapshots(out);
    }

    fn capture_offworld_snapshot(&mut self, body: world::BodyId, face: u8, gsu: i32, gsv: i32) -> world::SectorSnapshot {
        self.world.capture_offworld_snapshot(body, face, gsu, gsv)
    }

    fn collect_offworld_snapshots(&mut self, out: &mut Vec<(world::BodyId, world::SectorSnapshot)>) {
        self.world.collect_offworld_snapshots(out);
    }
}
