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

    pub fn pop_next(&mut self) -> Option<usize> {
        if self.mask == 0 {
            return None;
        }
        let idx = self.mask.trailing_zeros() as usize;
        self.mask &= !(1u32 << idx);
        Some(idx)
    }
}
