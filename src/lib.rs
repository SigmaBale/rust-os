#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)] // Experimental calling convention for interrupt/exception handler functions

pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod gdt;

use nostd_color::colors::{BRIGHT_RED, BRIGHT_GREEN, YELLOW, RED};
use nostd_color::colorize::Colored;

/// General init function for our OS.
/// 
pub fn init() {
    gdt::init_gdt();
    interrupts::init_idt();
    unsafe { interrupts::init_pics() }
    // The interrupts::enable function of the x86_64 crate executes the special sti instruction
    // (“set interrupts” - assembly) to enable external interrupts.
    x86_64::instructions::interrupts::enable();
}

pub fn htl_loop() -> ! {
    loop {
        // Enter idle state aka halts the CPU until next interrupt arrives.
        x86_64::instructions::hlt();
    }
}

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

// ---------------------------------------------------------------------------------------- //
//                                                                                          //
//                                     Tests section                                        //
//                                                                                          //
// ---------------------------------------------------------------------------------------- //
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
    htl_loop()
}

// ------------------------------------ //
//       ENTRY AND PANIC HANDLER        //
// ------------------------------------ //
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    htl_loop()
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

#[test_case]
fn test_exception_handler() {
    x86_64::instructions::interrupts::int3();
}