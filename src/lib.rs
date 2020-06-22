#![no_std]

// #![feature(asm)]
#![allow(dead_code)]

use core::panic::PanicInfo;
use core::fmt::{self, Write};
use core::iter::Iterator;

mod serial;
mod x86;

const VGA: *mut u16 = 0xFFFF_FFFF_800B_8000 as *mut u16;

struct FmtBuffer<'a> {
    x: &'a mut [u8],
    cursor: usize,
}

impl<'a> FmtBuffer<'a> {
    fn new(buf: &'a mut [u8]) -> FmtBuffer<'a> {
        FmtBuffer { x: buf, cursor: 0 }
    }

    fn len(&self) -> usize {
        self.cursor
    }

   fn reset(&mut self) {
       self.cursor = 0;
    }
}

impl fmt::Write for FmtBuffer<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (i, c) in s.chars().enumerate() {
            self.x[self.cursor + i] = c as u8;
        }
        self.cursor += s.len();
        Ok(())
    }
}


struct VgaScreen {
    x: usize,
    y: usize,
}

impl VgaScreen {
    fn new() -> VgaScreen {
        VgaScreen { x: 0, y: 0 }
    }

    fn vga_char(c: char) -> u16 {
        c as u16 | 0x0c << 8
    }

    fn raw_set(&mut self, x: usize, y: usize, c: u16) {
        let offset = (y*80 + x) as isize;
        unsafe { *VGA.offset(offset) = c };
    }

    fn set(&mut self, c: u16) {
        self.raw_set(self.x, self.y, c)
    }

    fn step(&mut self) {
        self.x += 1;
        if self.x == 80 {
            self.x = 0;
            self.y += 1;
        }
        if self.y == 25 {
            // TODO scroll
            self.y = 0;
        }
    }

    fn clear(&mut self) {
        let background = Self::vga_char(' ');
        for x in 0..80 {
            for y in 0..25 {
                self.raw_set(x, y, background);
            }
        }
    }
}

impl fmt::Write for VgaScreen {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            match c {
                '\n' => {
                    self.x = 0;
                    self.y += 1;
                },
                '\t' => {
                    self.x += 7;
                    self.x &= !7;
                },
                x @ _ => {
                    self.set(Self::vga_char(x));
                    self.step();
                }
            }
        }
        Ok(())
    }
}


#[no_mangle]
pub extern "C" fn kernel_main() -> ! {

    let mut v = VgaScreen::new();
    v.clear();
    write!(v, "Hello World from\n").unwrap();
    write!(v, "The Cardinal Operating System\n").unwrap();
    write!(v, "123\t12\t1\t\n").unwrap();

    let mut buf: [u8; 128] = [0; 128];

    let mut b = FmtBuffer::new(&mut buf);
    write!(b, "Cardinal OS").unwrap();
    b.reset();
    write!(b, "Hello World {}", 1234).unwrap();

    let mut s = serial::SerialPort::new(0x3f8);
    write!(s, "Hello World from the Cardinal Operating System\r\n").unwrap();
    write!(s, "Let's test some formatting {}\r\n", 1234).unwrap();

    x86::idt_init();
    x86::pic_init();
    x86::unmask_irq(4);
    unsafe { x86::enable_irqs(); }

    loop {}
}


#[no_mangle]
pub extern "C" fn c_interrupt_shim(frame: *mut x86::InterruptFrame) {
    let mut serial = unsafe { serial::SerialPort::new_raw(0x3f8) };

    let interrupt = unsafe { (*frame).interrupt_number };

    write!(serial, "interrupt {}\r\n", interrupt).unwrap();

    // let f = unsafe { &*frame };
    // write!(serial, "{:?}\r\n", f).unwrap();

    if interrupt == 36 {
        let c = serial.read_byte();
        write!(serial, "serial read: '{}'\r\n", c as char).unwrap();
    }

    if interrupt >= 32 && interrupt < 48 {
        x86::send_eoi(interrupt - 32);
    }
}


#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    let mut serial = unsafe { serial::SerialPort::new_raw(0x3f8) };

    write!(serial, "{}", panic_info).unwrap();

    loop {}
}
