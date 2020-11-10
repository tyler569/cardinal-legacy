use crate::{x86,serial,thread};

const DETAIL_PRINT: bool = false;

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
            dprintln!("Interrupt {} Triggered at {:x}",
                      interrupt, (*frame).ip);
            panic!();
        }
    }
}

use core::ops::Drop;

pub struct InterruptDisabler;

impl InterruptDisabler {
    pub fn new() -> Self {
        unsafe { x86::disable_irqs() };
        InterruptDisabler{}
    }
}

impl Drop for InterruptDisabler {
    fn drop(&mut self) {
        unsafe { x86::enable_irqs() };
    }
}
