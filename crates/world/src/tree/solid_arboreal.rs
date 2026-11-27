use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::arboreal::Arboreal;
use super::depository::TimberDepository;
use super::timber_yield::SurfaceFind;

#[derive(Clone, Copy, Debug)]
struct TrunkBarrier {
    u: f32,
    v: f32,
    base_elevation: f32,
    radius: f32,
}

/// Decorator constraining horizontal motion around an arboreal specimen ("Tell, Don't Ask").
pub struct SolidArboreal {
    origin: Box<dyn Arboreal>,
    trunk: TrunkBarrier,
    barrier_height: f32,
}

impl SolidArboreal {
    const ROOT_TOLERANCE: f32 = 0.45;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn rooted(
        origin: Box<dyn Arboreal>,
        u: f32,
        v: f32,
        base_elevation: f32,
        radius: f32,
        barrier_height: f32,
    ) -> Self {
        Self {
            origin,
            trunk: TrunkBarrier { u, v, base_elevation, radius },
            barrier_height,
        }
    }

    /// Secondary constructor anchoring the trunk barrier at datum elevation `0.0`.
    pub fn new(origin: Box<dyn Arboreal>, u: f32, v: f32, radius: f32, barrier_height: f32) -> Self {
        Self::rooted(origin, u, v, 0.0, radius, barrier_height)
    }

    fn retain_if_standing(&self, next: Box<dyn Arboreal>) -> Box<dyn Arboreal> {
        if next.standing() {
            let t = self.trunk;
            Box::new(Self::rooted(next, t.u, t.v, t.base_elevation, t.radius, self.barrier_height))
        } else {
            next
        }
    }
}

impl Arboreal for SolidArboreal {
    fn strike(&self, cleave: u32, recipient: &mut dyn TimberDepository) -> Box<dyn Arboreal> {
        let struck = self.origin.strike(cleave, recipient);
        self.retain_if_standing(struck)
    }

    fn cleave_at(
        &self,
        eye: Vec3,
        direction: Vec3,
        recipient: &mut dyn TimberDepository,
    ) -> (Box<dyn Arboreal>, bool) {
        let (struck, hit) = self.origin.cleave_at(eye, direction, recipient);
        (self.retain_if_standing(struck), hit)
    }

    fn forage_at(&self, eye: Vec3, direction: Vec3) -> (Box<dyn Arboreal>, SurfaceFind) {
        let (next, find) = self.origin.forage_at(eye, direction);
        (self.retain_if_standing(next), find)
    }

    fn step_timber(&mut self, dt: f32) -> bool {
        self.origin.step_timber(dt)
    }

    fn settle_onto(&mut self, face: &dyn crate::Face, frame: &SurfaceFrame) -> bool {
        let probe = frame.point_at(self.trunk.u, self.trunk.v, self.trunk.base_elevation + 0.25);
        let next_elev = frame.elevation_of(face.constrain(probe, 0.0));
        if (next_elev - self.trunk.base_elevation).abs() > 1e-3 {
            self.trunk.base_elevation = next_elev;
        }
        self.origin.settle_onto(face, frame)
    }

    fn solid(&self) -> bool {
        self.origin.standing()
    }

    fn standing(&self) -> bool {
        self.origin.standing()
    }

    fn constrain_surface(
        &self,
        base: Vec3,
        u: f32,
        v: f32,
        frame: &SurfaceFrame,
        distance: f32,
    ) -> Vec3 {
        let delegated = self.origin.constrain_surface(base, u, v, frame, distance);
        if !self.origin.standing() {
            return delegated;
        }

        let support_elev = frame.elevation_of(delegated) - distance;
        if support_elev < self.trunk.base_elevation - Self::ROOT_TOLERANCE
            || support_elev > self.trunk.base_elevation + self.barrier_height
        {
            return delegated;
        }

        let dist = (u - self.trunk.u).hypot(v - self.trunk.v);
        if dist <= self.trunk.radius {
            let base_elev = frame.elevation_of(delegated);
            frame.point_at(u, v, base_elev + self.barrier_height)
        } else {
            delegated
        }
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.origin.emit_surface(target);
    }

    fn canopy_drag(&self, u: f32, v: f32, height: f32) -> f32 {
        self.origin.canopy_drag(u, v, height)
    }
}
