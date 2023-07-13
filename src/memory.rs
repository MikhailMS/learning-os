use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        PageTable,
        page_table::FrameError
    },
    PhysAddr,
    VirtAddr
};

/// Returns a mutable reference (pointer) to the active Level 4 Page Table
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped onto virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is UB)
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static PageTable {
    // Read the active level 4 Page Table from C3 register
    let (level_4_page_table, _) = Cr3::read();

    let phys = level_4_page_table.start_address(); 
    let virt = physical_memory_offset + phys.as_u64();

    let page_table_prt: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_prt // unsafe
}

/// Translates given virtual address to the mapped physical address,
/// or `None` if the address is not mapped
///
/// This function is is unsafe because the caller must guarantee that the
/// complete physical memory is mapped onto virtual memory at the passed
/// `physical_memory_offset`
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

/// Private function that is called by `translate_addr`
///
/// This function is safe to limit the scope of `unsafe` because Rust
/// treats the whole body of unsafe functions as an unsafe block
/// This function must only be reachable through `unsafe fn` from outside of this module
fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
    // Read the active level 4 Page Table from C3 register
    let (level_4_page_frame, _) = Cr3::read();
     
    let table_indices = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_page_frame;

    // traverse the multi-level page table
    for &index in &table_indices {
        // convert frame into a Page Table reference
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr: *const PageTable = virt.as_ptr();
        let table = unsafe { &*table_ptr };

        // read the Page Table entry and update frame
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame)                        => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame)       => panic!("huge pages are not supported")
        };
    }

    // calculate physical address by adding the page offset
    Some(frame.start_address() + u64::from(addr.page_offset()))
}
