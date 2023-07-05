#![no_std]  // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

// Below flags are to enable tests
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod macros;
mod qemu_codes;
mod vga;

use core::panic::PanicInfo;

/// This function is called on panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic at the disco *dance*: {}", info);
    loop {}
}

#[no_mangle] // don't mangle (change) the name of this function
pub extern "C" fn _start() -> ! {

    vga::WRITER.lock().write_string("Hello there!");
    vga::WRITER.lock().write_byte(b'H');
    vga::WRITER.lock().write_byte_at(b'L', 10, 40);
    
    println!("It works!");

    #[cfg(test)]
    test_main();

    loop {}
}


#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());

    for test in tests {
        test();
    }

    qemu_codes::exit_qemu(qemu_codes::QemuExitCode::Success);
}

#[test_case]
fn trivial_assert() {
    print!("trivial assertion... ");
    assert_eq!(1, 1);
    println!("[ok]");
}
