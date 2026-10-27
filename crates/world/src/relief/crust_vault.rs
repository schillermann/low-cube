use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::{GridCell, InvertedShape, VoxelShape};
use crate::frame::SurfaceFrame;
use super::magma_core::MagmaCore;

struct VaultCeilingTarget<'a> {
    vault: &'a CrustVault,
    frame: &'a SurfaceFrame,
    tint: f32,
    target: &'a mut dyn MeshTarget,
}

impl MeshTarget for VaultCeilingTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let up = self.frame.normal();
        quad.transfer(|v0, v1, v2, v3, normal, _| {
            if normal.dot(up) >= -0.05 { return; }
            let clamp_seam = |p: Vec3| {
                let (a, b) = self.frame.decompose(p);
                let reach = a * a + b * b + a.abs().max(b.abs()).powi(2);
                if reach > self.vault.radius * self.vault.radius {
                    let (sa, sb, sh) = self.vault.shell_point(a, b);
                    self.frame.point_at(sa, sb, sh)
                } else { p }
            };
            let (c0, c1, c2, c3) = (clamp_seam(v0), clamp_seam(v1), clamp_seam(v2), clamp_seam(v3));
            let (center, glow) = ((c0 + c1 + c2 + c3) * 0.25, if normal.dot(up) < -0.05 { 0.7 } else { 0.55 });
            let col = self.vault.rock_tone(center) * self.tint;
            let quad_out = if (v2 - v3).length_squared() > 1e-6 && (c1 - c3).length_squared() < (c0 - c2).length_squared() {
                Quad::glowing(c1, c2, c3, c0, normal, col, glow)
            } else {
                Quad::glowing(c0, c1, c2, c3, normal, col, glow)
            };
            self.target.push_quad(quad_out);
        });
    }
}

/// Spherical inner shell of the 4-layer planetary crust enclosing the hollow world with beveled ceiling blocks.
#[derive(Clone, Copy, Debug)]
pub struct CrustVault {
    radius: f32,
    half_blocks: i32,
}

impl CrustVault {
    pub fn new(radius: f32, half_blocks: i32) -> Self { Self { radius, half_blocks } }
    pub fn beneath(half_blocks: i32) -> Self {
        let r = if half_blocks <= 16 { 4.0 } else if half_blocks < 128 { (half_blocks as f32) * 0.375 } else { (half_blocks as f32) * 0.421875 };
        Self::new(r, half_blocks)
    }
    pub fn circumference(&self) -> f32 { std::f32::consts::TAU * self.radius }
    pub fn core(&self) -> MagmaCore {
        let drop = if self.half_blocks >= 128 { (self.half_blocks as f32) * (14.0 / 128.0) } else { 8.0 };
        MagmaCore::new((self.radius - drop).max((self.radius - 2.0) * 0.5).max(1.0), self.half_blocks, (self.half_blocks / 16).max(1))
    }
    fn half_size(&self) -> f32 { (self.half_blocks as f32) * 0.5 }
    pub fn ceiling_inset(&self, margin: f32) -> f32 { self.half_size() - self.radius + margin }

    /// Face-local point on the vault vertically beneath (u, v), clamped onto this face's pyramid sector seam.
    fn shell_point(&self, u: f32, v: f32) -> (f32, f32, f32) {
        let reach = u * u + v * v + u.abs().max(v.abs()).powi(2);
        let scale = if reach > self.radius * self.radius { self.radius / reach.sqrt() } else { 1.0 };
        let (su, sv) = (u * scale, v * scale);
        (su, sv, (self.radius * self.radius - su * su - sv * sv).max(0.0).sqrt() - self.half_size())
    }

    fn opens_at(&self, u: f32, v: f32) -> bool {
        u * u + v * v + u.abs().max(v.abs()).powi(2) < self.radius * self.radius
    }

    pub fn cell_opens(&self, u: i32, v: i32) -> bool {
        let ((u0, u1), (v0, v1)) = (GridCell::new(u, v).u_bounds(), GridCell::new(u, v).v_bounds());
        [(u0, v0), (u1, v0), (u1, v1), (u0, v1)].iter().any(|&(a, b)| self.opens_at(a, b))
    }

    pub fn ceiling_height_at(&self, u: f32, v: f32) -> f32 {
        let h = self.shell_point(u, v).2;
        if self.opens_at(u, v) { (h * 4.0).round() * 0.25 } else { h }
    }

    /// Deepest 4-layer crust step of cell (u, v) directly above the spherical hollow world ceiling.
    pub fn floor_steps_at(&self, u: i32, v: i32) -> i32 {
        if self.cell_opens(u, v) {
            let c = self.corners_at(u, v);
            let max_c = c[0].max(c[1]).max(c[2]).max(c[3]);
            ((max_c * 2.0).ceil() as i32).min(-4)
        } else {
            let (uc, vc) = ((u as f32) * 0.5 + 0.25, (v as f32) * 0.5 + 0.25);
            ((self.ceiling_height_at(uc, vc) * 2.0).round() as i32 - 1).min(-4)
        }
    }

    /// Elevation step reached by digging one layer below `elev`; breaching the 4-layer crust opens the hollow world.
    pub fn excavated_steps(&self, u: i32, v: i32, elev: i32, shaft_steps: i32) -> i32 {
        let floor = self.floor_steps_at(u, v);
        if elev < floor {
            elev
        } else if elev > floor + 1 {
            elev - 1
        } else if self.cell_opens(u, v) {
            shaft_steps
        } else {
            floor
        }
    }

    /// Indicates whether a point at tangential (u, v) and face elevation lies inside the vault by at least `margin`.
    pub fn encloses(&self, u: f32, v: f32, elev: f32, margin: f32) -> bool {
        Vec3::new(u, v, self.half_size() + elev).length() < self.radius - margin
    }

    /// Pushes an eye elevation that rose into the vault ceiling back beneath it so the crust stays opaque.
    pub fn confine(&self, u: f32, v: f32, elev: f32) -> f32 {
        let (su, sv, shell_h) = self.shell_point(u, v);
        let inner_r = (self.radius - 0.30).max(1.0);
        let inner_h = (inner_r * inner_r - su * su - sv * sv).max(0.0).sqrt() - self.half_size();
        let (cu, cv) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32);
        let c = self.corners_at(cu, cv);
        let low_block = c[0].min(c[1]).min(c[2]).min(c[3]) - 0.25;
        let ceiling = inner_h.min(low_block).min(self.ceiling_height_at(u, v) - 0.30).min(shell_h - 0.30);
        if elev < ceiling + 1.2 && elev > ceiling { ceiling } else { elev }
    }

    fn rock_tone(&self, at: Vec3) -> Vec3 {
        let dir = at.normalize();
        let vein = ((dir.x * 11.0 + dir.z * 4.0).sin() * (dir.y * 9.0 - dir.x * 3.0).cos()) * 0.5 + 0.5;
        let grit = (((at.x * 3.1).sin() + (at.y * 2.7).cos() + (at.z * 3.7).sin()) / 6.0 + 0.5).clamp(0.0, 1.0);
        Vec3::new(0.36, 0.24, 0.16).lerp(Vec3::new(0.58, 0.33, 0.18), vein).lerp(Vec3::new(0.27, 0.20, 0.17), grit * 0.45)
    }
    fn block_at(&self, cell: GridCell) -> (f32, InvertedShape) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let q = [
            (self.shell_point(u0, v0).2 * 4.0).round() as i32, (self.shell_point(u1, v0).2 * 4.0).round() as i32,
            (self.shell_point(u1, v1).2 * 4.0).round() as i32, (self.shell_point(u0, v1).2 * 4.0).round() as i32,
        ];
        let (q_max, q_min) = (q[0].max(q[1]).max(q[2]).max(q[3]), q[0].min(q[1]).min(q[2]).min(q[3]));
        let (floor_elev, drops) = if q_max == q_min {
            if q_max.rem_euclid(2) == 0 { ((q_max as f32) * 0.25, [2, 2, 2, 2]) } else { (((q_max - 1) as f32) * 0.25, [1, 1, 1, 1]) }
        } else {
            let top_q = if q_max.rem_euclid(2) != 0 && q_max - q_min == 1 { q_max + 1 } else { q_max };
            ((top_q as f32) * 0.25 - 0.5, q.map(|qi| (top_q - qi).clamp(0, 2)))
        };
        (floor_elev, InvertedShape::from_corner_drops(drops[0], drops[1], drops[2], drops[3]))
    }
    fn corners_at(&self, u: i32, v: i32) -> [f32; 4] {
        let (floor_elev, block) = self.block_at(GridCell::new(u, v));
        block.ceiling_corners(floor_elev)
    }
    /// Emits the magma-lit inward-facing beveled block ceiling beneath this cell.
    pub fn emit_ceiling(&self, cell: GridCell, dig_stage: u8, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        if !self.cell_opens(u, v) { return; }
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (floor_elev, block) = self.block_at(cell);
        let c = block.ceiling_corners(floor_elev);
        let mut ceiling_out = VaultCeilingTarget { vault: self, frame, tint: 1.0 - (dig_stage.min(3) as f32) * 0.14, target };
        if [(u0, v0), (u1, v0), (u1, v1), (u0, v1)].iter().all(|&(a, b)| self.opens_at(a, b)) {
            block.emit_shape(cell, floor_elev, frame, Vec3::ONE, &mut ceiling_out);
        } else {
            ceiling_out.push_quad(Quad::new(frame.point_at(u0, v1, c[3]), frame.point_at(u1, v1, c[2]), frame.point_at(u1, v0, c[1]), frame.point_at(u0, v0, c[0]), -frame.normal(), Vec3::ONE));
        }
        let steps = [
            ((u0, v1), (u1, v1), c[3], c[2], self.corners_at(u, v + 1)[0], self.corners_at(u, v + 1)[1], frame.bitangent()),
            ((u1, v0), (u0, v0), c[1], c[0], self.corners_at(u, v - 1)[2], self.corners_at(u, v - 1)[3], -frame.bitangent()),
            ((u1, v0), (u1, v1), c[1], c[2], self.corners_at(u + 1, v)[0], self.corners_at(u + 1, v)[3], frame.tangent()),
            ((u0, v1), (u0, v0), c[3], c[0], self.corners_at(u - 1, v)[2], self.corners_at(u - 1, v)[1], -frame.tangent()),
        ];
        for ((a0, b0), (a1, b1), ca, cb, na, nb, side_n) in steps {
            if (self.opens_at(a0, b0) || self.opens_at(a1, b1)) && (ca < na - 1e-3 || cb < nb - 1e-3) {
                let norm = (side_n * 0.2 - frame.normal()).normalize();
                ceiling_out.push_quad(Quad::new(
                    frame.point_at(a0, b0, ca.min(na)), frame.point_at(a1, b1, cb.min(nb)),
                    frame.point_at(a1, b1, nb.max(cb)), frame.point_at(a0, b0, na.max(ca)), norm, Vec3::ONE,
                ));
            }
        }
    }

    /// Emits the crust skirt sealing the gap between this cell's 4-layer floor and the vault toward a breached neighbor.
    pub fn emit_skirt(&self, cell: GridCell, side: (i32, i32), frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let (floor, c) = ((self.floor_steps_at(u, v) as f32) * 0.5, self.corners_at(u, v));
        let ((a0, b0), (a1, b1), c0, c1, normal) = match side {
            (0, 1) => ((u0, v1), (u1, v1), c[3], c[2], frame.bitangent()),
            (0, _) => ((u1, v0), (u0, v0), c[1], c[0], -frame.bitangent()),
            (1, _) => ((u1, v1), (u1, v0), c[2], c[1], frame.tangent()),
            _ => ((u0, v0), (u0, v1), c[0], c[3], -frame.tangent()),
        };
        let (low0, low1) = (c0.min(self.ceiling_height_at(a0, b0)).min(floor), c1.min(self.ceiling_height_at(a1, b1)).min(floor));
        let (p0, p1) = (frame.point_at(a0, b0, low0), frame.point_at(a1, b1, low1));
        let (p2, p3) = (frame.point_at(a1, b1, floor), frame.point_at(a0, b0, floor));
        target.push_quad(Quad::glowing(p0, p1, p2, p3, normal, self.rock_tone((p0 + p2) * 0.5), 0.5));
    }
}
