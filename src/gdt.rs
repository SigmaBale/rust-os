use lazy_static::lazy_static;
use x86_64::registers::segmentation::{CS, Segment};
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::instructions::tables::load_tss;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    // --------------------------------------------------------------------------------------------------------------┑
    // Task State Segment contains interrupt stack table to use a "safe" stack                                       |
    // that is not overflowed, these stacks are used for interrupts e.g. if stack overflows                          |
    // CPU will invoke page fault exception and will push the exception stack frame on to the                        |
    // already overflowed stack, actually it will check that stack pointer is still pointing                         |
    // to the non-existing guard page and will throw page fault again which causes double fault.                     | 
    // So the CPU tries to invoke double fault handler now and will again have to push the exception stack           |
    // on to the stack, except it will again see that the stack pointer is pointing to the non-existing guard page   |
    // and will triple-fault, thus shutting down the CPU (powering off the machine).                                 |
    // Task State Segment allows for stack switching via interrupt stack table, thus preventing prior from happening.|
    // --------------------------------------------------------------------------------------------------------------┙
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };

    // Global Descriptor Table is legacy structure that was used mostly before memory paging was a thing
    // and when memory 
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init_gdt() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}