use core::fmt;

use crate::x86::outb;

pub struct SerialPort {
    port: u16,
}

const UART_BAUD_LOW: u16 = 0;
const UART_BAUD_HIGH: u16 = 1;
const UART_FIFO_CTRL: u16 = 2;
const UART_LINE_CTRL: u16 = 3;
const UART_MODEM_CTRL: u16 = 4;
const UART_LINE_STATUS: u16 = 5;
const UART_MODEM_STATUS: u16 = 6;

impl SerialPort {
    pub fn new(port: u16) -> Self {
        unsafe {
            outb(port + UART_BAUD_HIGH, 0x00);
            outb(port + UART_LINE_CTRL, 0x80);
            outb(port + UART_BAUD_LOW , 0x03);
            outb(port + UART_BAUD_HIGH, 0x00);
            outb(port + UART_LINE_CTRL, 0x03);
            outb(port + UART_FIFO_CTRL, 0xC7);
            outb(port + UART_MODEM_CTRL, 0x0B);
        }

        SerialPort { port }
    }

    // For the case where the port is already initialized
    pub unsafe fn new_raw(port: u16) -> Self {
        SerialPort { port }
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
