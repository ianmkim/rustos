#![no_std]
// main function doesn't make any sense because usually there is
// an underlying runtime which sets up the main entrypoint. For instance
// in JAVA this might be the garbage collection system. In Rust, where there
// is minimal underlying runtime, this is the crt0, which sets up the environment
// for a C application. Then this crt0 invokes the entrypoint for the rust runtime
// However, we can't do that for our binary, so we don't have a main function
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rustos::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
mod serial;

use core::panic::PanicInfo;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11
}

// make sure that the name _start is preserved. Usually Rust will randomize function
// names in order to ensure that each function has a unique name
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello world{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}


/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rustos::test_panic_handler(info)
}
