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
                (base + ship_up * 0.25 + out_dir * 0.15, out_dir, ship_up)
            }
        }
    }
}

/// Orthonormal half-block (`0.25m`) mounting basis for the V1 nose-cone dome camera pod.
#[derive(Clone, Copy)]
struct PodMount {
    origin: Vec3,
    axis: Vec3,
    front: Vec3,
    side: Vec3,
}

impl PodMount {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(origin: Vec3, axis: Vec3, front: Vec3, side: Vec3) -> Self {
        Self { origin, axis, front, side }
    }

    fn aligned(origin: Vec3, axis: Vec3, ref_up: Vec3) -> Self {
        let unit_axis = axis.normalize_or_zero();
        let front = unit_axis.cross(ref_up).normalize_or_zero();
        let side = unit_axis.cross(front).normalize_or_zero();
        Self::new(origin, unit_axis, front, side)
    }

    fn emit_frustum(
        &self,
        z_span: (f32, f32),
        radii: (f32, f32),
        col: Vec3,
        glow: f32,
        caps: (bool, bool),
        target: &mut dyn MeshTarget,
    ) {
        let (segs, step) = (8usize, (2.0 * PI) / 8.0);
        let half_step = 0.5 * step;
        let (mid0, mid1) = (self.origin + self.axis * z_span.0, self.origin + self.axis * z_span.1);
        for i in 0..segs {
            let a0_ang = (i as f32) * step - half_step;
            let a1_ang = a0_ang + step;
            let ((s0, c0), (s1, c1)) = (a0_ang.sin_cos(), a1_ang.sin_cos());
            let (d0, d1) = (self.front * c0 + self.side * s0, self.front * c1 + self.side * s1);
            let [a0, a1] = [mid0 + d0 * radii.0, mid0 + d1 * radii.0];
            let [b0, b1] = [mid1 + d0 * radii.1, mid1 + d1 * radii.1];
            let norm = (a1 - a0).cross(b0 - a0).normalize_or_zero();
            target.push_quad(Quad::glowing(a0, a1, b1, b0, norm, col, glow));
            if caps.0 {
                target.push_quad(Quad::glowing(mid0, a1, a0, a0, -self.axis, col, glow));
            }
            if caps.1 {
                target.push_quad(Quad::glowing(mid1, b0, b1, b1, self.axis, col, glow));
            }
        }
    }

    fn emit_tally_lamp(&self, target: &mut dyn MeshTarget) {
        let ruby = Vec3::new(0.98, 0.20, 0.18);
        let cos_oct = (PI / 8.0).cos();
        let depth = 0.193 * cos_oct + 0.003;
        let (w, z0, z1) = (0.015, 0.038, 0.058);
        let pt = |sx: f32, az: f32| self.origin + self.front * depth + self.side * sx + self.axis * az;
        target.push_quad(Quad::glowing(
            pt(-w, z0),
            pt(w, z0),
            pt(w, z1),
            pt(-w, z1),
            self.front,
            ruby,
            0.96,
        ));
    }

    fn emit_v1_dome(&self, target: &mut dyn MeshTarget) {
        let ceramic = Vec3::new(0.86, 0.89, 0.94);
        let graphite = Vec3::new(0.16, 0.18, 0.23);
        let brass = Vec3::new(0.88, 0.70, 0.24);
        let sapphire_rim = Vec3::new(0.16, 0.76, 0.98);
        let sapphire_crown = Vec3::new(0.38, 0.94, 1.00);

        self.emit_frustum((0.000, 0.022), (0.212, 0.206), brass, 0.0, (true, true), target);
        self.emit_frustum((0.022, 0.076), (0.196, 0.190), graphite, 0.0, (false, false), target);
        self.emit_tally_lamp(target);
        self.emit_frustum((0.076, 0.094), (0.198, 0.190), brass, 0.0, (true, true), target);
        self.emit_frustum((0.094, 0.182), (0.184, 0.118), ceramic, 0.0, (false, false), target);
        self.emit_frustum((0.182, 0.206), (0.122, 0.104), brass, 0.0, (true, true), target);
        self.emit_frustum((0.206, 0.234), (0.094, 0.064), sapphire_rim, 0.88, (false, false), target);
        self.emit_frustum((0.234, 0.250), (0.064, 0.026), sapphire_crown, 0.96, (false, true), target);
    }
}

/// Sculpted half-block (`0.25m`) V1 nose-cone dome rocket camera pod across zenith, nadir, and side mounts.
pub struct RocketCameraMesh {
    axis: SensorPodAxis,
}

impl RocketCameraMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(axis: SensorPodAxis) -> Self {
        Self { axis }
    }

    pub fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (uc, vc) = cell.center();
        let up = frame.normal();
        let tan = frame.tangent();
        let mount = match self.axis {
            SensorPodAxis::NoseCone => PodMount::aligned(frame.point_at(uc, vc, floor_elev), up, tan),
            SensorPodAxis::BellyPod => PodMount::aligned(frame.point_at(uc, vc, floor_elev + 0.50), -up, tan),
            SensorPodAxis::SideFairing(heading) => {
                let out_dir = -heading.forward_vector(frame).normalize_or_zero();
                let wall = frame.point_at(uc, vc, floor_elev + 0.25) - out_dir * 0.25;
                PodMount::aligned(wall, out_dir, up)
            }
        };
        mount.emit_v1_dome(target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ExtentProbe {
        min_pt: Vec3,
        max_pt: Vec3,
        glowing_quads: usize,
    }

    impl ExtentProbe {
        const fn empty() -> Self {
            Self {
                min_pt: Vec3::splat(f32::INFINITY),
                max_pt: Vec3::splat(f32::NEG_INFINITY),
                glowing_quads: 0,
            }
        }
    }

    impl MeshTarget for ExtentProbe {
        fn push_quad(&mut self, quad: Quad) {
            if quad.radiate(|g| g > 0.8) {
                self.glowing_quads += 1;
            }
            quad.transfer(|v0, v1, v2, v3, _, _| {
                for v in [v0, v1, v2, v3] {
                    self.min_pt = self.min_pt.min(v);
                    self.max_pt = self.max_pt.max(v);
                }
            });
        }
    }

    #[test]
    fn v1_rocket_camera_dome_is_exactly_half_block_tall_across_all_mounts() {
        let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let cell = GridCell::new(0, 0);
        let half_block = GridCell::SPAN * 0.5;

        let mut nose = ExtentProbe::empty();
        RocketCameraMesh::new(SensorPodAxis::NoseCone).emit(cell, 1.0, &frame, &mut nose);
        assert!((nose.min_pt.y - 1.0).abs() < 1e-5);
        assert!(((nose.max_pt.y - nose.min_pt.y) - half_block).abs() < 1e-5);
        assert!(nose.glowing_quads >= 17);

        let mut belly = ExtentProbe::empty();
        RocketCameraMesh::new(SensorPodAxis::BellyPod).emit(cell, 1.0, &frame, &mut belly);
        assert!((belly.max_pt.y - 1.5).abs() < 1e-5);
        assert!(((belly.max_pt.y - belly.min_pt.y) - half_block).abs() < 1e-5);

        let mut side = ExtentProbe::empty();
        RocketCameraMesh::new(SensorPodAxis::SideFairing(WedgeHeading::South)).emit(cell, 1.0, &frame, &mut side);
        assert!(((side.max_pt.z - side.min_pt.z) - half_block).abs() < 1e-5);
    }
}
