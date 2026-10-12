use glam::Vec3;
use crate::{Face, MeshTarget, OffsetMeshTarget};

/// Decorator elevating a planetary face outwards along its surface normal.
pub struct ElevatedFace {
    face: Box<dyn Face>,
    elevation: f32,
}

impl ElevatedFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(face: Box<dyn Face>, elevation: f32) -> Self {
        Self { face, elevation }
    }
}

impl Face for ElevatedFace {
    fn matches(&self, point: Vec3) -> bool {
        self.face.matches(point)
    }

    fn surface_up(&self) -> Vec3 {
        self.face.surface_up()
    }

    fn pull_direction(&self) -> Vec3 {
        self.face.pull_direction()
    }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        self.face.constrain(point, distance + self.elevation)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        let mut offset_target = OffsetMeshTarget::new(target, self.face.surface_up() * self.elevation);
        self.face.emit_surface(&mut offset_target);
    }
}
