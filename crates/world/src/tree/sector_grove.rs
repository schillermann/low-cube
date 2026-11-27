use crate::{MeshTarget, Quad};
use super::arboreal::Arboreal;
use super::depository::WoodlandDepository;
use super::floor_litter::FloorLitter;
use super::timber_yield::{CleaveOutcome, SurfaceFind};
use crate::frame::SurfaceFrame;
use crate::Face;
use glam::Vec3;

struct GroveQuadSink<'a>(&'a mut Vec<Quad>);

impl MeshTarget for GroveQuadSink<'_> {
    fn push_quad(&mut self, quad: Quad) {
        self.0.push(quad);
    }
}

/// Cached `8×8`-cell (`4m × 4m`) woodland sector grove owning local specimens and quad caches (`<= 4` fields).
pub struct SectorGrove {
    trees: Vec<Box<dyn Arboreal>>,
    quads: Vec<Vec<Quad>>,
    revision: u32,
    animating: bool,
}

impl SectorGrove {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        trees: Vec<Box<dyn Arboreal>>,
        quads: Vec<Vec<Quad>>,
        revision: u32,
        animating: bool,
    ) -> Self {
        Self { trees, quads, revision, animating }
    }

    pub fn populated(trees: Vec<Box<dyn Arboreal>>) -> Self {
        let mut quads = Vec::with_capacity(trees.len());
        for t in &trees {
            quads.push(Self::build_quads(t.as_ref()));
        }
        Self::new(trees, quads, 0, false)
    }

    fn build_quads(tree: &dyn Arboreal) -> Vec<Quad> {
        let mut out = Vec::with_capacity(32);
        let mut sink = GroveQuadSink(&mut out);
        tree.emit_surface(&mut sink);
        out
    }

    fn bump(&mut self) {
        self.revision = self.revision.wrapping_add(1).max(1);
    }

    fn refresh_slot(&mut self, idx: usize) {
        if idx < self.trees.len() && self.quads.len() == self.trees.len() {
            self.quads[idx] = Self::build_quads(self.trees[idx].as_ref());
        } else {
            self.quads = self.trees.iter().map(|t| Self::build_quads(t.as_ref())).collect();
        }
        self.bump();
    }

    pub const fn revision(&self) -> u32 { self.revision }
    pub const fn animating(&self) -> bool { self.animating }
    pub const fn pristine(&self) -> bool { self.revision == 0 && !self.animating }

    pub fn constrain(&self, mut pos: Vec3, u: f32, v: f32, frame: &SurfaceFrame, dist: f32) -> Vec3 {
        for tree in &self.trees {
            pos = tree.constrain_surface(pos, u, v, frame, dist);
        }
        pos
    }

    pub fn max_drag(&self, u: f32, v: f32, elev: f32, mut peak: f32) -> f32 {
        for tree in &self.trees {
            peak = peak.max(tree.canopy_drag(u, v, elev));
        }
        peak
    }

    pub fn strike(&mut self, eye: Vec3, direction: Vec3, depository: &mut WoodlandDepository) -> CleaveOutcome {
        for idx in 0..self.trees.len() {
            let (next_tree, hit) = self.trees[idx].cleave_at(eye, direction, depository);
            if hit {
                let outcome = if next_tree.standing() {
                    CleaveOutcome::Notched
                } else {
                    CleaveOutcome::Felled
                };
                self.trees[idx] = next_tree;
                self.animating = true;
                self.refresh_slot(idx);
                return outcome;
            }
        }
        CleaveOutcome::Missed
    }

    pub fn forage(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        for idx in (0..self.trees.len()).rev() {
            let (next_tree, find) = self.trees[idx].forage_at(eye, direction);
            if find.gathered() {
                self.trees[idx] = next_tree;
                self.refresh_slot(idx);
                return find;
            }
        }
        SurfaceFind::EmptyHanded
    }

    pub fn append_litter(&mut self, specimen: FloorLitter) {
        let q = Self::build_quads(&specimen);
        self.trees.push(Box::new(specimen));
        self.quads.push(q);
        self.bump();
    }

    pub fn settle_onto(&mut self, origin: &dyn Face, frame: &SurfaceFrame) -> bool {
        let mut changed = false;
        for idx in 0..self.trees.len() {
            if self.trees[idx].settle_onto(origin, frame) {
                self.refresh_slot(idx);
                changed = true;
            }
        }
        changed
    }

    pub fn step_timber(&mut self, dt: f32) -> bool {
        let mut moved = false;
        for idx in 0..self.trees.len() {
            if self.trees[idx].step_timber(dt) {
                self.refresh_slot(idx);
                moved = true;
            }
        }
        self.animating = moved;
        moved
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        for list in &self.quads {
            for &quad in list {
                target.push_quad(quad);
            }
        }
    }
}
