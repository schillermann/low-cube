use glam::Vec3;
use crate::{Face, MeshTarget};

/// Encapsulated bitmask and latest index of modified terrain and flora GPU mesh slices.
#[derive(Clone, Copy, Debug)]
pub struct ModifiedSlices {
    mask: u32,
    latest: usize,
}

impl ModifiedSlices {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(mask: u32, latest: usize) -> Self {
        Self { mask, latest }
    }

    /// Secondary constructor for a freshly uploaded planet without pending slice changes.
    pub const fn clean() -> Self {
        Self::new(0, 0)
    }

    pub const fn pending(self) -> bool {
        self.mask != 0
    }

    pub const fn latest(self) -> usize {
        self.latest
    }

    pub fn record(&mut self, slice_idx: usize) {
        self.latest = slice_idx;
        if slice_idx < 32 {
            self.mask |= 1u32 << slice_idx;
        }
    }

    pub fn clear_bit(&mut self, slice_idx: usize) {
        if slice_idx < 32 {
            self.mask &= !(1u32 << slice_idx);
        }
    }

    pub fn pop_next(&mut self) -> Option<usize> {
        if self.mask == 0 {
            return None;
        }
        let idx = self.mask.trailing_zeros() as usize;
        self.mask &= !(1u32 << idx);
        Some(idx)
    }

    pub fn drain_face_patch(&mut self, faces: &mut [Box<dyn Face>], target: &mut dyn MeshTarget) -> Option<(usize, usize)> {
        for (idx, face) in faces.iter_mut().enumerate() {
            if let Some(slot) = face.drain_terrain_patch(target) {
                if !face.pending_terrain_patches() {
                    self.clear_bit(idx);
                }
                return Some((idx, slot));
            }
        }
        None
    }

    pub fn horizon_mask(faces: &[Box<dyn Face>], position: Vec3, up: Vec3, half_span: f32) -> u32 {
        let len = faces.len();
        let mut mask = 1u32 << (len * 2).min(31);
        let continental = half_span > 128.0;
        let orbital = position.abs().max_element() > half_span + 96.0;
        for (idx, face) in faces.iter().enumerate() {
            let f_up = face.surface_up();
            let visible = if continental {
                f_up.dot(up) > 0.2
                    || position.dot(f_up) >= half_span - 180.0
                    || (orbital && position.dot(f_up) > -half_span * 0.25)
            } else {
                f_up.dot(up) > -0.5
            };
            if visible {
                mask |= 1u32 << idx.min(31);
                if !continental && (f_up.dot(up) > 0.2 || position.dot(f_up) >= -4.0) {
                    mask |= 1u32 << (len + idx).min(31);
                }
            }
        }
        mask
    }
}
