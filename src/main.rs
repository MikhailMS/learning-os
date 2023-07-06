#![no_std]  // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

// Below flags are to enable tests
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod macros;
mod qemu_codes;
mod serial_uart;
mod vga;

use core::panic::PanicInfo;

/// This panic only for dev & release builds
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic at the disco *dance*: {}", info);
    loop {}
}

/// This panic only for test builds
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Panic at the disco *dance*: {}", info);
    qemu_codes::exit_qemu(qemu_codes::QemuExitCode::Failure);
    loop {}
}


pub trait Testable {
    fn run(&self) -> ();
}

impl <T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
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
fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());

    for test in tests {
        test.run();
    }

    qemu_codes::exit_qemu(qemu_codes::QemuExitCode::Success);
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

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = vga::WRITER.lock().buffer.chars[vga::BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_char), c);
    }
}
