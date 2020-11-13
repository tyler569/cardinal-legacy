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
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

pub use spin as sync;

#[macro_use]
mod debug;

// This should be early, since it defines println!
#[macro_use]
mod serial;

#[cfg(target_os = "none")]
mod allocator;
mod interrupt;
mod memory;
mod thread;
mod x86;

pub use memory::phy::PhysicalAddress;
pub use memory::virt::VirtualAddress;

const LOAD_OFFSET: usize = 0xFFFF_FFFF_8000_0000;
const PHY_OFFSET: usize = 0xFFFF_8000_0000_0000;
const USE_TIMER: bool = true;
const MULTIBOOT2_MAGIC: u32 = 0x36d76289;

#[no_mangle]
pub extern "C" fn kernel_main(
    multiboot_magic: u32,
    multiboot_info: usize,
) -> ! {
    println!("Hello World from the Cardinal Operating System");

    assert_eq!(multiboot_magic, MULTIBOOT2_MAGIC);

    let boot_info =
        unsafe { multiboot2::load_with_offset(multiboot_info, LOAD_OFFSET) };

    if let Some(boot_loader_name_tag) = boot_info.boot_loader_name_tag() {
        println!("bootloader is: {}", boot_loader_name_tag.name());
    }

    if let Some(memory_map_tag) = boot_info.memory_map_tag() {
        memory::phy::map_init(memory_map_tag.all_memory_areas());
    }

    for module_tag in boot_info.module_tags() {
        println!("module: {}", module_tag.name());
    }

    x86::idt_init();
    x86::pic_init();
    x86::unmask_irq(4);
    x86::timer_init(1000);
    x86::unmask_irq(0);

    println!("Let's test some formatting {}", 1234);
    let a = |x| x + 10;
    println!("Call a lambda: {}", a(10));
    fn return_a_closure(x: i32) -> Box<dyn FnOnce(i32) -> i32> {
        Box::new(move |a| a + x)
    }
    let closed_fn = return_a_closure(10);
    println!("Call a closure: {}", closed_fn(10));

    let test_page = memory::phy::alloc();
    println!("{:x?}", test_page);

    thread::spawn(|| println!("a"));
    thread::spawn(|| println!("b"));
    thread::spawn(|| println!("c"));
    thread::spawn(|| println!("d"));
    thread::spawn(|| println!("e"));

    let x = 10;
    thread::spawn(move || println!("{}", x));

    fn thread_opener() {
        println!("bare fn");
    }
    thread::spawn(thread_opener);

    thread::spawn(|| {
        for _ in 0..1000 {
            dprint!("a")
        }
    });
    thread::spawn(|| {
        for _ in 0..1000 {
            dprint!("b")
        }
    });

    thread::spawn(|| {
        for _ in 0..1000 {
            dprint!("a");
            thread::schedule();
        }
        println!();
    });
    thread::spawn(|| {
        for _ in 0..1000 {
            dprint!("b");
            thread::schedule();
        }
        println!();
    });

    x86::enable_irqs();
    thread::schedule();
    panic!("thread::schedule should never return to main");
}

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    dprintln!("{}", panic_info);
    loop {
        x86::disable_interrupts();
        x86::pause();
    }
}
