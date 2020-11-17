use crate::x86::{self, FaultCode};
use crate::{serial, thread};

const DETAIL_PRINT: bool = false;

const EXCEPTIONS: [&str; 32] = [
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
//
// enum Exception {
//     DivByZero = 0,
//     Debug = 1,
//     Nmi = 2,
//     Breakpoint = 3,
//     OverflowTrap = 4,
//     OutOfBounds = 5,
//     InvalidOpcode = 6,
//     NoDevice = 7,
//     DoubleFault = 8,
//     InvalidTSS = 10,
//     InvalidSegment = 11,
//     StackFault = 12,
//     GeneralProtectionFault = 13,
//     PageFault = 14,
//     X87 = 16,
//     AlignmentCheck = 17,
//     MachineCheck = 18,
//     Simd = 19,
//     Virtualization = 20,
//     Security = 30,
//     Reserved,
// }
//
// impl Display for Exception {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         let name = match *self {
//             DivByZero => "Divide by zero",
//             Debug => "Debug",
//             Nmi => "Non-maskable Interrupt",
//             Breakpoint => "Breakpoint",
//             OverflowTrap => "Overflow Trap",
//             OutOfBounds => "Bound Range Exceeded",
//             InvalidOpcode => "Invalid Opcode",
//             NoDevice => "Device Not Available",
//             DoubleFault => "Double Fault",
//             InvalidTSS => "Invalid TSS",
//             InvalidSegment => "Segment Not Present",
//             StackFault => "Stack-Segment Fault",
//             GeneralProtectionFault => "General Protection Fault",
//             PageFault => "Page Fault",
//             X87 => "x87 Floating Point Exception",
//             AlignmentCheck => "Alignment Check",
//             MachineCheck => "Machine Check",
//             Simd => "SIMD Floating-Point Exception",
//             Virtualization => "Virtualization Exception",
//             Security => "Security Exception",
//             _ => "Reserved",
//         }
//         write!(f, "{}", name)
//     }
// }

#[no_mangle]
pub unsafe extern "C" fn c_interrupt_shim(frame: *mut x86::InterruptFrame) {
    let interrupt = (*frame).interrupt_number;

    if DETAIL_PRINT {
        println!("interrupt: {}", interrupt);
        let f = &*frame;
        dprintln!("{:#x?}", f);
    }

    #[allow(clippy::match_overlapping_arm)]
    match interrupt {
        14 => {
            dprintln!("Page fault at {:#x}", x86::read_cr2());
            dprintln!("Fault occurred at {:#x}", (*frame).ip);
            dprintln!(
                "Page fault code: {:?}",
                FaultCode::from_bits((*frame).error_code as u16)
            );
            panic!();
        }
        32 => {
            x86::send_eoi(interrupt - 32);
            thread::schedule();
        }
        36 => {
            let c = serial::GLOBAL_SERIAL.lock().read_byte();
            dprintln!("serial read: {}", c as char);
            x86::send_eoi(interrupt - 32);
        }
        32..=48 => {
            x86::send_eoi(interrupt - 32);
        }
        _ => {
            dprintln!(
                "Interrupt {} ({}) Triggered at {:x}",
                interrupt,
                EXCEPTIONS[interrupt],
                (*frame).ip
            );
            panic!();
        }
    }
}

use core::ops::Drop;

pub struct InterruptDisabler;

impl InterruptDisabler {
    pub fn new() -> Self {
        x86::disable_irqs();
        InterruptDisabler {}
    }
}

impl Drop for InterruptDisabler {
    fn drop(&mut self) {
        x86::enable_irqs();
    }
}
