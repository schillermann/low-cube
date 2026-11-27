/// Strict `< 250 MB` total VRAM ceiling targeted for `2 GB` GeForce MX150 & shared-RAM Intel UHD 620 laptops.
pub const THINKPAD_VRAM_BUDGET_BYTES: u64 = 250 * 1024 * 1024;

/// Focused contract accounting GPU buffer and texture allocations without exposing raw handles.
pub trait VramAccount {
    fn credit_allocation(&mut self, bytes: u64);
    fn debit_release(&mut self, bytes: u64);
    fn live_bytes(&self) -> u64;
    fn peak_bytes(&self) -> u64;
    fn within_budget(&self) -> bool;
}

/// Live VRAM ledger tracking active and peak GPU memory usage against `budget_bytes` (`3` private fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VramLedger {
    allocated_bytes: u64,
    peak_bytes: u64,
    budget_bytes: u64,
}

impl VramLedger {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(allocated_bytes: u64, peak_bytes: u64, budget_bytes: u64) -> Self {
        Self { allocated_bytes, peak_bytes, budget_bytes }
    }

    pub const fn thinkpad_250mb() -> Self {
        Self::new(0, 0, THINKPAD_VRAM_BUDGET_BYTES)
    }

    pub const fn budget_bytes(self) -> u64 {
        self.budget_bytes
    }
}

impl VramAccount for VramLedger {
    fn credit_allocation(&mut self, bytes: u64) {
        self.allocated_bytes = self.allocated_bytes.saturating_add(bytes);
        if self.allocated_bytes > self.peak_bytes {
            self.peak_bytes = self.allocated_bytes;
        }
    }

    fn debit_release(&mut self, bytes: u64) {
        self.allocated_bytes = self.allocated_bytes.saturating_sub(bytes);
    }

    fn live_bytes(&self) -> u64 {
        self.allocated_bytes
    }

    fn peak_bytes(&self) -> u64 {
        self.peak_bytes
    }

    fn within_budget(&self) -> bool {
        self.allocated_bytes < self.budget_bytes
    }
}

/// Fail-Fast Yegor Decorator over any `VramAccount` panicking immediately if live VRAM exceeds its budget (`1` field).
#[derive(Clone, Copy, Debug)]
pub struct StrictVramGuard<A> {
    origin: A,
}

impl<A> StrictVramGuard<A> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: A) -> Self {
        Self { origin }
    }
}

impl<A: VramAccount> VramAccount for StrictVramGuard<A> {
    fn credit_allocation(&mut self, bytes: u64) {
        self.origin.credit_allocation(bytes);
        assert!(
            self.origin.within_budget(),
            "VRAM budget exceeded on ThinkPad profile: live={} bytes",
            self.origin.live_bytes()
        );
    }

    fn debit_release(&mut self, bytes: u64) {
        self.origin.debit_release(bytes);
    }

    fn live_bytes(&self) -> u64 {
        self.origin.live_bytes()
    }

    fn peak_bytes(&self) -> u64 {
        self.origin.peak_bytes()
    }

    fn within_budget(&self) -> bool {
        self.origin.within_budget()
    }
}
