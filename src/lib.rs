#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod vga_buffer;
pub mod serial;

use nostd_color::colors::{BRIGHT_RED, BRIGHT_GREEN, YELLOW, RED};
use nostd_color::colorize::Colored;

#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10, // 16 
    Failed = 0x11, // 17
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>().fg(YELLOW));
        self();
        serial_println!("[{}]", "ok".fg(BRIGHT_GREEN));
    }
}

// ---------------------------------------------------------------------------------------- //
//                                                                                          //
//                                     Tests section                                        //
//                                                                                          //
// ---------------------------------------------------------------------------------------- //
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("\nRunning {} tests", tests.len().fg(YELLOW));
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("[{}]", "failed".fg(BRIGHT_RED));
    serial_println!("{}: {}\n","Error".fg(RED), info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

// ------------------------------------ //
//       ENTRY AND PANIC HANDLER        //
// ------------------------------------ //
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

// This function is called on panic.
#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
}

// ------------------------------------ //
//              UNIT TESTS              //
// ------------------------------------ //
#[test_case]
fn test_println_basic() {
    println!("Basic println test.");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("Testing println many.");
    }
}