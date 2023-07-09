#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

// Needed to enable x86-interrupt calling convention
#![feature(abi_x86_interrupt)]

pub mod interrupts;
pub mod macros;
pub mod qemu_codes;
pub mod serial_uart;
pub mod vga;

use core::panic::PanicInfo;

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
    loop {}
}

pub fn init() {
    interrupts::init_dft();
}

// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    loop {}
}
