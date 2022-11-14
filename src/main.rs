#![no_std] // Don't link the rust std library
#![no_main] // Disable all Rust-level entry points

use core::panic::PanicInfo;

#[no_mangle] // Don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    // This fn is the entry point, since the linker looks for 
    // function named '_start' by default
    loop {}
}

// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}