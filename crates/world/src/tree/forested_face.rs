use std::sync::RwLock;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::{BlockMedium, BlockMold, Face, MeshTarget, Quad};
use super::arboreal::Arboreal;
use super::depository::WoodlandDepository;
use super::timber_yield::{CleaveOutcome, SurfaceFind};
use super::zoned_forest::ZonedForest;

struct FloraQuadTarget<'a>(&'a mut Vec<Quad>);

impl MeshTarget for FloraQuadTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        self.0.push(quad);
    }
}

/// Composite decorator populating a planetary face with climate-adapted arboreal specimens.
pub struct ForestedFace {
    origin: Box<dyn Face>,
    frame: SurfaceFrame,
    trees: Vec<Box<dyn Arboreal>>,
    cached_flora: RwLock<Vec<Vec<Quad>>>,
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
            cached_flora,
        }
    }

    /// Secondary constructor populating the face with native trees and gatherable floor litter.
    pub fn in_zone(origin: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> Self {
        let frame = zone.surface_frame(half_size);
        let trees = ZonedForest::new(zone).populate_face(origin.as_ref(), frame, half_size);
        Self::new(origin, frame, trees, RwLock::new(Vec::new()))
    }

    fn build_tree_quads(tree: &dyn Arboreal) -> Vec<Quad> {
        let mut quads = Vec::with_capacity(32);
        let mut collector = FloraQuadTarget(&mut quads);
        tree.emit_surface(&mut collector);
        quads
    }

    fn refresh_tree_cache(&mut self, idx: usize) {
        let flora = self.cached_flora.get_mut().expect("Flora cache lock poisoned");
        if flora.len() == self.trees.len() {
            flora[idx] = Self::build_tree_quads(self.trees[idx].as_ref());
        }
    }
}

impl Face for ForestedFace {
    fn matches(&self, point: Vec3) -> bool { self.origin.matches(point) }
    fn surface_up(&self) -> Vec3 { self.origin.surface_up() }
    fn pull_direction(&self) -> Vec3 { self.origin.pull_direction() }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        let mut position = self.origin.constrain(point, distance);
        if self.frame.elevation_of(point) < -2.0 {
            return position;
        }
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

    fn emit_flora(&self, target: &mut dyn MeshTarget) {
        if let Ok(flora) = self.cached_flora.read() {
            if flora.len() == self.trees.len() {
                for tree_quads in flora.iter() {
                    for &quad in tree_quads { target.push_quad(quad); }
                }
                return;
            }
        }
        let mut flora = self.cached_flora.write().expect("Flora cache lock poisoned");
        if flora.len() != self.trees.len() {
            flora.clear();
            flora.reserve(self.trees.len());
            for tree in &self.trees {
                flora.push(Self::build_tree_quads(tree.as_ref()));
            }
        }
        for tree_quads in flora.iter() {
            for &quad in tree_quads { target.push_quad(quad); }
        }
    }

    fn canopy_drag(&self, point: Vec3) -> f32 {
        let (u, v) = self.frame.decompose(point);
        let height = self.frame.height_of(point);
        let mut peak = self.origin.canopy_drag(point);
        for tree in &self.trees {
            let drag = tree.canopy_drag(u, v, height);
            if drag > peak { peak = drag; }
        }
        peak
    }

    fn excavate(&mut self, eye: Vec3, direction: Vec3) -> bool { self.origin.excavate(eye, direction) }
    fn surface_medium(&self, eye: Vec3, direction: Vec3) -> BlockMedium { self.origin.surface_medium(eye, direction) }
    fn loosen(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> bool { self.origin.loosen(eye, direction, stage) }

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
                self.refresh_tree_cache(idx);
                return outcome;
            }
        }
        CleaveOutcome::Missed
    }

    fn forage_timber(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        for idx in 0..self.trees.len() {
            let (next_tree, find) = self.trees[idx].forage_at(eye, direction);
            if find.gathered() {
                self.trees[idx] = next_tree;
                self.refresh_tree_cache(idx);
                return find;
            }
        }
        SurfaceFind::EmptyHanded
    }

    fn place_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.origin.place_block(eye, direction, avatar_pos)
    }

    fn place_molded_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        self.origin.place_molded_block(eye, direction, avatar_pos, mold)
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        self.origin.step_surface(dt)
    }

    fn step_flora(&mut self, dt: f32) -> bool {
        let mut timber_moved = false;
        for idx in 0..self.trees.len() {
            if self.trees[idx].step_timber(dt) {
                self.refresh_tree_cache(idx);
                timber_moved = true;
            }
        }
        timber_moved
    }

    fn weightless_at(&self, point: Vec3) -> bool { self.origin.weightless_at(point) }
    fn lethal_at(&self, point: Vec3) -> bool { self.origin.lethal_at(point) }
    fn confine_beneath_crust(&self, point: Vec3) -> Vec3 { self.origin.confine_beneath_crust(point) }
}
