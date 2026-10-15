use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::arboreal::Arboreal;
use super::depository::TimberDepository;

/// Null Object representing a vacant ground plot where vegetation has been completely harvested.
pub struct ClearedPlot {
    u: f32,
    v: f32,
    elevation: f32,
}

impl ClearedPlot {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(u: f32, v: f32, elevation: f32) -> Self {
        Self {
            u,
            v,
            elevation,
        }
    }
}

impl Arboreal for ClearedPlot {
    fn strike(&self, _cleave: u32, _recipient: &mut dyn TimberDepository) -> Box<dyn Arboreal> {
        Box::new(Self::new(self.u, self.v, self.elevation))
    }

    fn solid(&self) -> bool {
        false
    }

    fn standing(&self) -> bool {
        false
    }

    fn constrain_surface(
        &self,
        base: Vec3,
        _u: f32,
        _v: f32,
        _frame: &SurfaceFrame,
        _distance: f32,
    ) -> Vec3 {
        base
    }

    fn emit_surface(&self, _target: &mut dyn MeshTarget) {}

    fn canopy_drag(&self, _u: f32, _v: f32, _height: f32) -> f32 {
        0.0
    }
}
