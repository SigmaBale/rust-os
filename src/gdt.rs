use lazy_static::lazy_static;
use x86_64::registers::segmentation::{CS, Segment};
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::instructions::tables::load_tss;
use x86_64::VirtAddr;

// Interrupt stack table to be used for double fault exceptions at index 0.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    // --------------------------------------------------------------------------------------------------------------┑
    // Task State Segment contains interrupt_stack_table that holds safe stack frames to prevent overflow, these     |
    // these stacks are used for interrupts.                                                                         |
    // CPU will invoke page fault exception and will push the exception stack frame on to the                        |
    // already overflowed stack, actually it will check that stack pointer is still pointing                         |
    // to the non-existing guard page and will throw page fault again which causes double fault.                     | 
    // So the CPU tries to invoke double fault handler now and will again have to push the exception_stack           |
    // on to the stack, except it will again see that the stack pointer is pointing to the non-existing guard page   |
    // and will triple-fault, thus shutting down the CPU (powering off the machine).                                 |
    // Task State Segment allows for stack switching via interrupt_stack_table, thus preventing prior from happening.|
    // --------------------------------------------------------------------------------------------------------------┙
    // TSS also holds privilege_stack_table, that is used when exception occurs in user-made programs to swap stacks.
    // Privilege stack table holds 4 stack tables, each representing privilege level (0, 1, 2, 3).
    // Modern OS only uses 0-kernel and 3-user.
    // In user-space privilege level is 3, if exception occurs then CPU will change privilege level to 0 and it
    // will switch stacks to the 0th stack in the privilege_stack_table since 0 is the target privilege level.
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

    // Global Descriptor Table is legacy structure that was used mostly before memory paging was a thing and when segmentation was a thing.
    // Now it is mostly used for two things: Switching between kernel space and user space, and loading a TSS structure.
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

/// **Initialize [`GDT`][x86_64::structures::gdt::GlobalDescriptorTable]**, aka load the gdt into
/// the *global descriptor table register* (`GDTR`).
/// 
/// Inline Assembly:
///```no_run
///#[inline]
///pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
///    unsafe {
///        // We pass the Global Descriptor Table pointer that contains limit and base:
///        // limit: u16 = size_of::<GlobalDescriptorTable>() - 1; (max = 255)
///        // address = crate::VirtAddr::new(self.table.as_ptr() as u64);
///        asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags));
///    }
///}
///```
/// We also set the `CS` register and load the [`TSS`], assembly for TSS:
///```no_run
/// #[inline]
///pub unsafe fn load_tss(sel: SegmentSelector) {
///    unsafe {
///       asm!("ltr {0:x}", in(reg) sel.0, options(nostack, preserves_flags));
///    }
///}
///```
pub fn init_gdt() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}