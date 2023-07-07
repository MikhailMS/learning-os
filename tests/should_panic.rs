#![no_std]
#![no_main]

use core::panic::PanicInfo;
use radius_os::{ qemu_codes, serial_println };

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]!");
    qemu_codes::exit_qemu(qemu_codes::QemuExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    qemu_codes::exit_qemu(qemu_codes::QemuExitCode::Failure);
    
    loop {}
}

fn should_fail() {
    serial_println!("should_panic::should_fail... \t");
    assert_eq!(0, 1);
}
