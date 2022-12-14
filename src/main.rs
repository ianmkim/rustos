#![no_std]
// main function doesn't make any sense because usually there is
// an underlying runtime which sets up the main entrypoint. For instance
// in JAVA this might be the garbage collection system. In Rust, where there
// is minimal underlying runtime, this is the crt0, which sets up the environment
// for a C application. Then this crt0 invokes the entrypoint for the rust runtime
// However, we can't do that for our binary, so we don't have a main function
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

// the -> ! indicates that the function is diverging (i.e never returns)
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static HELLO: &[u8] = b"Hello World";

// make sure that the name _start is preserved. Usually Rust will randomize function
// names in order to ensure that each function has a unique name
#[no_mangle]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;
    vga_buffer::WRITER.lock().write_str("Hello again").unwrap();
    write!(vga_buffer::WRITER.lock(), ", Some numbers: {} {}", 42, 1.337).unwrap();

    loop {}
}
