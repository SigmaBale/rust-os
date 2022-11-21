#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use nostd_color::{colorize::Colored, colors::{BRIGHT_GREEN, BRIGHT_RED, YELLOW}};
use rust_os::{exit_qemu, QemuExitCode, serial_print, serial_println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

// If test panics, then test passes. Only one test can be defined, because after each panic, 
// we exit qemu.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[{}]", "ok".fg(BRIGHT_GREEN));
    exit_qemu(QemuExitCode::Success);
    loop {}
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    for test in tests {
        test();
        serial_print!("[{}]", "failed".fg(BRIGHT_RED));
        exit_qemu(QemuExitCode::Failed);
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn should_fail() {
    serial_print!("{}...\t", "should_panic::should_fail".fg(YELLOW));
    assert!(1 == 0);
}