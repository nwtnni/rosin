use bootloader::bootinfo;
use x86_64::registers::control;
use x86_64::structures::paging;
use x86_64::structures::paging::Mapper as _;

use crate::util::Tap as _;

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

pub fn create_example_mapping<F>(
    page: paging::Page,
    page_table: &mut paging::OffsetPageTable,
    frame_allocator: &mut F,
) where F: paging::FrameAllocator<paging::Size4KiB> {

    let frame = 0xB8000
        .tap(x86_64::PhysAddr::new)
        .tap(paging::PhysFrame::containing_address);

    let flags = paging::PageTableFlags::PRESENT | paging::PageTableFlags::WRITABLE;

    unsafe {
        page_table
            .map_to(page, frame, flags, frame_allocator)
            .expect("`create_example_mapping` failed")
            .flush();
    }
}

#[derive(Debug)]
pub struct BootInfoFrameAllocator {
    memory_map: &'static bootinfo::MemoryMap,
    region: usize,
    page: u64,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static bootinfo::MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            region: 0,
            page: 0,
        }
    }
}

unsafe impl paging::FrameAllocator<paging::Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<paging::PhysFrame> {
        while let Some(region) = self.memory_map.get(self.region) {
            if region.region_type == bootinfo::MemoryRegionType::Usable {
                let phys = region.range.start_addr() + self.page * 4096;
                if phys < region.range.end_addr() {
                    self.page += 1;
                    return phys
                        .tap(x86_64::PhysAddr::new)
                        .tap(paging::PhysFrame::containing_address)
                        .tap(Option::Some);
                }
            }
            self.region += 1;
            self.page = 0;
        }
        None
    }
}
