#![no_std]
#![feature(alloc_error_handler)]
#![feature(negative_impls)]
#![feature(ffi_returns_twice)]
#![feature(const_btree_new)] 

#![allow(dead_code)]

#[cfg(target_os = "none")]
use core::panic::PanicInfo;

extern crate alloc;
use alloc::boxed::Box;

#[macro_use]
extern crate lazy_static;

pub use spin as sync;

#[cfg(target_os = "none")]
mod allocator;
mod thread;
mod serial;
mod x86;

use x86::{long_jump, set_jump, JmpBuf};

const LOAD_OFFSET: usize = 0xFFFF_FFFF_8000_0000;
const USE_TIMER: bool = false;

fn return_a_closure(x: i32) -> Box<dyn FnOnce(i32) -> i32> {
    Box::new(move |a| a + x)
}

unsafe fn jump_back(buffer: &JmpBuf) -> ! {
    long_jump(buffer, 0);
}

#[no_mangle]
pub extern "C" fn kernel_main(_multiboot_magic: u32, multiboot_info: usize) -> ! {
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
    x86::unmask_irq(4);

    if USE_TIMER {
        x86::timer_init(1000);
        x86::unmask_irq(0);
    }

    let closed_fn = return_a_closure(10);
    println!("Call a closure: {}", closed_fn(10));

    let mut some_jump_buf = JmpBuf::new();
    let is_return = unsafe { set_jump(&mut some_jump_buf) };
    println!("set_jump returned: {}", is_return);

    if is_return == 0 {
        unsafe { jump_back(&some_jump_buf) };
    }

    thread::spawn(|| { println!("This is a thread"); });
    thread::spawn(|| { println!("This is a thread too"); });
    thread::schedule();

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
        let c = serial::GLOBAL_SERIAL.lock().read_byte();
        println!("serial read: {}", c as char);
    }

    if interrupt == 32 {
        // thread::timeout();
    }

    if interrupt >= 32 && interrupt < 48 {
        x86::send_eoi(interrupt - 32);
    }

    if interrupt == 14 {
        println!("Page fault at {:x}", x86::read_cr2());
        panic!("not handled");
    }
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    // TODO: we may have panic'd while holding the serial lock, this could
    // probably deadlock.
    println!("{}", panic_info);

    loop {}
}
