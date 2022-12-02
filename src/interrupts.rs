use x86_64::{structures::idt::{InterruptDescriptorTable, InterruptStackFrame}, instructions::port::Port};
use pic8259::ChainedPics;
use lazy_static::lazy_static;
use crate::{println, gdt, print};

lazy_static!{
    // ---------------------------------------------------------------------------------------------------------------------------------┐
    // Interrupt Descriptor Table is a structure that is responsible for handling hardware and software interrupts.                     |
    // It holds entries and each entry represents the interrupt handler aka a function that is invoked on each interrupt.               |
    // IDT holds up to 256 interrupt handlers.                                                                                          |
    // First 32 entries are reserved by the CPU, and those are called exceptions (these are automatically thrown by hardware aka CPU).  |
    // Rest of the interrupts are customizable and can be used as system calls or other kinds of interrupts.                            |                                                           |
    // ---------------------------------------------------------------------------------------------------------------------------------┚
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            // Here we are able to modify and set options and handler function for each entry inside our IDT.
            idt.double_fault
                // Method for setting our handler function, for our selected entry.
                .set_handler_fn(double_fault_handler)
                // We can of course set which stack to use from interrupt_stack_table in GDT. (0-7)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

/// **Initialize [`IDT`][x86_64::structures::idt::InterruptDescriptorTable]**, aka load the idt into
/// the *interrupt descriptor table register* (`IDTR`).
/// 
/// Inline Assembly:
///```no_run
///fn lidt(idt: &DescriptorTablePointer) {
///    // We pass the Interrupt Descriptor Table pointer that contains limit and base:
///    // limit: u16 = size_of::<InterruptDescriptorTable>() - 1; (max = 255)
///    // address: u64 = InterruptDescriptorTable as *const _ as u64;
///    asm!("lidt [{}]", in(reg) idt, options(preserves_flags, readonly, nostack)); 
///}
///```
pub fn init_idt() {
    IDT.load();
}

// ---------------------------------------------------------------------------------------------------------------------------------------------┐
// Calling conventions specify how arguments are passed to a function, how return values are passed back out of a function,                     |
// how the function is called, and how the function manages the stack and its stack frame.                                                      |
// In short, the calling convention specifies how a function call in C is converted into assembly language.                                     |
// Each interrupt handler uses "x86-interrupt" calling convention, instead of compiler pushing all callee-saved registers on to the stack,      |
// compiler will know which registers to push and restore for the interrupt handler function, thus also providing higher performance.           |
// This is different from `CDECL` calling convention (C standard cc), also when we return from these exceptions we use iret instead of ret.     |
// x86-interrupt calling convention basically knows to search for the register values on the stack instead of looking for them in registers.    |
// ---------------------------------------------------------------------------------------------------------------------------------------------┚
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    print!(".");

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8())
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{Keyboard, layouts, ScancodeSet1, HandleControl, DecodedKey};
    use spin::Mutex;

    lazy_static! {
        pub static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(char) => print!("{}", char),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8())
    }
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// -----------------------------------------------------------------------------------------┐
//                      ____________                          ____________                  |
// Real Time Clock --> |            |   Timer -------------> |            |                 |
// ACPI -------------> |            |   Keyboard-----------> |            |      _____      |
// Available --------> | Secondary  |----------------------> | Primary    |     |     |     |
// Available --------> | Interrupt  |   Serial Port 2 -----> | Interrupt  |---> | CPU |     |
// Mouse ------------> | Controller |   Serial Port 1 -----> | Controller |     |_____|     |
// Co-Processor -----> |            |   Parallel Port 2/3 -> |            |                 |
// Primary ATA ------> |            |   Floppy disk -------> |            |                 |
// Secondary ATA ----> |____________|   Parallel Port 1----> |____________|                 |
//                                                                                          |
// -----------------------------------------------------------------------------------------┚
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}
