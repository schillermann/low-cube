use std::f32::consts::PI;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::cell::GridCell;
use super::super::mold::MoldArchetype;
use super::super::wedge::WedgeHeading;

/// Mounting orientation of an aerodynamic `RocketCamera` pod toggled via mouse wheel scroll.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensorPodAxis {
    NoseCone,
    BellyPod,
    SideFairing(WedgeHeading),
}

impl SensorPodAxis {
    /// Resolves nose-cone zenith (`0`), belly nadir (`1`), or lateral fairing (`2`) from archetype and heading.
    pub const fn from_mold(archetype: MoldArchetype, heading: WedgeHeading) -> Self {
        match archetype.code() % 3 {
            0 => Self::NoseCone,
            1 => Self::BellyPod,
            _ => Self::SideFairing(heading),
        }
    }

    /// Sorting rank placing nose-cone cameras first (`2`), belly landing pods second (`1`), and side pods third (`0`).
    pub const fn channel_rank(self) -> u8 {
        match self {
            Self::NoseCone => 2,
            Self::BellyPod => 1,
            Self::SideFairing(_) => 0,
        }
    }

    /// Optical `(eye_offset_from_cell_base, forward, up)` ray in the craft's local `SurfaceFrame`.
    pub fn lens_ray(self, cell: GridCell, base_elev: f32, col_top: f32, min_floor: f32, ship_fwd: Vec3, frame: &SurfaceFrame) -> (Vec3, Vec3, Vec3) {
        let (uc, vc) = cell.center();
        let ship_up = frame.normal().normalize_or_zero();
        let base = frame.point_at(uc, vc, base_elev);
        match self {
            Self::NoseCone => {
                let lift = 0.615_f32.max(col_top - base_elev + 0.115);
                (base + ship_up * lift, ship_up, -ship_fwd)
            }
            Self::BellyPod => {
                let drop = (base_elev - min_floor).max(0.0) + 0.325;
                (base - ship_up * drop, -ship_up, ship_fwd)
            }
            Self::SideFairing(heading) => {
                let out_dir = -heading.forward_vector(frame).normalize_or_zero();
                (base + ship_up * 0.26 + out_dir * 0.30, out_dir, ship_up)
            }
        }
    }
}

/// Sculpted aerodynamic rocket camera pod rendering as a nose-cone dome, belly landing turret, or hull blister fairing.
pub struct RocketCameraMesh {
    axis: SensorPodAxis,
}

impl RocketCameraMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(axis: SensorPodAxis) -> Self {
        Self { axis }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_frustum(
        origin: Vec3,
        axis: Vec3,
        ref_up: Vec3,
        z0: f32,
        r0: f32,
        z1: f32,
        r1: f32,
        col: Vec3,
        glow: f32,
        caps: (bool, bool),
        target: &mut dyn MeshTarget,
    ) {
        let sa = axis.cross(ref_up).normalize_or_zero();
        let sb = axis.cross(sa).normalize_or_zero();
        let (segs, step) = (8usize, (2.0 * PI) / 8.0);
        let (mid0, mid1) = (origin + axis * z0, origin + axis * z1);
        for i in 0..segs {
            let ((s0, c0), (s1, c1)) = (((i as f32) * step).sin_cos(), (((i + 1) as f32) * step).sin_cos());
            let (d0, d1) = (sa * c0 + sb * s0, sa * c1 + sb * s1);
            let [a0, a1] = [mid0 + d0 * r0, mid0 + d1 * r0];
            let [b0, b1] = [mid1 + d0 * r1, mid1 + d1 * r1];
            let norm = (a1 - a0).cross(b0 - a0).normalize_or_zero();
            target.push_quad(Quad::glowing(a0, a1, b1, b0, norm, col, glow));
            if caps.0 {
                target.push_quad(Quad::glowing(mid0, a1, a0, a0, -axis, col, glow));
            }
            if caps.1 {
                target.push_quad(Quad::glowing(mid1, b0, b1, b1, axis, col, glow));
            }
        }
    }

    pub fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (uc, vc) = cell.center();
        let up = frame.normal();
        let tan = frame.tangent();
        let ceramic = Vec3::new(0.84, 0.88, 0.93);
        let titanium = Vec3::new(0.46, 0.52, 0.60);
        let graphite = Vec3::new(0.16, 0.18, 0.23);
        let brass = Vec3::new(0.90, 0.72, 0.24);
        let sapphire = Vec3::new(0.24, 0.88, 0.98);

        match self.axis {
            SensorPodAxis::NoseCone => {
                let base = frame.point_at(uc, vc, floor_elev);
                Self::emit_frustum(base, up, tan, 0.0, 0.235, 0.055, 0.215, titanium, 0.0, (true, false), target);
                Self::emit_frustum(base, up, tan, 0.055, 0.220, 0.080, 0.208, brass, 0.0, (true, true), target);
                Self::emit_frustum(base, up, tan, 0.080, 0.205, 0.250, 0.132, ceramic, 0.0, (false, false), target);
                Self::emit_frustum(base, up, tan, 0.250, 0.132, 0.375, 0.076, graphite, 0.0, (false, false), target);
                Self::emit_frustum(base, up, tan, 0.375, 0.080, 0.415, 0.064, brass, 0.0, (true, true), target);
                Self::emit_frustum(base, up, tan, 0.415, 0.058, 0.470, 0.026, sapphire, 0.88, (false, true), target);
            }
            SensorPodAxis::BellyPod => {
                let ceil = frame.point_at(uc, vc, floor_elev + 0.50);
                let down = -up;
                Self::emit_frustum(ceil, down, tan, 0.0, 0.205, 0.060, 0.190, titanium, 0.0, (true, true), target);
                Self::emit_frustum(ceil, down, tan, 0.060, 0.148, 0.260, 0.112, graphite, 0.0, (false, false), target);
                Self::emit_frustum(ceil, down, tan, 0.260, 0.118, 0.305, 0.094, brass, 0.0, (true, true), target);
                Self::emit_frustum(ceil, down, tan, 0.305, 0.086, 0.365, 0.042, sapphire, 0.88, (false, true), target);
            }
            SensorPodAxis::SideFairing(heading) => {
                let center = frame.point_at(uc, vc, floor_elev + 0.25);
                let out_dir = -heading.forward_vector(frame).normalize_or_zero();
                Self::emit_frustum(center, out_dir, up, -0.25, 0.185, -0.185, 0.172, titanium, 0.0, (true, true), target);
                Self::emit_frustum(center, out_dir, up, -0.185, 0.145, -0.020, 0.105, ceramic, 0.0, (false, false), target);
                Self::emit_frustum(center, out_dir, up, -0.020, 0.110, 0.045, 0.082, graphite, 0.0, (false, false), target);
                Self::emit_frustum(center, out_dir, up, 0.045, 0.086, 0.085, 0.068, brass, 0.0, (true, true), target);
                Self::emit_frustum(center, out_dir, up, 0.085, 0.062, 0.135, 0.028, sapphire, 0.88, (false, true), target);
            }
        }
    }
}
