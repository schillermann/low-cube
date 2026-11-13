mod cache;

use std::sync::RwLock;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::{BlockMedium, BlockMold, Face, MeshTarget, Quad, VehicleAssembly};
use super::arboreal::Arboreal;
use super::depository::WoodlandDepository;
use super::floor_litter::FloorLitter;
use super::timber_yield::{CleaveOutcome, SurfaceFind};
use super::zoned_forest::ZonedForest;
use self::cache::{FloraCache, LitterLanding};

/// Composite decorator populating a planetary face with climate-adapted arboreal specimens.
pub struct ForestedFace {
    origin: Box<dyn Face>,
    frame: SurfaceFrame,
    trees: Vec<Box<dyn Arboreal>>,
    cache: FloraCache,
}

impl ForestedFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        origin: Box<dyn Face>,
        frame: SurfaceFrame,
        trees: Vec<Box<dyn Arboreal>>,
        cached_flora: RwLock<Vec<Vec<Quad>>>,
    ) -> Self {
        Self {
            origin,
            frame,
            trees,
            cache: FloraCache::new(cached_flora, false),
        }
    }

    /// Secondary constructor populating the face with native trees and gatherable floor litter.
    pub fn in_zone(origin: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> Self {
        let frame = zone.surface_frame(half_size);
        let trees = ZonedForest::new(zone).populate_face(origin.as_ref(), frame, half_size);
        Self::new(origin, frame, trees, RwLock::new(Vec::new()))
    }

    fn settle_flora(&mut self) {
        for idx in 0..self.trees.len() {
            if self.trees[idx].settle_onto(self.origin.as_ref(), &self.frame) {
                self.cache.refresh_slot(idx, &self.trees);
                self.cache.mark_settled();
            }
        }
    }
}

impl Face for ForestedFace {
    fn matches(&self, point: Vec3) -> bool { self.origin.matches(point) }
    fn surface_up(&self) -> Vec3 { self.origin.surface_up() }
    fn pull_direction(&self) -> Vec3 { self.origin.pull_direction() }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        let mut position = self.origin.constrain(point, distance);
        let (u, v) = self.frame.decompose(point);
        for tree in &self.trees {
            position = tree.constrain_surface(position, u, v, &self.frame, distance);
        }
        position
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_surface(target);
        self.emit_flora(target);
    }

    fn emit_terrain(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_terrain(target);
    }

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        self.origin.emit_meadow_around(center, view_dir, target);
    }

    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> {
        let variant = selector as u32;
        let near_count = self
            .trees
            .iter()
            .filter_map(|t| t.crown_perch(variant))
            .filter(|&p| (p - center).length_squared() <= 576.0)
            .count();
        if near_count > 0 {
            return self
                .trees
                .iter()
                .filter_map(|t| t.crown_perch(variant))
                .filter(|&p| (p - center).length_squared() <= 576.0)
                .nth(selector % near_count);
        }
        let total = self.trees.iter().filter_map(|t| t.crown_perch(variant)).count();
        (total > 0).then(|| self.trees.iter().filter_map(|t| t.crown_perch(variant)).nth(selector % total)).flatten()
    }

    fn emit_flora(&self, target: &mut dyn MeshTarget) {
        self.cache.emit(&self.trees, target);
    }

    fn canopy_drag(&self, point: Vec3) -> f32 {
        let (u, v) = self.frame.decompose(point);
        let elev = self.frame.elevation_of(point);
        let mut peak = self.origin.canopy_drag(point);
        for tree in &self.trees {
            let drag = tree.canopy_drag(u, v, elev);
            if drag > peak { peak = drag; }
        }
        peak
    }

    fn harvest(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        let harvested = self.origin.harvest(eye, direction);
        if harvested.is_some() {
            self.settle_flora();
        }
        harvested
    }

    fn loosen(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        self.origin.loosen(eye, direction, stage)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        let mut depository = WoodlandDepository::new();
        for idx in 0..self.trees.len() {
            let (next_tree, hit) = self.trees[idx].cleave_at(eye, direction, &mut depository);
            if hit {
                let outcome = if next_tree.standing() {
                    CleaveOutcome::Notched
                } else {
                    CleaveOutcome::Felled
                };
                self.trees[idx] = next_tree;
                self.cache.refresh_slot(idx, &self.trees);
                return outcome;
            }
        }
        CleaveOutcome::Missed
    }

    fn forage_timber(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        for idx in (0..self.trees.len()).rev() {
            let (next_tree, find) = self.trees[idx].forage_at(eye, direction);
            if find.gathered() {
                self.trees[idx] = next_tree;
                self.cache.refresh_slot(idx, &self.trees);
                return find;
            }
        }
        SurfaceFind::EmptyHanded
    }

    fn cast_litter(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        if !find.gathered() {
            return false;
        }
        let landing = LitterLanding::new(self.origin.as_ref(), self.frame);
        let Some((u, v, elev, yaw)) = landing.resolve(eye, direction, &self.trees) else {
            return false;
        };
        let specimen = FloorLitter::new(self.frame, u, v, elev, find, yaw);
        self.trees.push(Box::new(specimen));
        self.cache.append_specimen(&specimen, self.trees.len());
        true
    }

    fn place_molded_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        let changed = self.origin.place_molded_block(eye, direction, avatar_pos, mold);
        if changed {
            self.settle_flora();
        }
        changed
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        self.origin.adjust_rig(eye, direction, delta_steps, rotate_quarter)
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.origin.lower_rig(eye, direction)
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

    fn step_flora(&mut self, dt: f32) -> bool {
        let mut timber_moved = self.cache.take_settled();
        for idx in 0..self.trees.len() {
            if self.trees[idx].step_timber(dt) {
                self.cache.refresh_slot(idx, &self.trees);
                timber_moved = true;
            }
        }
        timber_moved
    }

    fn lethal_at(&self, point: Vec3) -> bool { self.origin.lethal_at(point) }
    fn confine_beneath_crust(&self, point: Vec3) -> Vec3 { self.origin.confine_beneath_crust(point) }
    fn accept_disclosure(&mut self, stratum: crate::ExposedStratum) { self.origin.accept_disclosure(stratum); }
    fn drain_disclosures(&mut self, out: &mut Vec<crate::ExposedStratum>) { self.origin.drain_disclosures(out); }
}
