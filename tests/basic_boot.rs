#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    loop {}
}

fn test_runner(tests: &[&dyn Fn()]) {
    for test in tests {
        test();
    }
}

