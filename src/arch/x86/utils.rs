
unsafe extern "C" {
    fn _outb(port: u16, data: u8);
    fn _inb(port: u16) -> u8;
}

pub unsafe fn write_port_byte(port: u16, data: u8) {
    unsafe { _outb(port, data) }
}

pub unsafe fn read_port_byte(port: u16) -> u8 {
    unsafe { _inb(port) }
}
