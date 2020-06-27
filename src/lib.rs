#![no_std]
#![feature(alloc_error_handler)]
#![feature(negative_impls)]
#![feature(ffi_returns_twice)]
// I wish I didn't have to do this -- I should see if I can figure out a
// better solution. Is it really better to delete code that isn't being used?
// What if I was previously using a data structure and pulled back from it
// temporarily? Maybe I can just annotate one data structure or something.
#![allow(dead_code)]

use core::fmt::{self, Write};

#[cfg(target_os = "none")]
use core::panic::PanicInfo;

extern crate alloc;
use alloc::boxed::Box;

#[cfg(target_os = "none")]
mod allocator;

mod thread;
mod serial;
mod sync;
mod x86;

use serial::SerialPort;
use sync::Mutex;
use x86::{long_jump, set_jump, JmpBuf};

const LOAD_OFFSET: usize = 0xFFFF_FFFF_8000_0000;
const VGA_BUFFER: *mut u16 = (LOAD_OFFSET + 0xB8000) as *mut u16;

pub struct VgaScreen {
    x: usize,
    y: usize,
}

impl VgaScreen {
    pub const fn new() -> VgaScreen {
        VgaScreen { x: 0, y: 0 }
    }

    fn vga_char(c: char) -> u16 {
        c as u16 | 0x0c << 8
    }

    fn raw_set(&mut self, x: usize, y: usize, c: u16) {
        let offset = (y * 80 + x) as isize;
        unsafe { *VGA_BUFFER.offset(offset) = c };
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
                }
                '\t' => {
                    self.x += 7;
                    self.x &= !7;
                }
                x => {
                    self.set(Self::vga_char(x));
                    self.step();
                }
            }
        }
        Ok(())
    }
}

pub static GLOBAL_VGA: Mutex<VgaScreen> = Mutex::new(VgaScreen::new());
pub static GLOBAL_SERIAL: Mutex<SerialPort> = Mutex::new(SerialPort::new(0x3f8));

pub fn serial_print(args: fmt::Arguments) {
    GLOBAL_SERIAL.lock().write_fmt(args).unwrap();
}

pub fn video_print(args: fmt::Arguments) {
    GLOBAL_VGA.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::serial_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\r\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\r\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\r\n"), $($arg)*));
}

#[macro_export]
macro_rules! vprint {
    ($($arg:tt)*) => {
        $crate::video_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! vprintln {
    () => ($crate::vprint!("\n"));
    ($fmt:expr) => ($crate::vprint!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::vprint!(concat!($fmt, "\n"), $($arg)*));
}

fn return_a_closure(x: i32) -> Box<dyn FnOnce(i32) -> i32> {
    Box::new(move |a| a + x)
}

unsafe fn jump_back(buffer: &JmpBuf) -> ! {
    long_jump(buffer, 0);
}

#[no_mangle]
pub extern "C" fn kernel_main(_multiboot_magic: u32, multiboot_info: usize) -> ! {
    GLOBAL_SERIAL.lock().init();
    GLOBAL_VGA.lock().clear();

    vprintln!("Hello World from");
    vprintln!("The Cardinal Operating System");
    vprintln!("123\t12\t1\t1234\t1");

    println!("Hello World from the Cardinal Operating System");
    println!("Let's test some formatting {}", 1234);

    let a = |x| x + 1;
    println!("Call a lambda: {}", a(10));

    let boot_info = unsafe { multiboot2::load_with_offset(multiboot_info, LOAD_OFFSET) };

    if let Some(boot_loader_name_tag) = boot_info.boot_loader_name_tag() {
        println!("bootloader is: {}", boot_loader_name_tag.name());
    }

    x86::idt_init();
    x86::pic_init();
    x86::unmask_irq(0);
    x86::unmask_irq(4);

    let closed_fn = return_a_closure(10);
    println!("Call a closure: {}", closed_fn(10));

    let mut some_jump_buf = JmpBuf::new();
    let is_return = unsafe { set_jump(&mut some_jump_buf) };
    println!("set_jump returned: {}", is_return);

    if is_return == 0 {
        unsafe { jump_back(&some_jump_buf) };
    }

    thread::spawn(|| {
        println!("This is happening in a thread\n");
        thread::exit();
    });

    unsafe {
        x86::enable_irqs();
    }

    // thread::sched_yield();

    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn c_interrupt_shim(frame: *mut x86::InterruptFrame) {
    let interrupt = (*frame).interrupt_number;

    // println!("interrupt: {}", interrupt);

    // let f = unsafe { &*frame };
    // write!(serial, "{:?}\r\n", f).unwrap();

    if interrupt == 36 {
        let c = GLOBAL_SERIAL.lock().read_byte();
        println!("serial read: {}", c as char);
    }

    if interrupt == 32 {
        // thread::timeout();
    }

    if interrupt >= 32 && interrupt < 48 {
        x86::send_eoi(interrupt - 32);
    }

    if interrupt == 14 {
        panic!("Page fault, not handled");
    }
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    let mut serial = unsafe { serial::SerialPort::new_raw(0x3f8) };

    write!(serial, "{}\r\n", panic_info).unwrap();

    loop {}
}
