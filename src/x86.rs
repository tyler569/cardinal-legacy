extern "C" {
    pub fn outb(port: u16, val: u8);
    pub fn inb(port: u16) -> u8;

    pub fn enable_irqs();
    pub fn disable_irqs();
}

#[repr(C)]
#[derive(Debug)]
pub struct InterruptFrame {
    pub ds: usize,
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub bp: usize,
    pub di: usize,
    pub si: usize,
    pub dx: usize,
    pub bx: usize,
    pub cx: usize,
    pub ax: usize,
    pub interrupt_number: usize,
    pub error_code: usize,
    pub ip: usize,
    pub cs: usize,
    pub flags: usize,
    pub sp: usize,
    pub ss: usize,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct JmpBuf {
    pub bx: usize,
    pub bp: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
    pub sp: usize,
    pub ip: usize,
}

impl JmpBuf {
    pub fn new() -> JmpBuf {
        Default::default()
    }
}

extern "C" {
    #[ffi_returns_twice]
    pub fn set_jump(buf: &mut JmpBuf) -> isize;
    pub fn long_jump(buf: &JmpBuf, value: isize) -> !;
}

const PRIMARY_PIC_COMMAND: u16 = 0x20;
const PRIMARY_PIC_DATA: u16 = 0x21;
const SECONDARY_PIC_COMMAND: u16 = 0xA0;
const SECONDARY_PIC_DATA: u16 = 0xA1;

pub fn unmask_irq(irq: usize) {
    if irq >= 8 {
        let mut mask = unsafe { inb(SECONDARY_PIC_DATA) };
        mask &= !(1 << (irq - 8));
        unsafe { outb(SECONDARY_PIC_DATA, mask) };
    } else {
        let mut mask = unsafe { inb(PRIMARY_PIC_DATA) };
        mask &= !(1 << irq);
        unsafe { outb(PRIMARY_PIC_DATA, mask) };
    }
}

pub fn mask_irq(irq: usize) {
    if irq >= 8 {
        let mut mask = unsafe { inb(SECONDARY_PIC_DATA) };
        mask |= 1 << (irq - 8);
        unsafe { outb(SECONDARY_PIC_DATA, mask) };
    } else {
        let mut mask = unsafe { inb(PRIMARY_PIC_DATA) };
        mask |= 1 << irq;
        unsafe { outb(PRIMARY_PIC_DATA, mask) };
    }
}

pub fn pic_init() {
    unsafe {
        outb(PRIMARY_PIC_COMMAND, 0x11); // Reprogram
        outb(PRIMARY_PIC_DATA, 0x20); // interrupt 0x20
        outb(PRIMARY_PIC_DATA, 0x04); // child PIC one line 2
        outb(PRIMARY_PIC_DATA, 0x01); // 8086 mode
        outb(PRIMARY_PIC_DATA, 0xFF); // mask all interrupts
        outb(SECONDARY_PIC_COMMAND, 0x11); // Reprogram
        outb(SECONDARY_PIC_DATA, 0x28); // interrupt 0x28
        outb(SECONDARY_PIC_DATA, 0x02); // ?
        outb(SECONDARY_PIC_DATA, 0x01); // 8086 mode
        outb(SECONDARY_PIC_DATA, 0xFF); // mask all interrupts
    }

    unmask_irq(2); // cascade irq
}

const TIMER_CH0: u16 = 0x40;
const TIMER_CMD: u16 = 0x43;

const TIMER_CHANNEL_0: u8 = 0;
const TIMER_ACCESS_HILO: u8 = 0x30;
const TIMER_MODE_3: u8 = 0x06; // square wave


pub fn timer_init(hertz: usize) {
    let mut divisor = 1_193_182 / hertz;
    if divisor > 65535 {
        // 0 represents 65536 and is the largest possible divisor,
        // giving 18.2Hz
        divisor = 0;
    }

    unsafe {
        outb(TIMER_CMD, TIMER_CHANNEL_0 | TIMER_ACCESS_HILO | TIMER_MODE_3);
        outb(TIMER_CH0, divisor as u8);
        outb(TIMER_CH0, (divisor >> 8) as u8);
    }
}

pub fn send_eoi(irq: usize) {
    if irq >= 8 {
        unsafe { outb(SECONDARY_PIC_COMMAND, 0x20) };
    }
    unsafe { outb(PRIMARY_PIC_COMMAND, 0x20) };
}

extern "C" {
    #[link_name = "idt"]
    static mut IDT: [u64; 1024];

    fn isr0();
    fn isr1();
    fn isr2();
    fn isr3();
    fn isr4();
    fn isr5();
    fn isr6();
    fn isr7();
    fn isr8();
    fn isr9();
    fn isr10();
    fn isr11();
    fn isr12();
    fn isr13();
    fn isr14();
    fn isr15();
    fn isr16();
    fn isr17();
    fn isr18();
    fn isr19();
    fn isr20();
    fn isr21();
    fn isr22();
    fn isr23();
    fn isr24();
    fn isr25();
    fn isr26();
    fn isr27();
    fn isr28();
    fn isr29();
    fn isr30();
    fn isr31();

    fn irq0();
    fn irq1();
    fn irq2();
    fn irq3();
    fn irq4();
    fn irq5();
    fn irq6();
    fn irq7();
    fn irq8();
    fn irq9();
    fn irq10();
    fn irq11();
    fn irq12();
    fn irq13();
    fn irq14();
    fn irq15();

    fn isr_syscall();
    fn isr_panic();
}

fn raw_set_idt_gate(irq: usize, handler: u64, flags: u64, cs: u64, ist: u64) {
    let gate = unsafe { &mut IDT[irq * 2..irq * 2 + 2] };

    let func = handler;
    let func_low = func & 0xFFFF;
    let func_mid = (func >> 16) & 0xFFFF;
    let func_high = func >> 32;

    gate[0] = func_low | (cs << 16) | (ist << 32) | (flags << 40) | (func_mid << 48);
    gate[1] = func_high;
}

fn set_idt_gate(irq: usize, handler: unsafe extern "C" fn()) {
    let rpl = 0; // 3 on syscall
    let gdt_selector = 8;
    let gate_type = 0xE;
    let flags = 0x80 | rpl << 5 | gate_type;

    // This is a defined bit structure and needs to be u64.
    #[allow(clippy::fn_to_numeric_cast)]
    raw_set_idt_gate(irq, handler as u64, flags, gdt_selector, 0);
}

pub fn idt_init() {
    set_idt_gate(0, isr0);
    set_idt_gate(1, isr1);
    set_idt_gate(2, isr2);
    set_idt_gate(3, isr3);
    set_idt_gate(4, isr4);
    set_idt_gate(5, isr5);
    set_idt_gate(6, isr6);
    set_idt_gate(7, isr7);
    set_idt_gate(8, isr8);
    set_idt_gate(9, isr9);
    set_idt_gate(10, isr10);
    set_idt_gate(11, isr11);
    set_idt_gate(12, isr12);
    set_idt_gate(13, isr13);
    set_idt_gate(14, isr14);
    set_idt_gate(15, isr15);
    set_idt_gate(16, isr16);
    set_idt_gate(17, isr17);
    set_idt_gate(18, isr18);
    set_idt_gate(19, isr19);
    set_idt_gate(20, isr20);
    set_idt_gate(21, isr21);
    set_idt_gate(22, isr22);
    set_idt_gate(23, isr23);
    set_idt_gate(24, isr24);
    set_idt_gate(25, isr25);
    set_idt_gate(26, isr26);
    set_idt_gate(27, isr27);
    set_idt_gate(28, isr28);
    set_idt_gate(29, isr29);
    set_idt_gate(30, isr30);
    set_idt_gate(31, isr31);

    set_idt_gate(32, irq0);
    set_idt_gate(33, irq1);
    set_idt_gate(34, irq2);
    set_idt_gate(35, irq3);
    set_idt_gate(36, irq4);
    set_idt_gate(37, irq5);
    set_idt_gate(38, irq6);
    set_idt_gate(39, irq7);
    set_idt_gate(40, irq8);
    set_idt_gate(41, irq9);
    set_idt_gate(42, irq10);
    set_idt_gate(43, irq11);
    set_idt_gate(44, irq12);
    set_idt_gate(45, irq13);
    set_idt_gate(46, irq14);
    set_idt_gate(47, irq15);

    // set_idt_gate(128, isr_syscall);
    set_idt_gate(130, isr_panic);
}
