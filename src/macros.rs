#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    ()            => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial_uart::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    ()                       => ($crate::serial_print!("\n"));
    ($fmt:expr)              => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr,$($arg:tt)* ) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}

