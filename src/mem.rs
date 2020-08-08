use x86_64::registers::control;
use x86_64::structures::paging;

pub unsafe fn init(
    phys_mem_offset: x86_64::VirtAddr,
) -> paging::OffsetPageTable<'static> {
    let (level_4_table_frame, _) = control::Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = phys_mem_offset + phys.as_u64();
    paging::OffsetPageTable::new(
        &mut *virt.as_mut_ptr(),
        phys_mem_offset,
    )
}
