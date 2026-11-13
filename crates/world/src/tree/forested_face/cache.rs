use std::sync::RwLock;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{Face, MeshTarget, Quad};
use super::super::arboreal::Arboreal;

struct FloraQuadTarget<'a>(&'a mut Vec<Quad>);

impl MeshTarget for FloraQuadTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        self.0.push(quad);
    }
}

/// Cached per-specimen flora quad lists for a planetary face.
pub(super) struct FloraCache {
    slots: RwLock<Vec<Vec<Quad>>>,
    settled: bool,
}

impl FloraCache {
    pub(super) const fn new(slots: RwLock<Vec<Vec<Quad>>>, settled: bool) -> Self {
        Self { slots, settled }
    }

    pub(super) fn build_tree_quads(tree: &dyn Arboreal) -> Vec<Quad> {
        let mut quads = Vec::with_capacity(32);
        let mut collector = FloraQuadTarget(&mut quads);
        tree.emit_surface(&mut collector);
        quads
    }

    pub(super) fn refresh_slot(&mut self, idx: usize, trees: &[Box<dyn Arboreal>]) {
        let flora = self.slots.get_mut().expect("Flora cache lock poisoned");
        if flora.len() == trees.len() {
            flora[idx] = Self::build_tree_quads(trees[idx].as_ref());
        }
    }

    pub(super) fn append_specimen(&mut self, tree: &dyn Arboreal, total_after: usize) {
        let flora = self.slots.get_mut().expect("Flora cache lock poisoned");
        if flora.len() + 1 == total_after {
            flora.push(Self::build_tree_quads(tree));
        } else {
            flora.clear();
        }
    }

    pub(super) fn mark_settled(&mut self) {
        self.settled = true;
    }

    pub(super) fn take_settled(&mut self) -> bool {
        let prev = self.settled;
        self.settled = false;
        prev
    }

    pub(super) fn emit(&self, trees: &[Box<dyn Arboreal>], target: &mut dyn MeshTarget) {
        if let Ok(flora) = self.slots.read() {
            if flora.len() == trees.len() {
                for tree_quads in flora.iter() {
                    for &quad in tree_quads {
                        target.push_quad(quad);
                    }
                }
                return;
            }
        }
        let mut flora = self.slots.write().expect("Flora cache lock poisoned");
        if flora.len() != trees.len() {
            flora.clear();
            flora.reserve(trees.len());
            for tree in trees {
                flora.push(Self::build_tree_quads(tree.as_ref()));
            }
        }
        for tree_quads in flora.iter() {
            for &quad in tree_quads {
                target.push_quad(quad);
            }
        }
    }
}

/// Resolves deterministic resting ground coordinates when an item is cast onto a face.
pub(super) struct LitterLanding<'a> {
    origin: &'a dyn Face,
    frame: SurfaceFrame,
}

impl<'a> LitterLanding<'a> {
    pub(super) const fn new(origin: &'a dyn Face, frame: SurfaceFrame) -> Self {
        Self { origin, frame }
    }

    fn base_spot(&self, eye: Vec3, direction: Vec3) -> Option<(f32, f32, f32)> {
        let dir = direction.normalize_or_zero();
        let eye_elev = self.frame.elevation_of(eye);
        let mut last_open: Option<(f32, f32, f32)> = None;
        for step in 4..=46 {
            let p = eye + dir * ((step as f32) * 0.06);
            if !self.origin.matches(p) {
                continue;
            }
            let ray_elev = self.frame.elevation_of(p);
            let (pu, pv) = self.frame.decompose(p);
            let ground_elev = self.frame.elevation_of(self.origin.constrain(p, 0.0));
            if ground_elev > ray_elev + 0.55 {
                if let Some(spot) = last_open {
                    return Some(spot);
                }
                break;
            }
            if ray_elev <= ground_elev + 0.05 {
                let (cu, cv) = self.frame.clamp_interior(pu, pv, 0.22);
                return Some((cu, cv, ground_elev));
            }
            last_open = Some((pu, pv, ground_elev));
        }
        if !self.origin.matches(eye) {
            return None;
        }
        let (eu, ev) = self.frame.decompose(eye);
        let (du, dv) = self.frame.decompose(dir);
        let plen = du.hypot(dv);
        let (ux, vx) = if plen > 1e-3 { (du / plen, dv / plen) } else { (0.0, 1.0) };
        let (mut bu, mut bv) = self.frame.clamp_interior(eu, ev, 0.22);
        let mut belev = self.frame.elevation_of(self.origin.constrain(self.frame.point_at(bu, bv, eye_elev), 0.0));
        for dist in [0.30, 0.52, 0.74, 0.95] {
            let (cu, cv) = self.frame.clamp_interior(eu + ux * dist, ev + vx * dist, 0.22);
            let g = self.frame.elevation_of(self.origin.constrain(self.frame.point_at(cu, cv, eye_elev), 0.0));
            if g > eye_elev + 0.25 {
                break;
            }
            (bu, bv, belev) = (cu, cv, g);
        }
        Some((bu, bv, belev))
    }

    pub(super) fn resolve(
        &self,
        eye: Vec3,
        direction: Vec3,
        trees: &[Box<dyn Arboreal>],
    ) -> Option<(f32, f32, f32, f32)> {
        let (bu, bv, belev) = self.base_spot(eye, direction)?;
        let eye_elev = self.frame.elevation_of(eye);
        for attempt in 0..8usize {
            let (u, v, elev) = if attempt == 0 {
                (bu, bv, belev)
            } else {
                let ang = (attempt as f32) * 2.39996;
                let rad = 0.24 + 0.06 * ((attempt / 4) as f32);
                let (cu, cv) = self.frame.clamp_interior(bu + rad * ang.cos(), bv + rad * ang.sin(), 0.22);
                let g = self.frame.elevation_of(self.origin.constrain(self.frame.point_at(cu, cv, eye_elev), 0.0));
                if g > eye_elev + 0.25 {
                    continue;
                }
                (cu, cv, g)
            };
            if !trees.iter().any(|t| t.crowds_spot(u, v, elev)) {
                let yaw = (u * 3.7 + v * 5.3 + (trees.len() as f32) * 0.85).rem_euclid(std::f32::consts::TAU);
                return Some((u, v, elev, yaw));
            }
        }
        let yaw = (bu * 3.7 + bv * 5.3 + (trees.len() as f32) * 0.85).rem_euclid(std::f32::consts::TAU);
        Some((bu, bv, belev, yaw))
    }
}
