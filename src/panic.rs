use core::panic::PanicInfo;
use crate::vga::write_bytes;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    write_bytes(b"Panicking");
    // if let Some(msg) = _info.message().as_str() {
    //     write_bytes(msg.as_bytes());
    // } else {
    //     write_bytes(b"No panic message");
    // }

    loop { }
}
