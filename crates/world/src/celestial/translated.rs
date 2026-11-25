use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CleaveOutcome, CompanionPortal, CraftBlueprint, CraftIdentity,
    CraftPose, ExposedStratum, GravityField, MeadowRing, MeshTarget, MiningEdge, PlanetaryWorld,
    PlannedMount, Quad, SurfaceFind, TerrainMesh, VehicleAssembly, VesselDeck, VesselSink,
    WoodlandLife, WorldEarthworks,
};

struct TranslatedVesselSink<'a> {
    shift: Vec3,
    inner: &'a mut dyn VesselSink,
}

impl VesselSink for TranslatedVesselSink<'_> {
    fn receive_vessel(&mut self, blueprint: &CraftBlueprint, pose: CraftPose) {
        self.inner.receive_vessel(blueprint, pose.with_origin(pose.origin() + self.shift));
    }
}

struct TranslatedTarget<'a> {
    shift: Vec3,
    inner: &'a mut dyn MeshTarget,
}

impl MeshTarget for TranslatedTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let (s, glow) = (self.shift, quad.radiate(|g| g));
        self.inner.push_quad(quad.transfer(|v0, v1, v2, v3, n, col| {
            Quad::glowing(v0 + s, v1 + s, v2 + s, v3 + s, n, col, glow)
        }));
    }
}

/// Spatial coordinate translation decorator positioning a `PlanetaryWorld` (such as the Moon) at `center`.
pub struct TranslatedWorld<W> {
    origin: W,
    center: Vec3,
}

impl<W> TranslatedWorld<W> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: W, center: Vec3) -> Self {
        Self { origin, center }
    }

    pub const fn inner(&self) -> &W {
        &self.origin
    }

    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.origin
    }

    pub fn into_inner(self) -> W {
        self.origin
    }

    pub fn relocate_center(&mut self, next_center: Vec3) {
        self.center = next_center;
    }

    pub fn in_vicinity(&self, position: Vec3, cheb_bound: f32) -> bool {
        (position - self.center).abs().max_element() <= cheb_bound
    }

    pub const fn compose_atlas(&self, home_half: f32, moon_half: f32) -> crate::CelestialAtlas {
        crate::CelestialAtlas::new(home_half, self.center, moon_half)
    }

    pub fn visible_faces_from(&self, position: Vec3, half_size: f32) -> u32 {
        let rel = position - self.center;
        let mut mask = 0u32;
        for (idx, zone) in crate::ClimateZone::all().into_iter().enumerate() {
            let n = zone.normal();
            if rel.dot(n) > -half_size * 0.45 {
                mask |= 1u32 << idx;
            }
        }
        mask
    }
}

impl<W: GravityField> GravityField for TranslatedWorld<W> {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 { self.origin.pull_direction_at(position - self.center) }
    fn surface_up_at(&self, position: Vec3) -> Vec3 { self.origin.surface_up_at(position - self.center) }
    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 { self.origin.apply_pull(position - self.center, dt) }
}

impl<W: WorldEarthworks> WorldEarthworks for TranslatedWorld<W> {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> { self.origin.harvest_surface(eye - self.center, direction) }
    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> { self.origin.loosen_surface(eye - self.center, direction, stage) }
    fn surface_yields_to(&self, eye: Vec3, direction: Vec3, edge: MiningEdge) -> bool { self.origin.surface_yields_to(eye - self.center, direction, edge) }
    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome { self.origin.strike_timber(eye - self.center, direction) }
    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind { self.origin.forage_surface(eye - self.center, direction) }
    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool { self.origin.cast_surface_find(eye - self.center, direction, find) }
    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        self.origin.place_molded_surface_block(eye - self.center, direction, avatar_pos - self.center, mold)
    }
    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        self.origin.adjust_rig(eye - self.center, direction, delta_steps, rotate_quarter)
    }
    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool { self.origin.lower_rig(eye - self.center, direction) }
    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        self.origin.build_rig_blueprint(eye - self.center, direction, blueprint)
    }
    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        let mut assembly = self.origin.board_vehicle(eye - self.center, direction, holding_block)?;
        assembly.translate_to(assembly.origin() + self.center);
        Some(assembly)
    }
    fn park_vehicle(&mut self, mut assembly: VehicleAssembly) -> bool {
        assembly.translate_to(assembly.origin() - self.center);
        self.origin.park_vehicle(assembly)
    }
    fn register_craft_blueprint(&mut self, blueprint: CraftBlueprint) -> bool { self.origin.register_craft_blueprint(blueprint) }
    fn claim_vehicle_helm(&mut self, craft: CraftIdentity, pilot_slot: u32, eye: Vec3, direction: Vec3) -> bool {
        self.origin.claim_vehicle_helm(craft, pilot_slot, eye - self.center, direction)
    }
    fn park_craft_by_id(&mut self, craft: CraftIdentity, pose: CraftPose) -> bool {
        self.origin.park_craft_by_id(craft, pose.with_origin(pose.origin() - self.center))
    }
    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) { self.origin.disclose_stratum(face_index, stratum); }
    fn capture_sector_snapshot(&self, face: u8, gsu: i32, gsv: i32) -> crate::SectorSnapshot {
        self.origin.capture_sector_snapshot(face, gsu, gsv)
    }
    fn apply_sector_snapshot(&mut self, snapshot: &crate::SectorSnapshot) -> bool {
        self.origin.apply_sector_snapshot(snapshot)
    }
    fn collect_sector_snapshots(&self, out: &mut Vec<crate::SectorSnapshot>) {
        self.origin.collect_sector_snapshots(out);
    }
}

impl<W: TerrainMesh> TerrainMesh for TranslatedWorld<W> {
    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        let mut shifted = TranslatedTarget { shift: self.center, inner: target };
        self.origin.emit_surface(&mut shifted);
    }
    fn face_count(&self) -> usize { self.origin.face_count() }
    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) {
        let mut shifted = TranslatedTarget { shift: self.center, inner: target };
        self.origin.emit_face(index, &mut shifted);
    }
    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        let mut shifted = TranslatedTarget { shift: self.center, inner: target };
        self.origin.emit_modified_face(&mut shifted)
    }
    fn pending_slices(&self) -> bool { self.origin.pending_slices() }
    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> {
        let mut shifted = TranslatedTarget { shift: self.center, inner: target };
        self.origin.drain_modified_slice(&mut shifted)
    }
    fn face_patch_count(&self, slice: usize) -> usize { self.origin.face_patch_count(slice) }
    fn emit_face_patch(&self, slice: usize, patch_slot: usize, target: &mut dyn MeshTarget) {
        let mut shifted = TranslatedTarget { shift: self.center, inner: target };
        self.origin.emit_face_patch(slice, patch_slot, &mut shifted);
    }
    fn drain_modified_patch(&mut self, target: &mut dyn MeshTarget) -> Option<(usize, usize)> {
        let mut shifted = TranslatedTarget { shift: self.center, inner: target };
        self.origin.drain_modified_patch(&mut shifted)
    }
    fn drain_stale_meadow_patch(&self, center: Vec3, ring: &mut MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        let mut shifted = TranslatedTarget { shift: self.center, inner: target };
        self.origin.drain_stale_meadow_patch(center - self.center, ring, &mut shifted)
    }
    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        let mut shifted = TranslatedTarget { shift: self.center, inner: target };
        self.origin.emit_meadow_around(center - self.center, view_dir, &mut shifted);
    }
    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) { self.origin.drain_disclosures(out); }
}

impl<W> WoodlandLife for TranslatedWorld<W> {
    fn woodland_habitat(&self, _position: Vec3) -> bool { false }
}

impl<W: VesselDeck> VesselDeck for TranslatedWorld<W> {
    fn present_vessels(&self, sink: &mut dyn VesselSink) {
        let mut shifted = TranslatedVesselSink { shift: self.center, inner: sink };
        self.origin.present_vessels(&mut shifted);
    }
    fn present_vessels_near(&self, center: Vec3, radius: f32, sink: &mut dyn VesselSink) {
        let mut shifted = TranslatedVesselSink { shift: self.center, inner: sink };
        self.origin.present_vessels_near(center - self.center, radius, &mut shifted);
    }
    fn plan_molded_block(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<PlannedMount> {
        self.origin
            .plan_molded_block(eye - self.center, direction, avatar_pos - self.center, mold)
            .map(|p| p.translated(self.center))
    }
    fn capture_vehicle_blueprint(&self, eye: Vec3, direction: Vec3) -> Option<CraftBlueprint> {
        self.origin.capture_vehicle_blueprint(eye - self.center, direction)
    }
    fn aimed_empty_rig(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.origin.aimed_empty_rig(eye - self.center, direction, avatar_pos - self.center)
    }
}

impl<W> CompanionPortal for TranslatedWorld<W> {}

impl<W: PlanetaryWorld> PlanetaryWorld for TranslatedWorld<W> {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.origin.constrain_to_surface(position - self.center, player_height) + self.center
    }
    fn surface_contact(&self, probe: Vec3, clearance: f32) -> crate::SurfaceContact {
        self.origin.surface_contact(probe - self.center, clearance).translated(self.center)
    }
    fn canopy_drag(&self, position: Vec3) -> f32 { self.origin.canopy_drag(position - self.center) }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        let (hub, fwd, up) = self.origin.placed_camera_tripod()?;
        Some((hub + self.center, fwd, up))
    }
    fn collect_medium_sites(&self, medium: crate::BlockMedium, out: &mut Vec<(Vec3, Vec3)>) {
        let start = out.len();
        self.origin.collect_medium_sites(medium, out);
        for (p, _) in &mut out[start..] {
            *p += self.center;
        }
    }
    fn step_surface(&mut self, dt: f32) -> bool { self.origin.step_surface(dt) }
    fn step_active_zones(&mut self, dt: f32, earth_zone_mask: u8, active_bodies: &[crate::BodyId]) -> bool {
        self.origin.step_active_zones(dt, earth_zone_mask, active_bodies)
    }
    fn lethal_at(&self, position: Vec3) -> bool { self.origin.lethal_at(position - self.center) }
    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 {
        self.origin.confine_beneath_crust(position - self.center) + self.center
    }
    fn local_body_point(&self, position: Vec3) -> Vec3 { position - self.center }
}
