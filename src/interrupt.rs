use crate::{x86,serial,thread};

const DETAIL_PRINT: bool = false;

const EXCEPTIONS: [&'static str; 32] = [
    "Divide by zero",
    "Debug",
    "Non-maskable Interrupt",
    "Breakpoint",
    "Overflow Trap",
    "Bound Range Exceeded",
    "Invalid Opcode",
    "Device Not Available",
    "Double Fault",
    "Coprocessor Segment Overrun (Deprecated)",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Reserved",
    "x87 Floating Point Exception",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
    "Virtualization Exception",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Security Exception",
    "Reserved",
];

#[no_mangle]
pub unsafe extern "C" fn c_interrupt_shim(frame: *mut x86::InterruptFrame) {
    let interrupt = (*frame).interrupt_number;

    if DETAIL_PRINT {
        println!("interrupt: {}", interrupt);
        let f = &*frame;
        dprintln!("{:#x?}", f);
    }

    match interrupt {
        14 => {
            dprintln!("Page fault at {:x}", x86::read_cr2());
            panic!();
        },
        32 => {
            x86::send_eoi(interrupt - 32);
            thread::schedule();
        },
        36 => {
            let c = serial::GLOBAL_SERIAL.lock().read_byte();
            dprintln!("serial read: {}", c as char);
            x86::send_eoi(interrupt - 32);
        },
        32..=48 => {
            x86::send_eoi(interrupt - 32);
        },
        _ => {
            dprintln!("Interrupt {} ({}) Triggered at {:x}",
                      interrupt, EXCEPTIONS[interrupt], (*frame).ip);
            panic!();
        }
    }
}

use core::ops::Drop;

pub struct InterruptDisabler;

impl InterruptDisabler {
    pub fn new() -> Self {
        x86::disable_irqs();
        InterruptDisabler{}
    }
}

impl Drop for InterruptDisabler {
    fn drop(&mut self) {
        x86::enable_irqs();
    }
}
