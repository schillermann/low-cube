use glam::Vec3;
use crate::{Face, MeshTarget};

/// Null Object pattern implementation for Face.
pub struct NullFace;

impl Face for NullFace {
    fn matches(&self, _point: Vec3) -> bool {
        false
    }

    fn surface_up(&self) -> Vec3 {
        Vec3::Y
    }

    fn pull_direction(&self) -> Vec3 {
        Vec3::NEG_Y
    }

    fn constrain(&self, point: Vec3, _distance: f32) -> Vec3 {
        point
    }

    fn emit_surface(&self, _target: &mut dyn MeshTarget) {}
}
