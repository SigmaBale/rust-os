#![no_std] // Don't link the rust std library
#![no_main] // Disable all Rust-level entry points
#![feature(custom_test_frameworks)] // Custom test framework, because of #![no_std]
#![test_runner(crate::test_runner)] // Test runner function = test_runner
#![reexport_test_harness_main = "test_main"] // Renaming/ReExporting test function name to test_main because of no_main attribute

mod vga_buffer;

#[no_mangle] // Don't mangle the name of this function
// This fn is the entry point, since the linker looks for function named '_start' by default,
// this is the reason for no_mangle attribute.
pub extern "C" fn _start() -> ! {
    for i in 0..25 {
        println!("{i}. El. Psy. Kongroo.");
    }

    #[cfg(test)]
    test_main();

    loop {}
}

// This function is called on panic.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}

#[test_case]
fn trivial_assertion() {
    print!("trivial_assertion...");
    assert!(1 == 1);
    println!("[ok]");
}


