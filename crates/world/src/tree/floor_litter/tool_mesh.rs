use glam::Vec3;
use crate::MeshTarget;
use super::super::timber_yield::SurfaceFind;
use super::FloorLitter;

/// Sculpted 3D resting figure for EVA tools and alloy billets cast onto the planetary surface.
pub(super) struct GroundToolFigure<'a> {
    litter: &'a FloorLitter,
    yaw: f32,
    find: SurfaceFind,
}

impl<'a> GroundToolFigure<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(litter: &'a FloorLitter, yaw: f32, find: SurfaceFind) -> Self {
        Self { litter, yaw, find }
    }

    fn rot(&self, along: f32, across: f32, h: f32) -> (f32, f32, f32) {
        let (s, c) = self.yaw.sin_cos();
        (along * c - across * s, along * s + across * c, h)
    }

    fn bar(
        &self,
        a: (f32, f32, f32),
        b: (f32, f32, f32),
        r0: f32,
        r1: f32,
        col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let p0 = self.rot(a.0, a.1, a.2);
        let p1 = self.rot(b.0, b.1, b.2);
        self.litter.emit_bough(p0, p1, r0, r1, col, col * 0.92, target);
    }

    pub(super) fn emit(&self, target: &mut dyn MeshTarget) {
        match self.find {
            SurfaceFind::RigWrench => {
                let (grip, body,amber, plasma) = (
                    Vec3::new(0.18, 0.21, 0.26),
                    Vec3::new(0.68, 0.75, 0.84),
                    Vec3::new(0.96, 0.64, 0.14),
                    Vec3::new(0.24, 0.92, 1.00),
                );
                self.bar((-0.20, 0.0, 0.024), (0.04, 0.0, 0.026), 0.018, 0.022, grip, target);
                self.bar((0.02, 0.0, 0.028), (0.14, 0.0, 0.032), 0.028, 0.026, amber, target);
                self.bar((0.12, -0.042, 0.030), (0.23, -0.036, 0.028), 0.014, 0.010, body, target);
                self.bar((0.12, 0.042, 0.030), (0.23, 0.036, 0.028), 0.014, 0.010, body, target);
                self.bar((0.14, 0.0, 0.032), (0.22, 0.0, 0.032), 0.011, 0.006, plasma, target);
            }
            SurfaceFind::SuitRadio => {
                let (casing, screen, whip) = (
                    Vec3::new(0.20, 0.24, 0.30),
                    Vec3::new(0.18, 0.96, 0.66),
                    Vec3::new(0.76, 0.80, 0.86),
                );
                self.bar((-0.09, 0.0, 0.028), (0.07, 0.0, 0.030), 0.038, 0.038, casing, target);
                self.bar((-0.03, 0.0, 0.052), (0.04, 0.0, 0.054), 0.022, 0.022, screen, target);
                self.bar((0.06, 0.022, 0.034), (0.24, 0.026, 0.048), 0.006, 0.003, whip, target);
            }
            SurfaceFind::TitaniumBillet => {
                self.bar((-0.12, 0.0, 0.046), (0.12, 0.0, 0.048), 0.050, 0.038, Vec3::new(0.76, 0.84, 0.96), target);
            }
            _ => {}
        }
    }
}
