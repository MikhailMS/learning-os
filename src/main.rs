#![no_std]  // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

// Below flags are to enable tests
#![feature(custom_test_frameworks)]
#![test_runner(radius_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{ BootInfo, entry_point };
use core::panic::PanicInfo;
use radius_os::{ println, vga, hlt_loop, memory };
use x86_64::{ structures::paging::PageTable, VirtAddr };

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

    let phys_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0; will panic with current page traverse
        // implementation
        // boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { memory::translate_addr(virt, phys_memory_offset) };

        println!("{:?} -> {:?}", virt, phys);
    }

    // let l4_table           = unsafe { memory::active_level_4_table(phys_memory_offset) };

    // for (i, entry) in l4_table.iter().enumerate() {
    //     if !entry.is_unused() {
    //         println!("L4 entry {}: {:?}", i, entry);

    //         let phys = entry.frame().unwrap().start_address();
    //         let virt = boot_info.physical_memory_offset + phys.as_u64();
    //         let ptr  = VirtAddr::new(virt).as_mut_ptr();

    //         let l3_table: &PageTable = unsafe { &*ptr };

    //         for (ii, l3_entry) in l3_table.iter().enumerate() {
    //             if !l3_entry.is_unused() {
    //                 println!("  L3 entry {}: {:?}", ii, l3_entry);
    //             }
    //         }
    //     }
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
