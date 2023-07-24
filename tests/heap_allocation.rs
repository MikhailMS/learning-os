#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(radius_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{
    boxed::Box,
    vec::Vec,
};
use bootloader::{ entry_point, BootInfo };
use core::panic::PanicInfo;
use radius_os::{
    memory::{ self, BootInfoFrameAllocator },
    allocator,
    init,
    test_panic_handler,
};
use x86_64::VirtAddr;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {

    init();

    // Initialise Heap space >>>>>>>
    let phys_memory_offset  = VirtAddr::new(boot_info.physical_memory_offset);
    // Initialise mapper (virt to phys)
    let mut mapper          = unsafe { memory::init(phys_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialisation failed");
    // <<<<<<<<

    test_main();
    loop {}
}

#[test_case]
fn simple_allocation() {
    let heap_1 = Box::new(41);
    let heap_2 = Box::new(11);
    let heap_3 = Box::new(31);
    let heap_4 = Box::new(21);

    assert_eq!(*heap_1, 41);
    assert_eq!(*heap_2, 11);
    assert_eq!(*heap_3, 31);
    assert_eq!(*heap_4, 21);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();

    for i in 0..n {
        vec.push(i);
    }

    assert_eq!(vec.iter().sum::<u64>(), (n-1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..allocator::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn many_boxes_long_lived() {
    let long = Box::new(10000);

    for i in 0..allocator::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long, 10000);
}
