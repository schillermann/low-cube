use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

pub struct HeapLedger;

thread_local! {
    static TRACKING_ACTIVE: Cell<bool> = const { Cell::new(false) };
    static ALLOCATION_COUNT: Cell<usize> = const { Cell::new(0) };
}

impl HeapLedger {
    /// Primary constructor doing nothing except returning the stateless allocator ledger.
    pub const fn new() -> Self {
        Self
    }

    pub fn measure<F: FnOnce()>(&self, action: F) -> usize {
        ALLOCATION_COUNT.with(|c| c.set(0));
        TRACKING_ACTIVE.with(|a| a.set(true));
        action();
        TRACKING_ACTIVE.with(|a| a.set(false));
        ALLOCATION_COUNT.with(|c| c.get())
    }
}

unsafe impl GlobalAlloc for HeapLedger {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        TRACKING_ACTIVE.with(|active| {
            if active.get() {
                ALLOCATION_COUNT.with(|count| count.set(count.get() + 1));
            }
        });
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}
