pub mod bump_allocator;
pub mod linked_allocator;

#[cfg(all(feature = "bump-allocator"))]
use bump_allocator::BumpAllocator as Allocator;
#[cfg(all(feature = "linked-allocator"))]
use linked_allocator::LinkedListAllocator as Allocator;

use alloc::alloc::Layout;
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
static ALLOCATOR: LockedAllocator<Allocator> = LockedAllocator::new(Allocator::new());

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE:  usize = 100 * 1024; // 100KiB

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


/// Wrapper for a custom Allocator to allow trait implementation on
/// Mutex<BumpAllocator/LinkedListAllocator>
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

