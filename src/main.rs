#![no_std]  // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

// Below flags are to enable tests
#![feature(custom_test_frameworks)]
#![test_runner(radius_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::boxed::Box;
use bootloader::{ BootInfo, entry_point };
use core::panic::PanicInfo;
use radius_os::{
    memory::{ self, BootInfoFrameAllocator },
    allocator,
    hlt_loop,
    println,
    vga,
};
use x86_64::{ structures::paging::Page, VirtAddr };

#[cfg(test)]
use radius_os::test_panic_handler;


/// This panic only for dev & release builds
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic at the disco *dance*: {}", info);
    hlt_loop();
}

/// This panic only for test builds
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    radius_os::init(); // Initialise Interrupt Descriptor Table for our kernel

    vga::WRITER.lock().write_string("Hello there!");
    vga::WRITER.lock().write_byte(b'H');
    vga::WRITER.lock().write_byte_at(b'L', 10, 40);
    
    println!("It works!");

    // x86_64::instructions::interrupts::int3(); // Invoke breakpoint exception

    // Let's cause page fault
    // let ptr = 0x2057f3 as *mut u8;
    // unsafe { *ptr = 42; }

    #[cfg(test)]
    test_main();

    hlt_loop();
}

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}
