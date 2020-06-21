#![no_std]

use core::panic::PanicInfo;
use core::fmt::{self, Write};
use core::iter::Iterator;

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


fn vga_char(c: char) -> u16 {
    c as u16 | 0x0c << 8
}

fn clear_screen() {
    for x in 0..79 {
        for y in 0..24 {
            unsafe { *VGA.offset(y*80 + x) = vga_char(' ') };
        }
    }
}


fn print(line: isize, buf: &[u8]) {
    let offset = line * 80;
    for (i, c) in buf.iter().enumerate() {
        let vga_item = vga_char(*c as char);
        unsafe { *VGA.offset(offset + i as isize) = vga_item };
    }
}


#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    clear_screen();

    let mut buf: [u8; 128] = [0; 128];
    let mut b = FmtBuffer::new(&mut buf);

    write!(b, "Hello World {}", 1234).unwrap();
    let l = b.len();

    print(3, &buf[..l]);

    loop {}
}




#[no_mangle]
pub extern "C" fn c_interrupt_shim() {}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
