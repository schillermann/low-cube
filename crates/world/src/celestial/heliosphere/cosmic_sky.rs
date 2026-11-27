use glam::{Quat, Vec3};
use crate::relief::ReliefPalette;
use crate::{ClimateZone, CosmicPos, MeshTarget, Quad};
use super::super::alien_worlds::AlienWorld;
use super::super::catalog::{BodyId, SurfaceNature};
use super::super::ephemeris::BodySnapshot;
use super::super::spin::{ApexSpin, PlanetarySpin};

const COSMIC_HORIZON: f64 = 2.0e6;
pub(super) const EARTH_VOXEL_HANDOFF: f32 = 40_000.0;
const ORBITAL_SHELL_DIVS: i32 = 4;

fn emit_sphere_shell(
    center: Vec3,
    viewer: Vec3,
    radius: f32,
    pose: Quat,
    color_fn: impl Fn(ClimateZone) -> Vec3,
    glow: f32,
    target: &mut dyn MeshTarget,
) {
    let step = (radius * 2.0) / (ORBITAL_SHELL_DIVS as f32);
    for zone in ClimateZone::all() {
        let frame = zone.surface_frame(radius);
        let col = color_fn(zone);
        for sv in 0..ORBITAL_SHELL_DIVS {
            let v0 = -radius + (sv as f32) * step;
            let v1 = v0 + step;
            for su in 0..ORBITAL_SHELL_DIVS {
                let u0 = -radius + (su as f32) * step;
                let u1 = u0 + step;
                let p00 = center + pose * frame.point_at(u0, v0, 0.0);
                let p10 = center + pose * frame.point_at(u1, v0, 0.0);
                let p11 = center + pose * frame.point_at(u1, v1, 0.0);
                let p01 = center + pose * frame.point_at(u0, v1, 0.0);
                let mid = (p00 + p10 + p11 + p01) * 0.25;
                let n = (mid - center).try_normalize().unwrap_or(Vec3::Y);
                if (mid - viewer).dot(n) < 0.0 {
                    target.push_quad(Quad::glowing(p00, p10, p11, p01, n, col, glow));
                }
            }
        }
    }
}

/// Gas-giant nested `QuadSphere` cloud shells and concentric Saturn rings at camera-relative `CosmicPos` (`<= 4` fields).
pub(super) struct GiantCloudShell {
    snap: BodySnapshot,
    epoch: f32,
}

impl GiantCloudShell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(snap: BodySnapshot, epoch: f32) -> Self {
        Self { snap, epoch }
    }

    pub(super) fn emit_near(self, viewer: Vec3, target: &mut dyn MeshTarget) {
        let viewer_cosmic = CosmicPos::from_vec3(viewer);
        let rel_d = self.snap.cosmic_locus().relative_dvec3(viewer_cosmic);
        let dist = rel_d.length();
        let h_real = self.snap.half_size();
        if dist <= h_real * 1.02 {
            return;
        }
        let k = (COSMIC_HORIZON / dist).min(1.0);
        let c = viewer + (rel_d * k).as_vec3();
        let h = ((h_real * k) as f32).max(0.5);
        let (c_primary, c_band) = self.snap.palette();
        emit_sphere_shell(c, viewer, h * 0.96, Quat::from_rotation_y(self.epoch * 0.03), |_| c_band, 0.0, target);
        emit_sphere_shell(c, viewer, h, Quat::from_rotation_y(-self.epoch * 0.018), |_| c_primary, -2.55, target);
        if self.snap.nature().ringed_giant() {
            let (ri, ro) = (h * 1.45, h * 2.45);
            let ring_col = Vec3::new(0.86, 0.80, 0.64);
            let segs = 12;
            for sign in [-1.0_f32, 1.0_f32] {
                let y = c + Vec3::Y * (sign * (h * 0.008).max(0.05));
                let n = Vec3::Y * sign;
                for i in 0..segs {
                    let a0 = (i as f32) * core::f32::consts::TAU / (segs as f32);
                    let a1 = ((i + 1) as f32) * core::f32::consts::TAU / (segs as f32);
                    let (s0, c0) = a0.sin_cos();
                    let (s1, c1) = a1.sin_cos();
                    let p0 = y + Vec3::new(ro * c0, 0.0, ro * s0);
                    let p1 = y + Vec3::new(ro * c1, 0.0, ro * s1);
                    let p2 = y + Vec3::new(ri * c1, 0.0, ri * s1);
                    let p3 = y + Vec3::new(ri * c0, 0.0, ri * s0);
                    target.push_quad(Quad::glowing(p0, p1, p2, p3, n, ring_col, 0.24));
                }
            }
        }
    }
}

/// True-metric 3D celestial `QuadSphere` emitter for `SOL`, spinning `EARTH`, and rocky worlds (`<= 4` fields).
pub(super) struct CosmicBodyCube {
    snap: BodySnapshot,
    spin: ApexSpin,
}

impl CosmicBodyCube {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(snap: BodySnapshot, spin: ApexSpin) -> Self {
        Self { snap, spin }
    }

    pub(super) fn emit_for_viewer(
        self,
        viewer: Vec3,
        uploaded_ids: [Option<BodyId>; 2],
        earth_voxel_active: bool,
        target: &mut dyn MeshTarget,
    ) {
        match self.snap.nature() {
            SurfaceNature::SolarStar => self.emit_solar_star(viewer, target),
            SurfaceNature::Homeworld => self.emit_distant_earth(viewer, earth_voxel_active, target),
            SurfaceNature::RockyWorld if !uploaded_ids.contains(&Some(self.snap.id())) => {
                self.emit_rocky_sphere(viewer, target);
            }
            _ => {}
        }
    }

    fn emit_solar_star(self, viewer: Vec3, target: &mut dyn MeshTarget) {
        let viewer_d = viewer.as_dvec3();
        let apparent_sun = self.spin.apparent_sun_center(self.snap.center(), viewer_d);
        let rel_d = apparent_sun - viewer_d;
        let dist = rel_d.length();
        let h_real = self.snap.half_size();
        if dist <= h_real * 1.02 {
            return;
        }
        let k = (COSMIC_HORIZON / dist).min(1.0);
        let c = viewer + (rel_d * k).as_vec3();
        let h = ((h_real * k) as f32).max(0.05);
        emit_sphere_shell(c, viewer, h, Quat::IDENTITY, |_| Vec3::new(1.00, 0.92, 0.58), 1.0, target);
    }

    fn emit_distant_earth(self, viewer: Vec3, earth_voxel_active: bool, target: &mut dyn MeshTarget) {
        let viewer_cosmic = CosmicPos::from_vec3(viewer);
        let rel_d = self.snap.cosmic_locus().relative_dvec3(viewer_cosmic);
        let dist = rel_d.length();
        let min_dist = if earth_voxel_active {
            f64::from(EARTH_VOXEL_HANDOFF - 50.0)
        } else {
            self.snap.half_size() * 1.02
        };
        if dist <= min_dist {
            return;
        }
        let k = (COSMIC_HORIZON / dist).min(1.0);
        let c = viewer + (rel_d * k).as_vec3();
        let h = ((self.snap.half_size() * k) as f32).max(0.05);
        let step = (h * 2.0) / (ORBITAL_SHELL_DIVS as f32);
        for zone in ClimateZone::all() {
            let frame = zone.surface_frame(h);
            let col = ReliefPalette::for_zone(zone).inspect_surface(|s| s);
            for sv in 0..ORBITAL_SHELL_DIVS {
                let (v0, v1) = (-h + (sv as f32) * step, -h + ((sv + 1) as f32) * step);
                for su in 0..ORBITAL_SHELL_DIVS {
                    let (u0, u1) = (-h + (su as f32) * step, -h + ((su + 1) as f32) * step);
                    let p00 = c + self.spin.pose_distant_earth(frame.point_at(u0, v0, 0.0));
                    let p10 = c + self.spin.pose_distant_earth(frame.point_at(u1, v0, 0.0));
                    let p11 = c + self.spin.pose_distant_earth(frame.point_at(u1, v1, 0.0));
                    let p01 = c + self.spin.pose_distant_earth(frame.point_at(u0, v1, 0.0));
                    let mid = (p00 + p10 + p11 + p01) * 0.25;
                    let n = (mid - c).try_normalize().unwrap_or(Vec3::Y);
                    if (mid - viewer).dot(n) < 0.0 {
                        target.push_quad(Quad::glowing(p00, p10, p11, p01, n, col, 0.24));
                    }
                }
            }
        }
    }

    fn emit_rocky_sphere(self, viewer: Vec3, target: &mut dyn MeshTarget) {
        let viewer_cosmic = CosmicPos::from_vec3(viewer);
        let rel_d = self.snap.cosmic_locus().relative_dvec3(viewer_cosmic);
        let dist = rel_d.length();
        let h_real = self.snap.half_size();
        if dist <= h_real * 1.02 {
            return;
        }
        let k = (COSMIC_HORIZON / dist).min(1.0);
        let c = viewer + (rel_d * k).as_vec3();
        let h = ((h_real * k) as f32).max(0.02);
        let id = self.snap.id();
        emit_sphere_shell(c, viewer, h, Quat::IDENTITY, |zone| AlienWorld::zone_tones(id, zone).0, 0.28, target);
    }
}
