use super::{ align_up, LockedAllocator };

use alloc::alloc::{ GlobalAlloc, Layout };
use core::ptr::null_mut;

pub struct BumpAllocator {
    heap_start:  usize,
    heap_end:    usize,
    next:        usize,
    allocations: usize

}

impl BumpAllocator {
    /// Creates new empty Bump Allocator
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start:  0,
            heap_end:    0,
            next:        0,
            allocations: 0
        }
    }

    /// # Safety
    /// Initialise the allocator with given heap boundaries
    ///
    /// This function is unsafe because the caller must guarantee that given
    /// heap bounds are valid and that heap is unused.
    /// This method to be called only once
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end   = heap_start + heap_size;
        self.next       = heap_start;
    }
}

unsafe impl GlobalAlloc for LockedAllocator<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock(); // mutable reference for which we created wrapper :)

        let alloc_start = align_up(allocator.next, layout.align());
        let alloc_end   = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None      => return null_mut(),
        };

        if alloc_end > allocator.heap_end {
            null_mut()
        } else {
            allocator.next = alloc_end;
            allocator.allocations += 1;

            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut allocator = self.lock();

        allocator.allocations -= 1;
        if allocator.allocations == 0 {
            allocator.next = allocator.heap_start;
        }
    }
}
