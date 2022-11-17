#![no_std] // Don't link the rust std library
#![no_main] // Disable all Rust-level entry points

use core::panic::PanicInfo;
mod vga_buffer;

#[no_mangle] // Don't mangle the name of this function
// This fn is the entry point, since the linker looks for 
// function named '_start' by default
pub extern "C" fn _start() -> ! {
    
    vga_buffer::print_smth();
    
    loop {}
}

// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}