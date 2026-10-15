use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::arboreal::Arboreal;
use super::depository::TimberDepository;

/// Decorator constraining horizontal motion around an arboreal specimen ("Tell, Don't Ask").
pub struct SolidArboreal {
    origin: Box<dyn Arboreal>,
    u: f32,
    v: f32,
    radius: f32,
    barrier_height: f32,
}

impl SolidArboreal {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn Arboreal>, u: f32, v: f32, radius: f32, barrier_height: f32) -> Self {
        Self {
            origin,
            u,
            v,
            radius,
            barrier_height,
        }
    }
}

impl Arboreal for SolidArboreal {
    fn strike(&self, cleave: u32, recipient: &mut dyn TimberDepository) -> Box<dyn Arboreal> {
        let struck = self.origin.strike(cleave, recipient);
        if struck.standing() {
            Box::new(Self::new(struck, self.u, self.v, self.radius, self.barrier_height))
        } else {
            struck
        }
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

        let dist = (u - self.u).hypot(v - self.v);
        if dist <= self.radius {
            let base_h = frame.height_of(delegated);
            frame.point_at(u, v, base_h + self.barrier_height)
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
