use super::{ align_up, LockedAllocator };

use alloc::alloc::{ GlobalAlloc, Layout };
use core::{
    ptr::null_mut,
    mem
};

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>
}

impl ListNode {
    /// Creates List Node entity
    const fn new(size: usize) -> Self {
        ListNode {
            size,
            next: None
        }
    }

    fn start_addr(&self) -> usize {
        // Immutable reference (pointer)
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode
}

impl LinkedListAllocator {
    /// Create empty Linked List Allocator
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0)
        }
    }

    /// # Safety
    /// Initialise the allocator with given heap boundaries
    ///
    /// This function is unsafe because the caller must guarantee that given
    /// heap bounds are valid and that heap is unused.
    /// This method to be called only once
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    /// Adds given memory region to the front of the list
    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr); // Making sure that addr is aligned, otherwise fail
        assert!(size >= mem::size_of::<ListNode>()); // Making sure that freed region has capacity to hold ListNode

        // Create new node
        let mut node = ListNode::new(size);
        node.next = self.head.next.take();

        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);

        // Append new node to the start of the list
        self.head.next = Some(&mut *node_ptr);
    }

    /// Looks for a free region with given size & alignment and removes it from the list
    ///
    /// Returns a tuple of (list node, start address of the allocation)
    fn find_region(&mut self, size: usize, align: usize) -> Option<(usize, usize, usize)> {
        // Reference to current list node, updated upon each WHILE iteration
        let mut current_node = &mut self.head;

        // Look for a large enogh memory region in linked list
        while let Some(ref mut region) = current_node.next {
            if let Ok((alloc_start, alloc_end, excess_size)) = Self::alloc_from_region(&region, size, align) {
                // region is suitable for allocation -> remove node from list
                let next = region.next.take();
                let ret  = Some((alloc_start, alloc_end, excess_size));

                current_node.next = next;
                return ret;
            } else {
                // region is not suitable -> continue to look into next regions
                current_node = current_node.next.as_mut().unwrap();
            }
        }
        // no suitable region found
        None
    }

    /// Try to use the given region for an allocation with given size and alignemnt
    ///
    /// Returns an allocation start address on success
    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<(usize, usize, usize), ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end   = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            // Region too small
            return Err(());
        }

        let excess_size = region.end_addr() - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            // rest of region too small to hold ListNode
            // (required because allocation splits the region in a used and free part)
            return Err(());
        }

        Ok((alloc_start, alloc_end, excess_size))
    }


    ///
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();

        let size = layout.size().max(mem::size_of::<ListNode>());

        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for LockedAllocator<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((alloc_start, alloc_end, excess_size)) = allocator.find_region(size, align) {
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);
        self.lock().add_free_region(ptr as usize, size);
    }
}

