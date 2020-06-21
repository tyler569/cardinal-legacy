#![no_std]

use core::panic::PanicInfo;

const VGA: *mut u16 = 0xFFFF_FFFF_800B_8000 as *mut u16;

fn vga_char(c: char) -> u16 {
    c as u16 | 0x04 << 8
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    for x in 0..79 {
        for y in 0..24 {
            unsafe { *VGA.offset(y*80 + x) = vga_char('R') };
        }
    }

    loop {}
}

#[no_mangle]
pub extern "C" fn c_interrupt_shim() {}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
