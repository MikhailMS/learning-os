use alloc::alloc::{ GlobalAlloc, Layout };
use core::ptr::null_mut;
use linked_list_allocator::LockedHeap;
use spin::{ Mutex, MutexGuard };
use x86_64::{
    structures::paging::{
        mapper::MapToError,
        FrameAllocator,
        Mapper,
        Page,
        PageTableFlags,
        Size4KiB
    },
    VirtAddr
};

#[global_allocator]
static ALLOCATOR: LockedAllocator<BumpAllocator> = LockedAllocator::new(BumpAllocator::new());
// static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100KiB

pub fn init_heap(
    mapper:          &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end   = heap_start + HEAP_SIZE - 1u64;

        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page   = Page::containing_address(heap_end);

        Page::range_inclusive(heap_start_page, heap_end_page) // returns PageRangeInclusive<S>
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE)
    }

    Ok(())
}

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

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end   = heap_start + heap_size;
        self.next       = heap_start;
    }
}

/// Wrapper for BumpAllocator to allow trait implementation on Mutex<BumpAllocator>
pub struct LockedAllocator<A> {
    alloc: Mutex<A>,
}

impl<A> LockedAllocator<A> {
    pub const fn new(alloc: A) -> Self {
        LockedAllocator {
            alloc: Mutex::new(alloc)
        }
    }

    pub fn lock(&self) -> MutexGuard<A> {
        self.alloc.lock()
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

/// Align given address `addr` upwards to be aligned with `align`
fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr // already aligned
    } else {
        addr - remainder + align
    }
    // More optimised version would be
    // (addr + align - 1) & !(align - 1)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("memory allocation of {} bytes failed", layout.size())
}
