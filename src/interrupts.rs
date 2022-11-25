use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::println;
use lazy_static::lazy_static;

lazy_static!{
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt
    };
}

/// **Initialize [`IDT`][x86_64::structures::idt::InterruptDescriptorTable]**, aka load the idt into
/// the interrupt descriptor table register.
/// 
/// Assembly:
///```no_run
///fn lido(idt: &DescriptorTablePointer) {
///    // We pass the Interrupt Descriptor Table pointer that contains limit and base:
///    // limit: u16 = size_of::<InterruptDescriptorTable>() - 1; (max = 255)
///    // address: u64 = InterruptDescriptorTable as *const _ as u64;
///    asm!("lido [{}]", in(reg) idt, options(preserve_flags, readonly, nostack)); 
///}
///```
pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

