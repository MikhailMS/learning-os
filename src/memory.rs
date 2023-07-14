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

/// Creates an example mapping for a given page to frame `0xb8000`
pub fn create_example_mapping(
    page:            Page,
    mapper:          &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    // FIXME: This is unsafe: only for illustration purpose
    //
    // map_to() is unsafe because caller must ensure that the frame is not already in use
    // if frame is mapped multiple times, it leads to UB
    let map_to_result = unsafe { mapper.map_to(page, frame, flags, frame_allocator) };

    map_to_result.expect("map_to failed").flush();
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
