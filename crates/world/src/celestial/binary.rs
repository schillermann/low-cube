use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CleaveOutcome, ClimateZone, CubePlanet, ExposedStratum, GravityField,
    MeadowRing, MeshTarget, PlanetaryWorld, SurfaceFind, VehicleAssembly, WorldEarthworks,
};
use super::lunar::LunarWorld;
use super::translated::TranslatedWorld;

/// Two-body planetary world uniting the 6-face Homeworld (`0..6`), 6-face Moon (`6..12`), and `BinaryGravity`.
pub struct BinaryPlanet<G> {
    home: CubePlanet,
    moon: TranslatedWorld<CubePlanet>,
    gravity: G,
    home_half: f32,
}

impl<G> BinaryPlanet<G> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        home: CubePlanet,
        moon: TranslatedWorld<CubePlanet>,
        gravity: G,
        home_half: f32,
    ) -> Self {
        Self { home, moon, gravity, home_half }
    }

    fn near_moon(&self, pos: Vec3) -> bool {
        self.moon.in_vicinity(pos, LunarWorld::HALF_SIZE + 36.0)
    }

    fn map_home_slice(idx: usize) -> usize {
        if idx < 6 { idx } else { idx + 6 }
    }

    fn map_moon_slice(idx: usize) -> usize {
        if idx < 6 { idx + 6 } else { idx + 12 }
    }
}

impl<G: GravityField> GravityField for BinaryPlanet<G> {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        self.gravity.pull_direction_at(position)
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.gravity.surface_up_at(position)
    }

    fn apply_pull(&self, position: Vec3, dt: f32) -> Vec3 {
        self.gravity.apply_pull(position, dt)
    }
}

impl<G: GravityField + Send> WorldEarthworks for BinaryPlanet<G> {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        if self.near_moon(eye) {
            self.moon.harvest_surface(eye, direction).or_else(|| self.home.harvest_surface(eye, direction))
        } else {
            self.home.harvest_surface(eye, direction).or_else(|| self.moon.harvest_surface(eye, direction))
        }
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        if self.near_moon(eye) {
            self.moon.loosen_surface(eye, direction, stage).or_else(|| self.home.loosen_surface(eye, direction, stage))
        } else {
            self.home.loosen_surface(eye, direction, stage).or_else(|| self.moon.loosen_surface(eye, direction, stage))
        }
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        if self.near_moon(eye) { CleaveOutcome::Missed } else { self.home.strike_timber(eye, direction) }
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        if self.near_moon(eye) { SurfaceFind::EmptyHanded } else { self.home.forage_surface(eye, direction) }
    }

    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        if self.near_moon(eye) {
            self.moon.cast_surface_find(eye, direction, find)
        } else {
            self.home.cast_surface_find(eye, direction, find)
        }
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        if self.near_moon(eye) {
            self.moon.place_molded_surface_block(eye, direction, avatar_pos, mold)
        } else {
            self.home.place_molded_surface_block(eye, direction, avatar_pos, mold)
        }
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        if self.near_moon(eye) {
            self.moon.adjust_rig(eye, direction, delta_steps, rotate_quarter)
        } else {
            self.home.adjust_rig(eye, direction, delta_steps, rotate_quarter)
        }
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        if self.near_moon(eye) {
            self.moon.lower_rig(eye, direction)
        } else {
            self.home.lower_rig(eye, direction)
        }
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        if self.near_moon(eye) {
            self.moon.board_vehicle(eye, direction, holding_block)
        } else {
            self.home.board_vehicle(eye, direction, holding_block)
        }
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        if self.near_moon(assembly.origin()) {
            self.moon.park_vehicle(assembly)
        } else {
            self.home.park_vehicle(assembly)
        }
    }

    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        if face_index < 6 {
            self.home.disclose_stratum(face_index, stratum);
        } else {
            self.moon.disclose_stratum(face_index - 6, stratum);
        }
    }
}

impl<G: GravityField + Send> PlanetaryWorld for BinaryPlanet<G> {
    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.home.emit_surface(target);
        self.moon.emit_surface(target);
    }

    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        if self.near_moon(position) {
            self.moon.constrain_to_surface(position, player_height)
        } else {
            self.home.constrain_to_surface(position, player_height)
        }
    }

    fn canopy_drag(&self, position: Vec3) -> f32 {
        if self.near_moon(position) { 0.0 } else { self.home.canopy_drag(position) }
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.home.placed_camera_tripod().or_else(|| self.moon.placed_camera_tripod())
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let a = self.home.step_surface(dt);
        let b = self.moon.step_surface(dt);
        a || b
    }

    fn lethal_at(&self, position: Vec3) -> bool {
        if self.near_moon(position) { self.moon.lethal_at(position) } else { self.home.lethal_at(position) }
    }

    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 {
        if self.near_moon(position) {
            self.moon.confine_beneath_crust(position)
        } else if position.abs().max_element() <= self.home_half + 36.0 {
            self.home.confine_beneath_crust(position)
        } else {
            position
        }
    }

    fn face_count(&self) -> usize { 12 }

    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) {
        match index {
            0..=5 => self.home.emit_face(index, target),
            6..=11 => self.moon.emit_face(index - 6, target),
            12..=17 => self.home.emit_face(index - 6, target),
            _ => {}
        }
    }

    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        if self.moon.pending_slices() {
            Self::map_moon_slice(self.moon.emit_modified_face(target))
        } else {
            Self::map_home_slice(self.home.emit_modified_face(target))
        }
    }

    fn pending_slices(&self) -> bool {
        self.home.pending_slices() || self.moon.pending_slices()
    }

    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> {
        if let Some(idx) = self.home.drain_modified_slice(target) {
            return Some(Self::map_home_slice(idx));
        }
        self.moon.drain_modified_slice(target).map(Self::map_moon_slice)
    }

    fn face_patch_count(&self, slice: usize) -> usize {
        match slice {
            0..=5 => self.home.face_patch_count(slice),
            6..=11 => self.moon.face_patch_count(slice - 6),
            _ => 0,
        }
    }

    fn emit_face_patch(&self, slice: usize, patch_slot: usize, target: &mut dyn MeshTarget) {
        match slice {
            0..=5 => self.home.emit_face_patch(slice, patch_slot, target),
            6..=11 => self.moon.emit_face_patch(slice - 6, patch_slot, target),
            _ => {}
        }
    }

    fn drain_modified_patch(&mut self, target: &mut dyn MeshTarget) -> Option<(usize, usize)> {
        if let Some(hit) = self.home.drain_modified_patch(target) {
            return Some(hit);
        }
        self.moon.drain_modified_patch(target).map(|(s, slot)| (s + 6, slot))
    }

    fn drain_stale_meadow_patch(&self, center: Vec3, ring: &mut MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        if position_near_home(center, self.home_half) {
            self.home.drain_stale_meadow_patch(center, ring, target)
        } else {
            None
        }
    }

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        if position_near_home(center, self.home_half) {
            self.home.emit_meadow_around(center, view_dir, target);
        }
    }

    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> {
        if self.near_moon(center) { None } else { self.home.arboreal_perch_near(center, selector) }
    }

    fn woodland_habitat(&self, position: Vec3) -> bool {
        !self.near_moon(position) && position_near_home(position, self.home_half)
    }

    fn local_body_point(&self, position: Vec3) -> Vec3 {
        if self.near_moon(position) {
            self.moon.local_body_point(position)
        } else {
            position
        }
    }

    fn visible_slice_mask(&self, position: Vec3) -> u32 {
        let mut mask = self.moon.visible_faces_from(position, LunarWorld::HALF_SIZE) << 6;
        let show_flora = position.length() <= self.home_half + 420.0;
        let up = self.home.surface_up_at(position);
        for (idx, zone) in ClimateZone::all().into_iter().enumerate() {
            let n = zone.normal();
            if position.dot(n) > -self.home_half * 0.45 || n.dot(up) > -0.5 {
                mask |= 1u32 << idx;
                if show_flora {
                    mask |= 1u32 << (12 + idx);
                }
            }
        }
        if position_near_home(position, self.home_half) {
            mask |= 1u32 << 24;
        }
        mask
    }

    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) {
        self.home.drain_disclosures(out);
        let mut moon_buf = Vec::new();
        self.moon.drain_disclosures(&mut moon_buf);
        for (face, stratum) in moon_buf {
            out.push((face + 6, stratum));
        }
    }
}

fn position_near_home(pos: Vec3, home_half: f32) -> bool {
    pos.abs().max_element() <= home_half + 48.0
}
