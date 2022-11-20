#![no_std] // Don't link the rust std library
#![no_main] // Disable all Rust-level entry points
#![feature(custom_test_frameworks)] // Custom test framework, because of #![no_std]
#![test_runner(crate::test_runner)] // Test runner function = test_runner
#![reexport_test_harness_main = "test_main"] // Renaming/ReExporting test function name to test_main because of no_main attribute

mod vga_buffer;
mod serial;
use nostd_color::colorize::Colored;
use nostd_color::colors::*;

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
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial_println!("{}\n", "[failed]".fg(BRIGHT_RED));
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("\nRunning {} tests", tests.len());

    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    assert!(1 == 1);
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[{}]", "ok".fg(BRIGHT_GREEN));
    }
}
