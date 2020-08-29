use x86_64::structures::paging;
use x86_64::structures::paging::mapper;

use crate::util;
use crate::util::Tap as _;

pub mod bump;

pub const ADDR: usize = 0x4444_4444_0000;
pub const SIZE: usize = 100 * 1024;

#[global_allocator]
static ALLOCATOR: util::Mutex<bump::Allocator> = util::Mutex::new(bump::Allocator::new());

pub fn init<M, F>(
    page_table_mapper: &mut M,
    frame_allocator: &mut F,
) -> Result<(), mapper::MapToError<paging::Size4KiB>>
where
    M: paging::Mapper<paging::Size4KiB>,
    F: paging::FrameAllocator<paging::Size4KiB>,
{
    let pages = paging::Page::range(
        ADDR.tap(|addr| addr as u64)
            .tap(x86_64::VirtAddr::new)
            .tap(paging::Page::containing_address),
        ADDR.tap(|addr| (addr + SIZE) as u64)
            .tap(x86_64::VirtAddr::new)
            .tap(paging::Page::containing_address),
    );

    for page in pages {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(mapper::MapToError::FrameAllocationFailed)?;
        let flags = paging::PageTableFlags::PRESENT | paging::PageTableFlags::WRITABLE;
        unsafe {
            page_table_mapper
                .map_to(page, frame, flags, frame_allocator)?
                .flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(ADDR, SIZE);
    }

    Ok(())
}

pub fn align(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
