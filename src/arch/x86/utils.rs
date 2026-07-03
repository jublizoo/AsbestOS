unsafe extern "C" {
    unsafe fn _outb(port: u16, data: u8);
    unsafe fn _inb(port: u16) -> u8;

    unsafe fn _ltr() -> u16;
    unsafe fn _str(register: u16);
    unsafe fn _lgdt() -> *const u8;
    unsafe fn _sgdt(gdt_ptr: *const u8);
}

pub unsafe fn write_port_byte(port: u16, data: u8) {
    unsafe { _outb(port, data) }
}

pub unsafe fn read_port_byte(port: u16) -> u8 {
    unsafe { _inb(port) }
}


pub unsafe fn load_task_register() -> u16 {
    unsafe { _ltr() }
}

pub unsafe fn store_task_register(register: u16) {
    unsafe { _str(register) };
}

pub unsafe fn load_gdt() -> *const u8 {
    unsafe { _lgdt() }
}

pub unsafe fn store_gdt(gdt_ptr: *const u8) {
    unsafe { _sgdt(gdt_ptr) };
}
