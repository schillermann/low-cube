pub const SECTOR_SPAN: usize = 16;

/// Per-face `16×16` sector invalidation queue and monotonic revision table for `O(1)` GPU patch updates.
pub struct DirtySectors {
    queue: Vec<usize>,
    revisions: Vec<u32>,
}

impl DirtySectors {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(queue: Vec<usize>, revisions: Vec<u32>) -> Self {
        Self { queue, revisions }
    }

    pub fn sized(divisions: u32) -> Self {
        let side = Self::sectors_per_side(divisions);
        Self::new(Vec::new(), vec![1; side * side])
    }

    pub const fn sectors_per_side(divisions: u32) -> usize {
        let d = divisions as usize;
        let s = d.div_ceil(SECTOR_SPAN);
        if s == 0 { 1 } else { s }
    }

    pub fn sector_count(&self) -> usize {
        self.revisions.len().max(1)
    }

    pub fn mark_cell_dirty(&mut self, u: i32, v: i32, half: i32, divisions: u32) {
        let side = Self::sectors_per_side(divisions);
        let su = (((u + half).max(0) as usize) / SECTOR_SPAN).min(side - 1);
        let sv = (((v + half).max(0) as usize) / SECTOR_SPAN).min(side - 1);
        let slot = sv * side + su;
        if let Some(rev) = self.revisions.get_mut(slot) {
            *rev = rev.wrapping_add(1).max(1);
        }
        if !self.queue.contains(&slot) {
            self.queue.push(slot);
        }
    }

    pub fn revision_at(&self, su: i32, sv: i32, divisions: u32) -> u32 {
        let side = Self::sectors_per_side(divisions) as i32;
        if su < 0 || su >= side || sv < 0 || sv >= side {
            return 0;
        }
        let slot = (sv as usize) * (side as usize) + (su as usize);
        self.revisions.get(slot).copied().unwrap_or(0)
    }

    pub fn pending(&self) -> bool {
        !self.queue.is_empty()
    }

    pub fn pop_next(&mut self) -> Option<usize> {
        self.queue.pop()
    }
}
