use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CleaveOutcome, ClimateZone, CompanionPresence, CraftBlueprint,
    CraftIdentity, CraftPose, DrivenCraft, ExposedStratum, GravityField, MeadowRing, MeshTarget,
    PerimeterEdge, PilotCraft, PlanetaryTopology, PlanetaryWorld, SeamBevel, SurfaceFind,
    VehicleAssembly,
};

/// Decorator enabling smooth, continuous surface normal transitions in bevel zones across planetary seams.
pub struct BeveledWorld {
    world: Box<dyn PlanetaryWorld>,
    topology: PlanetaryTopology,
    half_size: f32,
    margin: f32,
}

impl BeveledWorld {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        world: Box<dyn PlanetaryWorld>,
        topology: PlanetaryTopology,
        half_size: f32,
        margin: f32,
    ) -> Self {
        Self {
            world,
            topology,
            half_size,
            margin,
        }
    }

    /// Secondary constructor deriving cubic topology for a world of specified half size.
    pub fn decorating(world: Box<dyn PlanetaryWorld>, half_size: f32, margin: f32) -> Self {
        let topology = PlanetaryTopology::cubic(half_size);
        Self::new(world, topology, half_size, margin)
    }

    /// Dispatches continuous bevel behavior if within a seam transition zone ("Tell, Don't Ask").
    fn with_bevel<T>(
        &self,
        position: Vec3,
        on_bevel: impl FnOnce(SeamBevel) -> T,
        on_origin: impl FnOnce() -> T,
    ) -> T {
        let dominant = ClimateZone::dominant(position);
        for edge in PerimeterEdge::all() {
            let bevel = self.topology.bevel(dominant, edge, self.half_size, self.margin);
            if bevel.contains(position) {
                return on_bevel(bevel);
            }
        }
        on_origin()
    }
}

impl GravityField for BeveledWorld {
    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.with_bevel(
            position,
            |bevel| bevel.normal_at(position),
            || self.world.surface_up_at(position),
        )
    }

    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        -self.surface_up_at(position)
    }
}

impl PlanetaryWorld for BeveledWorld {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.world.constrain_to_surface(position, player_height)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.world.emit_surface(target);
    }

    fn canopy_drag(&self, position: Vec3) -> f32 {
        self.world.canopy_drag(position)
    }

    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.world.harvest_surface(eye, direction)
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        self.world.loosen_surface(eye, direction, stage)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        self.world.strike_timber(eye, direction)
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        self.world.forage_surface(eye, direction)
    }

    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        self.world.cast_surface_find(eye, direction, find)
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        self.world.place_molded_surface_block(eye, direction, avatar_pos, mold)
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        self.world.adjust_rig(eye, direction, delta_steps, rotate_quarter)
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.world.lower_rig(eye, direction)
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        self.world.board_vehicle(eye, direction, holding_block)
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        self.world.park_vehicle(assembly)
    }

    fn register_craft_blueprint(&mut self, blueprint: CraftBlueprint) -> bool {
        self.world.register_craft_blueprint(blueprint)
    }

    fn claim_vehicle_helm(&mut self, craft: CraftIdentity, pilot_slot: u32, eye: Vec3, direction: Vec3) -> bool {
        self.world.claim_vehicle_helm(craft, pilot_slot, eye, direction)
    }

    fn park_craft_by_id(&mut self, craft: CraftIdentity, pose: CraftPose) -> bool {
        self.world.park_craft_by_id(craft, pose)
    }

    fn board_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        self.world.board_passenger_seat(eye, direction)
    }

    fn passenger_seat_frame(&self, craft: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> {
        self.world.passenger_seat_frame(craft, local_seat)
    }

    fn helm_revoked(&mut self, craft: CraftIdentity) -> bool {
        self.world.helm_revoked(craft)
    }

    fn emit_active_crafts(&self, target: &mut dyn MeshTarget) {
        self.world.emit_active_crafts(target);
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

    fn drain_stale_meadow_patch(&self, center: Vec3, ring: &mut MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        self.world.drain_stale_meadow_patch(center, ring, target)
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.world.placed_camera_tripod()
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        self.world.step_surface(dt)
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
        self.world.sync_presence(presence);
    }

    fn sync_pilot_craft(&mut self, craft: PilotCraft) {
        self.world.sync_pilot_craft(craft);
    }

    fn active_companions(&self, out: &mut Vec<CompanionPresence>) {
        self.world.active_companions(out);
    }

    fn active_crafts(&self, out: &mut Vec<DrivenCraft>) {
        self.world.active_crafts(out);
    }

    fn blend_companions(&mut self, subtick_secs: f32) {
        self.world.blend_companions(subtick_secs);
    }

    fn portal_ticket(&self) -> Option<String> {
        self.world.portal_ticket()
    }

    fn pump_portal(&mut self, dt: f32) -> bool {
        self.world.pump_portal(dt)
    }
}
