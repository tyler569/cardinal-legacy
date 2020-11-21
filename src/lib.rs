#![no_std]
#![feature(alloc_error_handler)]
#![feature(negative_impls)]
#![feature(ffi_returns_twice)]
#![allow(dead_code)]

#[cfg(target_os = "none")]
use core::panic::PanicInfo;

extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

use spin as sync;

#[macro_use]
mod debug;

// This should be early, since it defines println!
#[macro_use]
mod serial;

#[cfg(target_os = "none")]
mod allocator;
mod interrupt;
mod memory;
mod phy_map;
mod thread;
mod util;
mod x86;

use memory::LOAD_OFFSET;

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
        phy_map::map_init(memory_map_tag.all_memory_areas());
    }

    for module_tag in boot_info.module_tags() {
        println!("module: {}", module_tag.name());
    }

    x86::idt_init();
    x86::pic_init();
    x86::unmask_irq(4);
    x86::timer_init(1000);
    x86::unmask_irq(0);

    for q in 0..40 {
        thread::spawn(move || dprint!("a{}", q));
        thread::spawn(move || dprint!("b{}", q));
        thread::spawn(move || dprint!("c{}", q));
        thread::spawn(move || dprint!("d{}", q));
        thread::spawn(move || dprint!("e{}", q));
    }

    thread::spawn(|| {
        thread::spawn(|| {
            println!("INNNNNNNNNNER");
        });
    });

    // for _ in 0..30 {
    //     thread::spawn(|| dprint!("a"));
    //     thread::spawn(|| dprint!("b"));
    //     thread::spawn(|| dprint!("c"));
    //     thread::spawn(|| dprint!("d"));
    //     thread::spawn(|| dprint!("e"));
    // }

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
