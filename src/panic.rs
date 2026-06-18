use core::panic::PanicInfo;
use crate::vga::{VGA, VgaAttr, FgColor};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut vga = unsafe { VGA.force_lock() };
    if !vga.is_configured() {
        vga.configure();
    }

    let red = VgaAttr::from_fg(FgColor::Red);
    vga.write_bytes(b"Panicking", red, 0, 0);
    if let Some(msg) = _info.message().as_str() {
        vga.write_bytes(msg.as_bytes(), red, 1, 0);
    } else {
        vga.write_bytes(b"No panic message", red, 1, 0);
    }

    loop { }
}
