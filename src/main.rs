#![no_std] // Don't link the rust std library
#![no_main] // Disable all Rust-level entry points

mod vga_buffer;

#[no_mangle] // Don't mangle the name of this function
// This fn is the entry point, since the linker looks for 
// function named '_start' by default
pub extern "C" fn _start() -> ! {
    for i in 0..25 {
        println!("{i}. Am I writing to the VGA buffer????");
    }
    println!("Oh no, my first iteration is gone, small buffer...");
    loop {}
}

// This function is called on panic.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    loop {}
}