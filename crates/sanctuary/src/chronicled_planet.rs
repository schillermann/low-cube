use glam::Vec3;
use world::{
    BlockMedium, BlockMold, CleaveOutcome, CompanionPresence, ExposedStratum, GravityField,
    MeshTarget, PlanetaryWorld, SurfaceFind, VehicleAssembly, WorldFact,
};

/// Focused contract absorbing locally enacted world mutations into a host chronicle or P2P petition stream.
pub trait FactJournal {
    fn inscribe_fact(&mut self, fact: WorldFact);
}

/// `PlanetaryWorld` decorator recording successful local surface mutations into a `FactJournal`.
pub struct ChronicledPlanet<'a> {
    origin: &'a mut dyn PlanetaryWorld,
    journal: &'a mut dyn FactJournal,
}

impl<'a> ChronicledPlanet<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: &'a mut dyn PlanetaryWorld, journal: &'a mut dyn FactJournal) -> Self {
        Self { origin, journal }
    }
}

impl GravityField for ChronicledPlanet<'_> {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        self.origin.pull_direction_at(position)
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.origin.surface_up_at(position)
    }
}

impl PlanetaryWorld for ChronicledPlanet<'_> {
    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_surface(target);
    }

    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.origin.constrain_to_surface(position, player_height)
    }

    fn canopy_drag(&self, position: Vec3) -> f32 {
        self.origin.canopy_drag(position)
    }

    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        let medium = self.origin.harvest_surface(eye, direction)?;
        self.journal.inscribe_fact(WorldFact::Harvested { eye, direction });
        Some(medium)
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        let medium = self.origin.loosen_surface(eye, direction, stage)?;
        self.journal.inscribe_fact(WorldFact::Loosened { eye, direction, stage });
        Some(medium)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        let out = self.origin.strike_timber(eye, direction);
        if out.struck() {
            self.journal.inscribe_fact(WorldFact::TimberStruck { eye, direction });
        }
        out
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        let find = self.origin.forage_surface(eye, direction);
        if find.gathered() {
            self.journal.inscribe_fact(WorldFact::Foraged { eye, direction });
        }
        find
    }

    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        let ok = self.origin.cast_surface_find(eye, direction, find);
        if ok {
            self.journal.inscribe_fact(WorldFact::LitterCast { eye, direction, find });
        }
        ok
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        let ok = self.origin.place_molded_surface_block(eye, direction, avatar_pos, mold);
        if ok {
            self.journal.inscribe_fact(WorldFact::MoldPlaced { eye, direction, avatar_pos, mold });
        }
        ok
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        let ok = self.origin.adjust_rig(eye, direction, delta_steps, rotate_quarter);
        if ok {
            self.journal.inscribe_fact(WorldFact::RigAdjusted { eye, direction, delta_steps, rotate_quarter });
        }
        ok
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        let ok = self.origin.lower_rig(eye, direction);
        if ok {
            self.journal.inscribe_fact(WorldFact::RigLowered { eye, direction });
        }
        ok
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        self.origin.board_vehicle(eye, direction, holding_block)
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        self.origin.park_vehicle(assembly)
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.origin.placed_camera_tripod()
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        self.origin.step_surface(dt)
    }

    fn lethal_at(&self, position: Vec3) -> bool {
        self.origin.lethal_at(position)
    }

    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 {
        self.origin.confine_beneath_crust(position)
    }

    fn face_count(&self) -> usize {
        self.origin.face_count()
    }

    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) {
        self.origin.emit_face(index, target);
    }

    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        self.origin.emit_modified_face(target)
    }

    fn pending_slices(&self) -> bool {
        self.origin.pending_slices()
    }

    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> {
        self.origin.drain_modified_slice(target)
    }

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        self.origin.emit_meadow_around(center, view_dir, target);
    }

    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> {
        self.origin.arboreal_perch_near(center, selector)
    }

    fn visible_slice_mask(&self, position: Vec3) -> u32 {
        self.origin.visible_slice_mask(position)
    }

    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        self.origin.disclose_stratum(face_index, stratum);
    }

    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) {
        self.origin.drain_disclosures(out);
    }

    fn sync_presence(&mut self, presence: CompanionPresence) {
        self.origin.sync_presence(presence);
    }

    fn active_companions(&self, out: &mut Vec<CompanionPresence>) {
        self.origin.active_companions(out);
    }
}
