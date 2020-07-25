use lazy_static::lazy_static;
use x86_64::structures::tss;
use x86_64::structures::gdt;
use x86_64::instructions::segmentation;
use x86_64::instructions::tables;

pub const DOUBLE_FAULT: u16 = 0;
pub const STACK_SIZE: usize = 4096;

lazy_static! {
    static ref GDT: (gdt::GlobalDescriptorTable, Selectors) = {
        let mut gdt = gdt::GlobalDescriptorTable::new();
        let code = gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        let tss = gdt.add_entry(gdt::Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code, tss })
    };

    static ref TSS: tss::TaskStateSegment = {
        let mut tss = tss::TaskStateSegment::new();

        // Allocate space for double fault exception handler stack
        tss.interrupt_stack_table[DOUBLE_FAULT as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let lo = x86_64::VirtAddr::from_ptr(unsafe { &STACK });
            let hi = lo + STACK_SIZE;
            hi
        };

        tss
    };
}

#[derive(Copy, Clone, Debug)]
struct Selectors {
    code: gdt::SegmentSelector,
    tss: gdt::SegmentSelector,
}

pub fn init() {
    let (gdt, selectors) = &*GDT;
    gdt.load();
    unsafe {
        segmentation::set_cs(selectors.code);
        tables::load_tss(selectors.tss);
    }
}
