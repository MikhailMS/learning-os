#![no_std]  // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

// Below flags are to enable tests
#![feature(custom_test_frameworks)]
#![test_runner(radius_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{ BootInfo, entry_point };
use core::panic::PanicInfo;
use radius_os::{ println, vga, hlt_loop, memory };
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

    let phys_memory_offset  = VirtAddr::new(boot_info.physical_memory_offset);
    let mut frame_allocator = memory::EmptyFrameAllocator;
    // Initialise mapper (virt to phys)
    let mut mapper = unsafe { memory::init(phys_memory_offset) };

    let page = Page::containing_address(VirtAddr::new(0));

    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();

    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e); }

    // let addresses = [
    //     // the identity-mapped vga buffer page
    //     0xb8000,
    //     // some code page
    //     0x201008,
    //     // some stack page
    //     0x0100_0020_1a10,
    //     // virtual address mapped to physical address 0
    //     boot_info.physical_memory_offset,
    // ];

    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);
    //     let phys = mapper.translate_addr(virt);

    //     println!("{:?} -> {:?}", virt, phys);
    // }

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
