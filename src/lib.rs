#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

// Needed to enable x86-interrupt calling convention
#![feature(abi_x86_interrupt)]
// Needed to enable custom allocator
#![feature(alloc_error_handler)]

// Needed to ensure we can use new() fn of LinkedListAllocator
#![feature(const_mut_refs)]

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod macros;
pub mod memory;
pub mod qemu_codes;
pub mod serial_uart;
pub mod task;
pub mod vga;

extern crate alloc;

#[cfg(test)]
use bootloader::{ BootInfo, entry_point };
use core::panic::PanicInfo;
use x86_64;

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_println!("{}... \t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]!");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());

    for test in tests {
        test.run();
    }
    qemu_codes::exit_qemu(qemu_codes::QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);

    qemu_codes::exit_qemu(qemu_codes::QemuExitCode::Failure);
    hlt_loop();
}

pub fn init() {
    // Initialise Global Descriptor Table
    gdt::init();
    // Initialise Interrupt Descriptor Table
    interrupts::init_idt();
    // Initialise Programmable Interrupt Controller
    // Unsafe because can causes UB if PICS are misconfigured
    unsafe {
        interrupts::PICS.lock().initialize()
    }
    // enable() is a wrapper around ASM 'sti' instruction
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        // hlt() is a wrapper around ASM 'hlt' instruction
        // 'hlt' instructs CPU to halt until the next external interrupt is fired
        x86_64::instructions::hlt();
    }
}

// Entry point for `cargo test`
#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}
