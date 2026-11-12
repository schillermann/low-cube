use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::{ElevatedShape, NICHE_CONTACT_TOLERANCE};

const NICHE_MIN_GAP: f32 = 0.45;
const NICHE_WALL_MARGIN: f32 = 0.10;
const NICHE_HEAD_CLEARANCE: f32 = 0.15;
const SUB_METER_NICHE_SPAN: f32 = 0.75;

pub(super) struct FlankMeshTarget<'a> {
    inner: &'a mut dyn MeshTarget,
    up: Vec3,
}

impl<'a> FlankMeshTarget<'a> {
    pub(super) fn new(inner: &'a mut dyn MeshTarget, up: Vec3) -> Self {
        Self { inner, up }
    }
}

impl MeshTarget for FlankMeshTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let horizontal = quad.transfer(|_, _, _, _, norm, _| norm.dot(self.up).abs() > 0.9);
        if !horizontal {
            self.inner.push_quad(quad);
        }
    }
}

pub(super) struct LoosenedPartTarget<'a> {
    inner: &'a mut dyn MeshTarget,
    pivot: Vec3,
    stage: u8,
}

impl<'a> LoosenedPartTarget<'a> {
    pub(super) fn new(inner: &'a mut dyn MeshTarget, pivot: Vec3, stage: u8) -> Self {
        Self { inner, pivot, stage }
    }
}

impl MeshTarget for LoosenedPartTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let s = self.stage.min(3) as f32;
        let (scale, shade, glow) = (1.0 - s * 0.14, 1.0 - s * 0.16, quad.radiate(|g| g));
        let p = self.pivot;
        let contracted = quad.transfer(|v0, v1, v2, v3, norm, col| {
            Quad::glowing(
                p + (v0 - p) * scale,
                p + (v1 - p) * scale,
                p + (v2 - p) * scale,
                p + (v3 - p) * scale,
                norm,
                (col * shade).clamp(Vec3::ZERO, Vec3::ONE),
                glow,
            )
        });
        self.inner.push_quad(contracted);
    }
}

/// Vertical column slice of elevated blocks evaluating sub-meter niche cavities.
pub struct NicheColumn<'a> {
    blocks: &'a [ElevatedShape],
    ground_h: f32,
}

impl<'a> NicheColumn<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(blocks: &'a [ElevatedShape], ground_h: f32) -> Self {
        Self { blocks, ground_h }
    }

    pub fn confine(&self, elev: f32) -> Option<f32> {
        let (mut idx, mut floor) = (0, self.ground_h);
        while idx < self.blocks.len() {
            let base = self.blocks[idx].floor();
            if base <= floor + NICHE_CONTACT_TOLERANCE {
                floor = floor.max(self.blocks[idx].ceiling());
                idx += 1;
            } else if base >= floor + NICHE_MIN_GAP {
                let (cav_ceil, mut roof_top) = (base, self.blocks[idx].ceiling());
                idx += 1;
                while idx < self.blocks.len() && self.blocks[idx].floor() <= roof_top + NICHE_CONTACT_TOLERANCE {
                    roof_top = roof_top.max(self.blocks[idx].ceiling());
                    idx += 1;
                }
                let limit = (cav_ceil + SUB_METER_NICHE_SPAN).max(roof_top - NICHE_WALL_MARGIN);
                if elev >= floor - NICHE_WALL_MARGIN && elev <= limit {
                    let min_z = floor + NICHE_HEAD_CLEARANCE;
                    return Some(elev.clamp(min_z, (cav_ceil - NICHE_HEAD_CLEARANCE).max(min_z)));
                }
                floor = roof_top;
            } else {
                floor = floor.max(self.blocks[idx].ceiling());
                idx += 1;
            }
        }
        None
    }
}
