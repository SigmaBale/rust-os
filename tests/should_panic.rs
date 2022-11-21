#![no_std]
#![no_main]

use core::panic::PanicInfo;
use nostd_color::{colorize::Colored, colors::{BRIGHT_GREEN, BRIGHT_RED, YELLOW}};
use rust_os::{exit_qemu, QemuExitCode, serial_print, serial_println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[{}]", "failed".fg(BRIGHT_RED));
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

// If test panics, then test passes.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[{}]", "ok".fg(BRIGHT_GREEN));
    exit_qemu(QemuExitCode::Success);
    loop {}
}

fn should_fail() {
    serial_print!("{}...\t", "should_panic::should_fail".fg(YELLOW));
    assert!(1 == 0);
}