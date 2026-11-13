use std::net::SocketAddr;
use glam::Vec3;
use world::{
    BlockMedium, BlockMold, CleaveOutcome, CompanionPresence, ExposedStratum, GravityField,
    MeshTarget, PlanetaryWorld, SurfaceFind, VehicleAssembly, WorldFact,
};
use crate::sanctuary_host::SanctuaryHost;

/// Decorator turning any authoritative `PlanetaryWorld` into a live multiplayer host without duplicate logic.
pub struct HostedPlanet {
    host: SanctuaryHost,
}

impl HostedPlanet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(host: SanctuaryHost) -> Self {
        Self { host }
    }
}

impl GravityField for HostedPlanet {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        self.host.world().pull_direction_at(position)
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.host.world().surface_up_at(position)
    }
}

impl PlanetaryWorld for HostedPlanet {
    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.host.world().emit_surface(target);
    }

    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.host.world().constrain_to_surface(position, player_height)
    }

    fn canopy_drag(&self, position: Vec3) -> f32 {
        self.host.world().canopy_drag(position)
    }

    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        let medium = self.host.world_mut().harvest_surface(eye, direction)?;
        self.host.record_local_fact(WorldFact::Harvested { eye, direction });
        Some(medium)
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        let medium = self.host.world_mut().loosen_surface(eye, direction, stage)?;
        self.host.record_local_fact(WorldFact::Loosened { eye, direction, stage });
        Some(medium)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        let out = self.host.world_mut().strike_timber(eye, direction);
        if out.struck() {
            self.host.record_local_fact(WorldFact::TimberStruck { eye, direction });
        }
        out
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        let find = self.host.world_mut().forage_surface(eye, direction);
        if find.gathered() {
            self.host.record_local_fact(WorldFact::Foraged { eye, direction });
        }
        find
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        let ok = self.host.world_mut().place_molded_surface_block(eye, direction, avatar_pos, mold);
        if ok {
            self.host.record_local_fact(WorldFact::MoldPlaced { eye, direction, avatar_pos, mold });
        }
        ok
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        let ok = self.host.world_mut().adjust_rig(eye, direction, delta_steps, rotate_quarter);
        if ok {
            self.host.record_local_fact(WorldFact::RigAdjusted { eye, direction, delta_steps, rotate_quarter });
        }
        ok
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        let ok = self.host.world_mut().lower_rig(eye, direction);
        if ok {
            self.host.record_local_fact(WorldFact::RigLowered { eye, direction });
        }
        ok
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        self.host.world_mut().board_vehicle(eye, direction, holding_block)
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        self.host.world_mut().park_vehicle(assembly)
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.host.world().placed_camera_tripod()
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let regrown = self.host.world_mut().step_surface(dt);
        let net_changed = self.host.pump_network(dt);
        regrown || net_changed || self.host.slices().pending()
    }

    fn lethal_at(&self, position: Vec3) -> bool {
        self.host.world().lethal_at(position)
    }

    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 {
        self.host.world().confine_beneath_crust(position)
    }

    fn face_count(&self) -> usize {
        self.host.world().face_count()
    }

    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) {
        self.host.world().emit_face(index, target);
    }

    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        if let Some(idx) = self.host.slices().pop_next() {
            self.host.world().emit_face(idx, target);
            idx
        } else {
            self.host.world().emit_modified_face(target)
        }
    }

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        self.host.world().emit_meadow_around(center, view_dir, target);
    }

    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> {
        self.host.world().arboreal_perch_near(center, selector)
    }

    fn visible_slice_mask(&self, position: Vec3) -> u32 {
        self.host.world().visible_slice_mask(position)
    }

    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        self.host.world_mut().disclose_stratum(face_index, stratum);
    }

    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) {
        self.host.world_mut().drain_disclosures(out);
    }

    fn sync_presence(&mut self, presence: CompanionPresence) {
        self.host.sync_host_presence(presence);
    }

    fn active_companions(&self, out: &mut Vec<CompanionPresence>) {
        self.host.guest_companions(out);
    }

    fn portal_ticket(&self) -> Option<String> {
        Some(self.host.ticket().encode())
    }

    fn portal_phrase(&self) -> Option<String> {
        Some(self.host.code().encode())
    }

    fn punch_nat_peer(&mut self, peer: SocketAddr) -> bool {
        self.host.punch_nat_peer(peer);
        true
    }

    fn pump_portal(&mut self, dt: f32) -> bool {
        self.host.pump_network(dt) || self.host.slices().pending()
    }
}
