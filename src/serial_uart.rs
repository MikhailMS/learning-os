use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) }; // I/O mapped port of UART device - 0x3f8; looks like it is a standard port for serial interface (although UART utilises multiple ports, here it is enough to only specify one, then SerialPort would figure out the rest)

        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Only to be used by macros!
    use core::fmt::Write;
    // SerialPort already implements fmt::Write, so we don't need to do it here like wee did for
    // custom VGA buffer
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}
