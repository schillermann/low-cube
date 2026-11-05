mod mesh;

use glam::Vec3;
use crate::block::{GridCell, InvertedShape};
use super::magma_core::MagmaCore;

/// Spherical inner shell of the 4-layer planetary crust enclosing the hollow world with beveled ceiling blocks.
#[derive(Clone, Copy, Debug)]
pub struct CrustVault {
    radius: f32,
    half_blocks: i32,
}

impl CrustVault {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(radius: f32, half_blocks: i32) -> Self {
        Self { radius, half_blocks }
    }

    pub fn beneath(half_blocks: i32) -> Self {
        let r = if half_blocks <= 16 {
            4.0
        } else if half_blocks < 128 {
            (half_blocks as f32) * 0.375
        } else {
            (half_blocks as f32) * 0.421875
        };
        Self::new(r, half_blocks)
    }

    pub fn circumference(&self) -> f32 {
        std::f32::consts::TAU * self.radius
    }

    pub(super) const fn radius(&self) -> f32 {
        self.radius
    }

    pub fn core(&self) -> MagmaCore {
        let drop = if self.half_blocks >= 128 { (self.half_blocks as f32) * (14.0 / 128.0) } else { 8.0 };
        MagmaCore::new((self.radius - drop).max((self.radius - 2.0) * 0.5).max(1.0), self.half_blocks, (self.half_blocks / 16).max(1))
    }

    fn half_size(&self) -> f32 {
        (self.half_blocks as f32) * 0.5
    }

    pub fn ceiling_inset(&self, margin: f32) -> f32 {
        self.half_size() - self.radius + margin
    }

    /// Face-local point on the vault vertically beneath (u, v), clamped onto this face's pyramid sector seam.
    pub(super) fn shell_point(&self, u: f32, v: f32) -> (f32, f32, f32) {
        let reach = u * u + v * v + u.abs().max(v.abs()).powi(2);
        let scale = if reach > self.radius * self.radius { self.radius / reach.sqrt() } else { 1.0 };
        let (su, sv) = (u * scale, v * scale);
        (su, sv, (self.radius * self.radius - su * su - sv * sv).max(0.0).sqrt() - self.half_size())
    }

    pub(super) fn opens_at(&self, u: f32, v: f32) -> bool {
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

    pub fn crust_top_step_at(&self, u: i32, v: i32) -> i32 {
        self.floor_steps_at(u, v) + 4
    }

    pub fn crust_layer_at(&self, u: i32, v: i32, layer: i32) -> bool {
        layer < self.crust_top_step_at(u, v)
    }

    pub fn crust_surface_at(&self, u: i32, v: i32, top_step: i32) -> bool {
        self.crust_layer_at(u, v, top_step - 1)
    }

    /// Indicates whether a point at tangential (u, v) and face elevation lies inside the vault by at least `margin`.
    pub fn encloses(&self, u: f32, v: f32, elev: f32, margin: f32) -> bool {
        Vec3::new(u, v, self.half_size() + elev).length() < self.radius - margin
    }

    /// Pushes an eye elevation that rose into the vault ceiling back beneath it so the crust stays opaque.
    pub fn confine(&self, u: f32, v: f32, elev: f32) -> f32 {
        if elev >= -0.5 {
            return elev;
        }
        let (su, sv, shell_h) = self.shell_point(u, v);
        let inner_r = (self.radius - 0.30).max(1.0);
        let inner_h = (inner_r * inner_r - su * su - sv * sv).max(0.0).sqrt() - self.half_size();
        let (cu, cv) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32);
        let c = self.corners_at(cu, cv);
        let low_block = c[0].min(c[1]).min(c[2]).min(c[3]) - 0.25;
        let ceiling = inner_h.min(low_block).min(self.ceiling_height_at(u, v) - 0.30).min(shell_h - 0.30);
        if elev < ceiling + 1.2 && elev > ceiling { ceiling } else { elev }
    }

    pub(super) fn block_at(&self, cell: GridCell) -> (f32, InvertedShape) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let q = [
            (self.shell_point(u0, v0).2 * 4.0).round() as i32,
            (self.shell_point(u1, v0).2 * 4.0).round() as i32,
            (self.shell_point(u1, v1).2 * 4.0).round() as i32,
            (self.shell_point(u0, v1).2 * 4.0).round() as i32,
        ];
        let (q_max, q_min) = (q[0].max(q[1]).max(q[2]).max(q[3]), q[0].min(q[1]).min(q[2]).min(q[3]));
        let (floor_elev, drops) = if q_max == q_min {
            if q_max.rem_euclid(2) == 0 {
                ((q_max as f32) * 0.25, [2, 2, 2, 2])
            } else {
                (((q_max - 1) as f32) * 0.25, [1, 1, 1, 1])
            }
        } else {
            let top_q = if q_max.rem_euclid(2) != 0 && q_max - q_min == 1 { q_max + 1 } else { q_max };
            ((top_q as f32) * 0.25 - 0.5, q.map(|qi| (top_q - qi).clamp(0, 2)))
        };
        (floor_elev, InvertedShape::from_corner_drops(drops[0], drops[1], drops[2], drops[3]))
    }

    pub(super) fn corners_at(&self, u: i32, v: i32) -> [f32; 4] {
        let ((u0, u1), (v0, v1)) = (GridCell::new(u, v).u_bounds(), GridCell::new(u, v).v_bounds());
        let (floor_elev, block) = self.block_at(GridCell::new(u, v));
        let c = block.ceiling_corners(floor_elev);
        [
            if self.opens_at(u0, v0) { c[0] } else { self.shell_point(u0, v0).2 },
            if self.opens_at(u1, v0) { c[1] } else { self.shell_point(u1, v0).2 },
            if self.opens_at(u1, v1) { c[2] } else { self.shell_point(u1, v1).2 },
            if self.opens_at(u0, v1) { c[3] } else { self.shell_point(u0, v1).2 },
        ]
    }
}
