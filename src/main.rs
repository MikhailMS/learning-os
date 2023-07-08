#![no_std]  // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

// Below flags are to enable tests
#![feature(custom_test_frameworks)]
#![test_runner(radius_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use radius_os::{ println, vga };

#[cfg(test)]
use radius_os::test_panic_handler;


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
    test_panic_handler(info)
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
