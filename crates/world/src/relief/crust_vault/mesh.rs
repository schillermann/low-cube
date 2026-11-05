use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::{GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use super::CrustVault;

struct VaultCeilingTarget<'a> {
    geometry: VaultGeometry<'a>,
    tint: f32,
    target: &'a mut dyn MeshTarget,
}

impl MeshTarget for VaultCeilingTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let up = self.geometry.frame.normal();
        let rad_sq = self.geometry.vault.radius() * self.geometry.vault.radius();
        quad.transfer(|v0, v1, v2, v3, normal, _| {
            if normal.dot(up) >= -0.05 {
                return;
            }
            let clamp_seam = |p: Vec3| {
                let (a, b) = self.geometry.frame.decompose(p);
                let reach = a * a + b * b + a.abs().max(b.abs()).powi(2);
                if reach > rad_sq {
                    let (sa, sb, sh) = self.geometry.vault.shell_point(a, b);
                    self.geometry.frame.point_at(sa, sb, sh)
                } else {
                    p
                }
            };
            let (c0, c1, c2, c3) = (clamp_seam(v0), clamp_seam(v1), clamp_seam(v2), clamp_seam(v3));
            let (center, glow) = ((c0 + c1 + c2 + c3) * 0.25, if normal.dot(up) < -0.05 { 0.7 } else { 0.55 });
            let col = self.geometry.rock_tone(center) * self.tint;
            let quad_out = if (v2 - v3).length_squared() > 1e-6 && (c1 - c3).length_squared() < (c0 - c2).length_squared() {
                Quad::glowing(c1, c2, c3, c0, normal, col, glow)
            } else {
                Quad::glowing(c0, c1, c2, c3, normal, col, glow)
            };
            self.target.push_quad(quad_out);
        });
    }
}

/// Decorator over `CrustVault` and `SurfaceFrame` emitting magma-lit 3D crust and vault geometry.
#[derive(Clone, Copy)]
pub struct VaultGeometry<'a> {
    vault: &'a CrustVault,
    frame: &'a SurfaceFrame,
}

impl<'a> VaultGeometry<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(vault: &'a CrustVault, frame: &'a SurfaceFrame) -> Self {
        Self { vault, frame }
    }

    fn rock_tone(&self, at: Vec3) -> Vec3 {
        let dir = at.normalize();
        let vein = ((dir.x * 11.0 + dir.z * 4.0).sin() * (dir.y * 9.0 - dir.x * 3.0).cos()) * 0.5 + 0.5;
        let grit = (((at.x * 3.1).sin() + (at.y * 2.7).cos() + (at.z * 3.7).sin()) / 6.0 + 0.5).clamp(0.0, 1.0);
        Vec3::new(0.35, 0.31, 0.28)
            .lerp(Vec3::new(0.54, 0.38, 0.24), vein * 0.65)
            .lerp(Vec3::new(0.25, 0.21, 0.20), grit * 0.45)
    }

    fn emit_crag_relief(q0: Vec3, q1: Vec3, q3: Vec3, out_n: Vec3, tint: f32, target: &mut dyn MeshTarget) {
        let (va, vb) = ((q1 - q0) * 2.0, (q3 - q0) * 2.0);
        let (na, nb) = (va.normalize_or_zero(), vb.normalize_or_zero());
        for (a0, a1, b0, b1, d, raw_col, glow) in [
            (0.03, 0.21, 0.04, 0.22, 0.022, Vec3::new(0.43, 0.38, 0.35), 0.18),
            (0.25, 0.46, 0.05, 0.20, 0.018, Vec3::new(0.25, 0.21, 0.20), 0.18),
            (0.06, 0.27, 0.26, 0.45, 0.025, Vec3::new(0.38, 0.33, 0.30), 0.18),
            (0.30, 0.47, 0.24, 0.46, 0.020, Vec3::new(0.47, 0.41, 0.37), 0.18),
            (0.15, 0.35, 0.15, 0.29, 0.030, Vec3::new(0.84, 0.58, 0.26), 0.62),
            (0.34, 0.44, 0.08, 0.17, 0.028, Vec3::new(0.88, 0.68, 0.36), 0.70),
            (0.07, 0.17, 0.32, 0.41, 0.026, Vec3::new(0.76, 0.48, 0.20), 0.55),
        ] {
            let (col, side_col) = (raw_col * tint, raw_col * (tint * 0.86));
            let [r0, r1, r2, r3] = [
                q0 + va * a0 + vb * b0,
                q0 + va * a1 + vb * b0,
                q0 + va * a1 + vb * b1,
                q0 + va * a0 + vb * b1,
            ];
            let [t0, t1, t2, t3] = [r0 + out_n * d, r1 + out_n * d, r2 + out_n * d, r3 + out_n * d];
            target.push_quad(Quad::glowing(t0, t1, t2, t3, out_n, col, glow));
            target.push_quad(Quad::glowing(r0, r1, t1, t0, -nb, side_col, glow));
            target.push_quad(Quad::glowing(t3, t2, r2, r3, nb, side_col, glow));
            target.push_quad(Quad::glowing(r3, r0, t0, t3, -na, side_col, glow));
            target.push_quad(Quad::glowing(r1, r2, t2, t1, na, side_col, glow));
        }
    }

    pub fn emit_shaft_ceiling(&self, cell: GridCell, ceil_h: f32, tint: f32, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (c0, c1, c2, c3) = (
            self.frame.point_at(u0, v0, ceil_h),
            self.frame.point_at(u1, v0, ceil_h),
            self.frame.point_at(u1, v1, ceil_h),
            self.frame.point_at(u0, v1, ceil_h),
        );
        target.push_quad(Quad::glowing(c3, c2, c1, c0, -self.frame.normal(), Vec3::new(0.31, 0.27, 0.25) * tint, 0.35));
        Self::emit_crag_relief(c3, c2, c0, -self.frame.normal(), tint, target);
    }

    pub fn emit_crust_flank(&self, edge: ((f32, f32), (f32, f32)), layer: i32, floor_step: i32, norm: Vec3, target: &mut dyn MeshTarget) {
        let (((a0, b0), (a1, b1)), y0, y1) = (edge, (layer as f32) * 0.5, ((layer + 1) as f32) * 0.5);
        let shade = 0.92 + ((layer - floor_step) as f32) * 0.04;
        let ((seam, face), (ym0, ym1)) = ((Vec3::new(0.22, 0.19, 0.18), Vec3::new(0.31, 0.27, 0.25) * shade), (y0 + 0.035, y1 - 0.035));
        let (q0, q1, q3) = (self.frame.point_at(a0, b0, y0), self.frame.point_at(a1, b1, y0), self.frame.point_at(a0, b0, y1));
        target.push_quad(Quad::glowing(q0, q1, self.frame.point_at(a1, b1, ym0), self.frame.point_at(a0, b0, ym0), norm, seam, 0.24));
        target.push_quad(Quad::glowing(self.frame.point_at(a0, b0, ym0), self.frame.point_at(a1, b1, ym0), self.frame.point_at(a1, b1, ym1), self.frame.point_at(a0, b0, ym1), norm, face, 0.30));
        target.push_quad(Quad::glowing(self.frame.point_at(a0, b0, ym1), self.frame.point_at(a1, b1, ym1), self.frame.point_at(a1, b1, y1), q3, norm, seam, 0.24));
        Self::emit_crag_relief(q0, q1, q3, norm, shade, target);
    }

    pub fn emit_crust_floor(&self, cell: GridCell, top_h: f32, breached: bool, target: &mut dyn MeshTarget) -> Vec3 {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (p0, p1, p2, p3) = (
            self.frame.point_at(u0, v0, top_h),
            self.frame.point_at(u1, v0, top_h),
            self.frame.point_at(u1, v1, top_h),
            self.frame.point_at(u0, v1, top_h),
        );
        let col = Vec3::new(0.35, 0.31, 0.28);
        if !breached {
            target.push_quad(Quad::glowing(p0, p1, p2, p3, self.frame.normal(), col, 0.26));
            Self::emit_crag_relief(p0, p1, p3, self.frame.normal(), 1.0, target);
        }
        col
    }

    /// Emits the magma-lit inward-facing beveled block ceiling beneath this cell.
    pub fn emit_ceiling(&self, cell: GridCell, dig_stage: u8, target: &mut dyn MeshTarget) {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        if !self.vault.cell_opens(u, v) {
            return;
        }
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (floor_elev, block) = self.vault.block_at(cell);
        let c = self.vault.corners_at(u, v);
        let mut ceiling_out = VaultCeilingTarget {
            geometry: *self,
            tint: 1.0 - (dig_stage.min(3) as f32) * 0.14,
            target,
        };
        if [(u0, v0), (u1, v0), (u1, v1), (u0, v1)].iter().all(|&(a, b)| self.vault.opens_at(a, b)) {
            block.emit_shape(cell, floor_elev, self.frame, Vec3::ONE, &mut ceiling_out);
        } else {
            ceiling_out.push_quad(Quad::new(
                self.frame.point_at(u0, v1, c[3]),
                self.frame.point_at(u1, v1, c[2]),
                self.frame.point_at(u1, v0, c[1]),
                self.frame.point_at(u0, v0, c[0]),
                -self.frame.normal(),
                Vec3::ONE,
            ));
        }
        let steps = [
            ((u0, v1), (u1, v1), c[3], c[2], self.vault.corners_at(u, v + 1)[0], self.vault.corners_at(u, v + 1)[1], self.frame.bitangent()),
            ((u1, v0), (u0, v0), c[1], c[0], self.vault.corners_at(u, v - 1)[2], self.vault.corners_at(u, v - 1)[3], -self.frame.bitangent()),
            ((u1, v1), (u1, v0), c[2], c[1], self.vault.corners_at(u + 1, v)[3], self.vault.corners_at(u + 1, v)[0], self.frame.tangent()),
            ((u0, v0), (u0, v1), c[0], c[3], self.vault.corners_at(u - 1, v)[1], self.vault.corners_at(u - 1, v)[2], -self.frame.tangent()),
        ];
        for ((a0, b0), (a1, b1), ca, cb, na, nb, side_n) in steps {
            if (self.vault.opens_at(a0, b0) || self.vault.opens_at(a1, b1)) && (ca < na - 1e-3 || cb < nb - 1e-3) {
                let norm = (side_n * 0.2 - self.frame.normal()).normalize();
                if ca < na - 1e-3 && cb > nb + 1e-3 {
                    let mid = (self.frame.point_at(a0, b0, ca) + self.frame.point_at(a1, b1, nb)) * 0.5;
                    ceiling_out.push_quad(Quad::new(self.frame.point_at(a0, b0, ca), mid, mid, self.frame.point_at(a0, b0, na), norm, Vec3::ONE));
                } else if cb < nb - 1e-3 && ca > na + 1e-3 {
                    let mid = (self.frame.point_at(a0, b0, na) + self.frame.point_at(a1, b1, cb)) * 0.5;
                    ceiling_out.push_quad(Quad::new(mid, self.frame.point_at(a1, b1, cb), self.frame.point_at(a1, b1, nb), mid, norm, Vec3::ONE));
                } else {
                    ceiling_out.push_quad(Quad::new(
                        self.frame.point_at(a0, b0, ca.min(na)),
                        self.frame.point_at(a1, b1, cb.min(nb)),
                        self.frame.point_at(a1, b1, nb.max(cb)),
                        self.frame.point_at(a0, b0, na.max(ca)),
                        norm,
                        Vec3::ONE,
                    ));
                }
            }
        }
    }

    /// Emits the crust skirt sealing the gap between this cell's 4-layer floor and the vault toward a breached or upward-dug neighbor.
    pub fn emit_skirt(&self, cell: GridCell, side: (i32, i32), collar_h: f32, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let (floor, c) = ((self.vault.floor_steps_at(u, v) as f32) * 0.5, self.vault.corners_at(u, v));
        let ((a0, b0), (a1, b1), c0, c1, normal) = match side {
            (0, 1) => ((u0, v1), (u1, v1), c[3], c[2], self.frame.bitangent()),
            (0, _) => ((u1, v0), (u0, v0), c[1], c[0], -self.frame.bitangent()),
            (1, _) => ((u1, v1), (u1, v0), c[2], c[1], self.frame.tangent()),
            _ => ((u0, v0), (u0, v1), c[0], c[3], -self.frame.tangent()),
        };
        let top = floor.min(collar_h);
        let (low0, low1) = (c0.min(self.vault.ceiling_height_at(a0, b0)).min(top), c1.min(self.vault.ceiling_height_at(a1, b1)).min(top));
        let (high0, high1) = (c0.max(top), c1.max(top));
        let (p0, p1) = (self.frame.point_at(a0, b0, low0), self.frame.point_at(a1, b1, low1));
        let (p2, p3) = (self.frame.point_at(a1, b1, high1), self.frame.point_at(a0, b0, high0));
        target.push_quad(Quad::glowing(p0, p1, p2, p3, normal, self.rock_tone((p0 + p2) * 0.5), 0.5));
    }
}
