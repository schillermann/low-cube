use glam::{Quat, Vec3};
use crate::relief::ReliefPalette;
use crate::{ClimateZone, MeshTarget, Quad};
use super::super::alien_worlds::AlienWorld;
use super::super::catalog::{BodyId, SurfaceNature};
use super::super::ephemeris::BodySnapshot;
use super::super::spin::{ApexSpin, PlanetarySpin};

const COSMIC_HORIZON: f64 = 2.0e6;
pub(super) const EARTH_VOXEL_HANDOFF: f32 = 40_000.0;

/// Gas-giant nested cloud-cube and concentric cubic Saturn-ring emitter at true 3D metric coordinates (`<= 4` fields).
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
        let rel_d = self.snap.center() - viewer.as_dvec3();
        let dist = rel_d.length();
        let h_real = self.snap.half_size();
        if dist <= h_real * 1.02 {
            return;
        }
        let k = (COSMIC_HORIZON / dist).min(1.0);
        let c = viewer + (rel_d * k).as_vec3();
        let h = ((h_real * k) as f32).max(0.5);
        let (c_primary, c_band) = self.snap.palette();
        let spin_inner = Quat::from_rotation_y(self.epoch * 0.03);
        let spin_outer = Quat::from_rotation_y(-self.epoch * 0.018);
        for zone in ClimateZone::all() {
            let (nl, ul, vl) = (zone.normal(), zone.tangent(), zone.bitangent());
            let (ni, ui, vi) = (spin_inner * nl, spin_inner * ul * (h * 0.94), spin_inner * vl * (h * 0.94));
            let fci = c + ni * (h * 0.94);
            if (fci - viewer).dot(ni) < 0.0 {
                target.push_quad(Quad::new(fci - ui - vi, fci + ui - vi, fci + ui + vi, fci - ui + vi, ni, c_band));
            }
            let (no, uo, vo) = (spin_outer * nl, spin_outer * ul * h, spin_outer * vl * h);
            let fco = c + no * h;
            if (fco - viewer).dot(no) < 0.0 {
                target.push_quad(Quad::glowing(
                    fco - uo - vo, fco + uo - vo, fco + uo + vo, fco - uo + vo, no, c_primary, -2.55,
                ));
            }
        }
        if self.snap.nature().ringed_giant() {
            let (ri, ro) = (h * 1.45, h * 2.45);
            let ring_col = Vec3::new(0.86, 0.80, 0.64);
            for sign in [-1.0_f32, 1.0_f32] {
                let y = c + Vec3::Y * (sign * (h * 0.008).max(0.05));
                let n = Vec3::Y * sign;
                target.push_quad(Quad::glowing(
                    y + Vec3::new(-ro, 0.0, -ro), y + Vec3::new(ro, 0.0, -ro),
                    y + Vec3::new(ri, 0.0, -ri), y + Vec3::new(-ri, 0.0, -ri), n, ring_col, 0.24,
                ));
                target.push_quad(Quad::glowing(
                    y + Vec3::new(ro, 0.0, -ro), y + Vec3::new(ro, 0.0, ro),
                    y + Vec3::new(ri, 0.0, ri), y + Vec3::new(ri, 0.0, -ri), n, ring_col * 0.94, 0.24,
                ));
                target.push_quad(Quad::glowing(
                    y + Vec3::new(ro, 0.0, ro), y + Vec3::new(-ro, 0.0, ro),
                    y + Vec3::new(-ri, 0.0, ri), y + Vec3::new(ri, 0.0, ri), n, ring_col, 0.24,
                ));
                target.push_quad(Quad::glowing(
                    y + Vec3::new(-ro, 0.0, ro), y + Vec3::new(-ro, 0.0, -ro),
                    y + Vec3::new(-ri, 0.0, -ri), y + Vec3::new(-ri, 0.0, ri), n, ring_col * 0.94, 0.24,
                ));
            }
        }
    }
}

/// True-metric 3D celestial cube emitter for `SOL`, distant spinning `EARTH`, and rocky worlds (`<= 4` fields).
pub(super) struct CosmicBodyCube {
    snap: BodySnapshot,
    spin: ApexSpin,
}

impl CosmicBodyCube {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(snap: BodySnapshot, spin: ApexSpin) -> Self {
        Self { snap, spin }
    }

    fn emit_mare_patches(
        fc: Vec3,
        (n, u, v): (Vec3, Vec3, Vec3),
        mare_col: Vec3,
        seed: u32,
        target: &mut dyn MeshTarget,
    ) {
        let lift = n * (u.length() * 0.010);
        let s0 = (((seed & 0xF) as f32) / 15.0) * 0.42 - 0.26;
        let t0 = ((((seed >> 4) & 0xF) as f32) / 15.0) * 0.42 - 0.22;
        let r0 = 0.24_f32;
        let p0 = fc + lift + u * s0 + v * t0;
        let (du0, dv0) = (u * r0, v * r0);
        target.push_quad(Quad::glowing(p0 - du0 - dv0, p0 + du0 - dv0, p0 + du0 + dv0, p0 - du0 + dv0, n, mare_col, 0.22));

        let s1 = ((((seed >> 8) & 0xF) as f32) / 15.0) * 0.48 - 0.14;
        let t1 = ((((seed >> 12) & 0xF) as f32) / 15.0) * 0.48 - 0.16;
        let r1 = 0.14_f32;
        let p1 = fc + lift * 1.3 + u * (-s1) + v * (-t1);
        let (du1, dv1) = (u * r1, v * r1);
        target.push_quad(Quad::glowing(p1 - du1 - dv1, p1 + du1 - dv1, p1 + du1 + dv1, p1 - du1 + dv1, n, mare_col * 0.90, 0.20));
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
                self.emit_rocky_cube(viewer, target);
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
        let (core_col, rim_col) = (Vec3::new(1.00, 0.96, 0.66), Vec3::new(1.00, 0.72, 0.24));
        for zone in ClimateZone::all() {
            let n = zone.normal();
            let fc = c + n * h;
            if (fc - viewer).dot(n) >= 0.0 {
                continue;
            }
            let (u, v) = (zone.tangent() * h, zone.bitangent() * h);
            target.push_quad(Quad::glowing(fc - u - v, fc + u - v, fc + u + v, fc - u + v, n, rim_col, 1.0));
            let (uc, vc, lift) = (u * 0.80, v * 0.80, fc + n * (h * 0.012));
            target.push_quad(Quad::glowing(lift - uc - vc, lift + uc - vc, lift + uc + vc, lift - uc + vc, n, core_col, 1.0));
        }
    }

    fn emit_distant_earth(self, viewer: Vec3, earth_voxel_active: bool, target: &mut dyn MeshTarget) {
        let rel_d = self.snap.center() - viewer.as_dvec3();
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
        for zone in ClimateZone::all() {
            let n = self.spin.pose_distant_earth(zone.normal());
            let fc = c + n * h;
            if (fc - viewer).dot(n) >= 0.0 {
                continue;
            }
            let u = self.spin.pose_distant_earth(zone.tangent()) * h;
            let v = self.spin.pose_distant_earth(zone.bitangent()) * h;
            let col = ReliefPalette::for_zone(zone).inspect_surface(|s| s);
            target.push_quad(Quad::glowing(fc - u - v, fc + u - v, fc + u + v, fc - u + v, n, col, 0.24));
        }
    }

    fn emit_rocky_cube(self, viewer: Vec3, target: &mut dyn MeshTarget) {
        let rel_d = self.snap.center() - viewer.as_dvec3();
        let dist = rel_d.length();
        let h_real = self.snap.half_size();
        if dist <= h_real * 1.02 {
            return;
        }
        let k = (COSMIC_HORIZON / dist).min(1.0);
        let c = viewer + (rel_d * k).as_vec3();
        let h = ((h_real * k) as f32).max(0.02);
        let id_seed = u32::from(self.snap.id().raw_code()).wrapping_mul(0x9E37_79B9);
        for zone in ClimateZone::all() {
            let n = zone.normal();
            let fc = c + n * h;
            if (fc - viewer).dot(n) >= 0.0 {
                continue;
            }
            let (u, v) = (zone.tangent() * h, zone.bitangent() * h);
            let (top, mare) = AlienWorld::zone_tones(self.snap.id(), zone);
            target.push_quad(Quad::glowing(fc - u - v, fc + u - v, fc + u + v, fc - u + v, n, top, 0.28));
            let face_seed = id_seed ^ ((zone.index() as u32 + 1).wrapping_mul(0x517C_C1B7));
            Self::emit_mare_patches(fc, (n, u, v), mare, face_seed, target);
        }
    }
}
