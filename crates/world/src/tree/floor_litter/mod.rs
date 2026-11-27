mod tool_mesh;

use glam::Vec3;
use crate::{BlockMold, MeshTarget, Quad, SurfaceFrame};
use super::arboreal::Arboreal;
use super::cleared_plot::ClearedPlot;
use super::depository::TimberDepository;
use super::timber_yield::SurfaceFind;
use self::tool_mesh::GroundToolFigure;

/// Cast EVA rig tool, radio, alloy billet, or loose voxel block resting on the planetary floor.
#[derive(Clone, Copy, Debug)]
pub struct FloorLitter {
    frame: SurfaceFrame,
    coords: (f32, f32, f32),
    find: SurfaceFind,
    yaw: f32,
}

impl FloorLitter {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        frame: SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        find: SurfaceFind,
        yaw: f32,
    ) -> Self {
        Self {
            frame,
            coords: (u, v, elevation),
            find,
            yaw,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_bough(
        &self,
        a: (f32, f32, f32),
        b: (f32, f32, f32),
        r0: f32,
        r1: f32,
        pigment: Vec3,
        cap_col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let (u, v, elevation) = self.coords;
        let p0 = self.frame.point_at(u + a.0, v + a.1, elevation + a.2);
        let p1 = self.frame.point_at(u + b.0, v + b.1, elevation + b.2);
        let axis = (p1 - p0).normalize_or_zero();
        let up = self.frame.normal();
        let side = axis.cross(up).normalize_or_zero();
        let binorm = side.cross(axis).normalize_or_zero();
        let step = std::f32::consts::TAU / 6.0;
        for i in 0..6 {
            let (s0, c0) = ((i as f32) * step).sin_cos();
            let (s1, c1) = (((i + 1) as f32) * step).sin_cos();
            let (d0, d1) = (side * c0 + binorm * s0, side * c1 + binorm * s1);
            let (v0, v1, v2, v3) = (p0 + d0 * r0, p0 + d1 * r0, p1 + d1 * r1, p1 + d0 * r1);
            let n = (v1 - v0).cross(v3 - v0).normalize_or_zero();
            target.push_quad(Quad::new(v0, v1, v2, v3, if n.length_squared() > 0.5 { n } else { d0 }, pigment));
            target.push_quad(Quad::new(p0, v0, v1, v1, -axis, cap_col));
            target.push_quad(Quad::new(p1, v3, v2, v2, axis, cap_col));
        }
    }
}

impl Arboreal for FloorLitter {
    fn strike(&self, _cleave: u32, _recipient: &mut dyn TimberDepository) -> Box<dyn Arboreal> {
        Box::new(*self)
    }

    fn forage_at(&self, eye: Vec3, direction: Vec3) -> (Box<dyn Arboreal>, SurfaceFind) {
        if !self.find.gathered() {
            return (Box::new(*self), SurfaceFind::EmptyHanded);
        }
        let (u, v, elevation) = self.coords;
        let dir = direction.normalize_or_zero();
        for step in 2..=75 {
            let sample = eye + dir * ((step as f32) * 0.04);
            let (su, sv) = self.frame.decompose(sample);
            let sh = self.frame.elevation_of(sample) - elevation;
            if (su - u).hypot(sv - v) <= 0.46 && (-0.28..=0.52).contains(&sh) {
                return (Box::new(ClearedPlot::vacant(self.frame, u, v, elevation)), self.find);
            }
        }
        (Box::new(*self), SurfaceFind::EmptyHanded)
    }

    fn solid(&self) -> bool { false }
    fn standing(&self) -> bool { false }

    fn crowds_spot(&self, u: f32, v: f32, elevation: f32) -> bool {
        self.find.gathered()
            && (self.coords.0 - u).hypot(self.coords.1 - v) < 0.22
            && (self.coords.2 - elevation).abs() < 0.35
    }

    fn settle_onto(&mut self, face: &dyn crate::Face, frame: &SurfaceFrame) -> bool {
        if !self.find.gathered() {
            return false;
        }
        let (u, v, elev) = self.coords;
        let probe = frame.point_at(u, v, elev + 0.25);
        let next_elev = frame.elevation_of(face.constrain(probe, 0.0));
        if (next_elev - elev).abs() > 1e-3 {
            self.coords.2 = next_elev;
            true
        } else {
            false
        }
    }

    fn constrain_surface(&self, base: Vec3, _u: f32, _v: f32, _frame: &SurfaceFrame, _distance: f32) -> Vec3 {
        base
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        let (s, c) = self.yaw.sin_cos();
        match self.find {
            SurfaceFind::EmptyHanded => {}
            SurfaceFind::LooseBlock(medium) => {
                let (u, v, elevation) = self.coords;
                let origin = self.frame.point_at(u, v, elevation + 0.008);
                let right = self.frame.tangent() * c + self.frame.bitangent() * s;
                let forward = self.frame.bitangent() * c - self.frame.tangent() * s;
                BlockMold::cube().in_medium(medium).emit_miniature(
                    origin,
                    right,
                    self.frame.normal(),
                    forward,
                    0.44,
                    Vec3::new(0.44, 0.31, 0.19),
                    Vec3::new(0.28, 0.65, 0.25),
                    target,
                );
            }
            _ => GroundToolFigure::new(self, self.yaw, self.find).emit(target),
        }
    }

    fn canopy_drag(&self, _u: f32, _v: f32, _height: f32) -> f32 { 0.0 }
}
