#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use rust_os::{serial_print, QemuExitCode, exit_qemu, htl_loop};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use nostd_color::colorize::Colored;
use nostd_color::colors::{GREEN, YELLOW, RED};

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(rust_os::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        
        idt
    };
}

extern "x86-interrupt" fn test_double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    serial_print!("[{}]\n", "ok".fg(GREEN));
    exit_qemu(QemuExitCode::Success);
    htl_loop()
}

fn idt_init_test() {
    TEST_IDT.load();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("{}...\t", "stack_overflow::stack_oveflow".fg(YELLOW));

    rust_os::gdt::init_gdt();
    idt_init_test();

    stack_overflow();

    panic!("Execution continued after overflow!");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    htl_loop()
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // Each recursion return address is pushed eventually overflowing the stack
    volatile::Volatile::new(0).read(); // Prevent tail recursion optimization
}