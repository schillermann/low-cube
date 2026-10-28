use glam::Vec3;
use crate::{MeshTarget, Quad, SurfaceFrame};
use super::arboreal::Arboreal;
use super::cleared_plot::ClearedPlot;
use super::depository::TimberDepository;
use super::timber_yield::SurfaceFind;

/// Gatherable deadwood branch or flint nodule resting by woodland trees or rocky outcrops.
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
    fn emit_bough(
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

    #[allow(clippy::too_many_arguments)]
    fn emit_nodule(
        &self,
        bot: (f32, f32, f32),
        top: (f32, f32, f32),
        br: (f32, f32),
        tr: (f32, f32),
        segs: usize,
        pigment: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let (u, v, elevation) = self.coords;
        let (sy, cy) = self.yaw.sin_cos();
        let map = |x: f32, z: f32, h: f32| {
            let (ru, rv) = (x * cy - z * sy, x * sy + z * cy);
            self.frame.point_at(u + ru, v + rv, elevation + h)
        };
        let step = std::f32::consts::TAU / (segs as f32);
        let top_mid = map(top.0, top.1, top.2);
        for i in 0..segs {
            let (s0, c0) = (((i as f32) + 0.5) * step).sin_cos();
            let (s1, c1) = (((i as f32) + 1.5) * step).sin_cos();
            let p0 = map(bot.0 + br.0 * c0, bot.1 + br.1 * s0, bot.2);
            let p1 = map(bot.0 + br.0 * c1, bot.1 + br.1 * s1, bot.2);
            let p2 = map(top.0 + tr.0 * c1, top.1 + tr.1 * s1, top.2);
            let p3 = map(top.0 + tr.0 * c0, top.1 + tr.1 * s0, top.2);
            let n = (p1 - p0).cross(p3 - p0).normalize_or_zero();
            target.push_quad(Quad::new(p0, p1, p2, p3, if n.length_squared() > 0.5 { n } else { self.frame.normal() }, pigment));
            let tn = (p3 - top_mid).cross(p2 - top_mid).normalize_or_zero();
            if tn.length_squared() > 0.5 {
                target.push_quad(Quad::new(top_mid, p3, p2, p2, tn, pigment * 1.12));
            }
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
        let (eu, ev) = self.frame.decompose(eye);
        let (du, dv) = self.frame.decompose(dir);
        for step in 2..=75 {
            let t = (step as f32) * 0.04;
            let (su, sv) = (eu + du * t, ev + dv * t);
            let sh = self.frame.elevation_of(eye + dir * t) - elevation;
            if (su - u).hypot(sv - v) <= 0.46 && (-0.28..=0.52).contains(&sh) {
                return (Box::new(ClearedPlot::new(u, v, elevation)), self.find);
            }
        }
        (Box::new(*self), SurfaceFind::EmptyHanded)
    }

    fn solid(&self) -> bool { false }
    fn standing(&self) -> bool { false }

    fn constrain_surface(&self, base: Vec3, _u: f32, _v: f32, _frame: &SurfaceFrame, _distance: f32) -> Vec3 {
        base
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        let (s, c) = self.yaw.sin_cos();
        match self.find {
            SurfaceFind::EmptyHanded => {}
            SurfaceFind::DeadwoodBough => {
                let (bark, dark_bark, heart) = (Vec3::new(0.46, 0.29, 0.16), Vec3::new(0.36, 0.23, 0.12), Vec3::new(0.82, 0.66, 0.38));
                self.emit_bough((-0.22 * c, -0.22 * s, 0.032), (0.01 * c - 0.02 * s, 0.01 * s + 0.02 * c, 0.038), 0.034, 0.028, bark, heart, target);
                self.emit_bough((0.01 * c - 0.02 * s, 0.01 * s + 0.02 * c, 0.038), (0.23 * c, 0.23 * s, 0.052), 0.028, 0.018, dark_bark, heart, target);
                let (fc, fs) = ((self.yaw + 0.68).cos(), (self.yaw + 0.68).sin());
                self.emit_bough((0.02 * c, 0.02 * s, 0.040), (0.02 * c + 0.17 * fc, 0.02 * s + 0.17 * fs, 0.088), 0.020, 0.010, bark * 1.10, heart, target);
            }
            SurfaceFind::FlintStone => {
                let (cortex, core, flake, edge) = (
                    Vec3::new(0.76, 0.70, 0.58), Vec3::new(0.19, 0.22, 0.28),
                    Vec3::new(0.52, 0.60, 0.70), Vec3::new(0.80, 0.86, 0.94),
                );
                self.emit_nodule((0.0, 0.0, 0.0), (0.0, 0.0, 0.035), (0.125, 0.095), (0.115, 0.088), 7, cortex, target);
                self.emit_nodule((0.0, 0.0, 0.030), (-0.012, 0.008, 0.085), (0.110, 0.084), (0.064, 0.048), 7, core, target);
                self.emit_nodule((-0.024, 0.014, 0.042), (-0.042, 0.026, 0.122), (0.062, 0.046), (0.012, 0.022), 5, flake, target);
                self.emit_nodule((0.028, -0.012, 0.038), (0.045, -0.020, 0.108), (0.052, 0.038), (0.010, 0.018), 5, edge, target);
            }
        }
    }

    fn canopy_drag(&self, _u: f32, _v: f32, _height: f32) -> f32 { 0.0 }
}
