use core::fmt;

use crate::x86::{inb, outb};

pub struct SerialPort {
    port: u16,
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
    pub const fn new(port: u16) -> Self {
        SerialPort { port }
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

    // For the case where the port is already initialized
    pub unsafe fn new_raw(port: u16) -> Self {
        SerialPort { port }
    }

    fn status(&self) -> u8 {
        unsafe { inb(self.port + UART_LINE_STATUS) }
    }

    fn data_available(&self) -> bool {
        self.status() & 0x01 == 0
    }

    pub fn read_byte(&mut self) -> u8 {
        unsafe { inb(self.port + UART_DATA) }
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
