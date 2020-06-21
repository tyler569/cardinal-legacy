extern "C" {
    pub fn outb(port: u16, val: u8);
    pub fn inb(port: u16) -> u8;
}
