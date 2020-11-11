use core::fmt::{self, Write};

use crate::serial::SerialPort;

pub fn print(args: fmt::Arguments) {
    SerialPort::new(0x3f8).write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! dprint {
    ($($arg:tt)*) => {
        $crate::debug::print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! dprintln {
    () => ($crate::dprint!("\r\n"));
    ($fmt:expr) => ($crate::dprint!(concat!($fmt, "\r\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::dprint!(concat!($fmt, "\r\n"), $($arg)*));
}
