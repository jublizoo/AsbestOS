use core::panic::PanicInfo;
use crate::vga::{VGA, VgaAttr, FgColor};
use crate::tty::TTY;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut tty = unsafe { TTY.force_lock() };
    if !tty.is_configured() {
        tty.configure();
    }

    let vga = &mut tty.vga;

    let red = VgaAttr::from_fg(FgColor::Red);
    vga.write_bytes(b"Panicking", red, 0, 0);
    if let Some(msg) = _info.message().as_str() {
        vga.write_bytes(msg.as_bytes(), red, 1, 0);
    } else {
        vga.write_bytes(b"No panic message", red, 1, 0);
    }

    loop { }
}
