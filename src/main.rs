#![no_std] // Don't link the rust std library
#![no_main] // Disable all Rust-level entry points
#![feature(custom_test_frameworks)] // Custom test framework, because of #![no_std]
#![test_runner(rust_os::test_runner)] // Test runner function = test_runner
#![reexport_test_harness_main = "test_main"] // Renaming/ReExporting test function name to test_main because of no_main attribute

use rust_os::println;
use core::panic::PanicInfo;

#[no_mangle] // Don't mangle the name of this function
// This fn is the entry point, since the linker looks for function named '_start' by default,
// this is the reason for no_mangle attribute.
pub extern "C" fn _start() -> ! {
    println!("El. Psy. Kongroo.");

    rust_os::init();

    x86_64::instructions::interrupts::int3();

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
    rust_os::test_panic_handler(info)
}