use glam::Vec3;
use crate::{Face, MeshTarget, TintedMeshTarget};

/// Decorator scaling surface vertex coloration of an underlying face by an encapsulated tint.
pub struct TintedFace {
    face: Box<dyn Face>,
    tint: Vec3,
}

impl TintedFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(face: Box<dyn Face>, tint: Vec3) -> Self {
        Self { face, tint }
    }
}

impl Face for TintedFace {
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
        self.face.constrain(point, distance)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        let mut tinted_target = TintedMeshTarget::new(target, self.tint);
        self.face.emit_surface(&mut tinted_target);
    }

    fn canopy_drag(&self, point: Vec3) -> f32 {
        self.face.canopy_drag(point)
    }
}
