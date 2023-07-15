use bootloader::bootinfo::{
    MemoryMap,
    MemoryRegionType
};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator,
        Mapper,
        OffsetPageTable,
        Page,
        PageTable,
        PageTableFlags,
        PhysFrame,
        Size4KiB,
        page_table::FrameError
    },
    PhysAddr,
    VirtAddr
};

/// FrameAllocator that returns usable frames from the bootloader's memory map
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next:       usize
}

impl BootInfoFrameAllocator {
    /// Create FrameAllocator from the passed memory map
    ///
    /// This function is unsafe because the caller must guarantee that passed
    /// memory map is valid. The main requirement is that all frames that marked
    /// as `USABLE` in it are really unused
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // Get usable regions from memory map
        let regions = self.memory_map.iter(); // Iterator<Item = MemoryRegion>
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);

        // Map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr()); // Iterator<Item = Iterator<Item = u64>>, iterator over ranges [start, end]

        // Transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096)); // Iterator<Item = u64>

        // Create `PhysFrame` types from the start address
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);

        self.next += 1;
        frame
    }
}

/// Initialises a new OffsetPageTable
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped onto virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is UB)
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
  let level_4_table = active_level_4_table(physical_memory_offset);

  OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference (pointer) to the active Level 4 Page Table
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped onto virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is UB)
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    // Read the active level 4 Page Table from C3 register
    let (level_4_page_table, _) = Cr3::read();

    let phys = level_4_page_table.start_address(); 
    let virt = physical_memory_offset + phys.as_u64();

    let page_table_prt: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_prt // unsafe
}


pub struct EmptyFrameAllocator;

// unsafe because implementer must ensure that allocator only ever yields unused frames
unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}
