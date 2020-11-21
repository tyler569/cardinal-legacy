use core::fmt::{self, Write};
use alloc::collections::VecDeque;
use crate::sync::Mutex;
use crate::x86::{inb, outb};

pub struct SerialPort {
    port: u16,
    buffer: VecDeque<u8>,
}

const UART_DATA: u16 = 0;
const UART_INTERRUPT: u16 = 1;
const UART_BAUD_LOW: u16 = 0;
const UART_BAUD_HIGH: u16 = 1;
const UART_FIFO_CTRL: u16 = 2;
const UART_LINE_CTRL: u16 = 3;
const UART_MODEM_CTRL: u16 = 4;
const UART_LINE_STATUS: u16 = 5;
const UART_MODEM_STATUS: u16 = 6;

impl SerialPort {
    pub fn new(port: u16) -> Self {
        SerialPort {
            port,
            buffer: VecDeque::new(),
        }
    }

    pub fn init(&mut self) {
        unsafe {
            outb(self.port + UART_BAUD_HIGH, 0x00);
            outb(self.port + UART_LINE_CTRL, 0x80);
            outb(self.port + UART_BAUD_LOW, 0x03);
            outb(self.port + UART_BAUD_HIGH, 0x00);
            outb(self.port + UART_LINE_CTRL, 0x03);
            outb(self.port + UART_FIFO_CTRL, 0xC7);
            outb(self.port + UART_MODEM_CTRL, 0x0B);

            outb(self.port + UART_INTERRUPT, 0x09);
        }
    }

    fn status(&self) -> u8 {
        unsafe { inb(self.port + UART_LINE_STATUS) }
    }

    fn data_available(&self) -> bool {
        self.status() & 0x01 == 0
    }

    pub unsafe fn handle_irq(&mut self) {
        let byte = inb(self.port + UART_DATA);
        self.buffer.push_back(byte);
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            unsafe { outb(self.port, c as u8) };
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref GLOBAL_SERIAL: Mutex<SerialPort> = {
        let mut serial = SerialPort::new(0x3f8);
        serial.init();
        Mutex::new(serial)
    };
}

pub fn serial_print(args: fmt::Arguments) {
    GLOBAL_SERIAL.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::serial::serial_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\r\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\r\n"), $($arg)*));
}
