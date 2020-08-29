use core::ptr;

use alloc::alloc;

use crate::util;

#[derive(Debug)]
pub struct Allocator {
    start: usize,
    end: usize,
    next: usize,
    allocations: usize,
}

impl Allocator {
    pub const fn new() -> Self {
        Allocator {
            start: 0,
            end: 0,
            next: 0,
            allocations: 0,
        }
    }

    pub unsafe fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.next = start;
    }
}

unsafe impl alloc::GlobalAlloc for util::Mutex<Allocator> {
    unsafe fn alloc(&self, layout: alloc::Layout) -> *mut u8 {
        let mut bump = self.lock();

        let start = super::align(bump.next, layout.align());
        let end = match start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if end > bump.end {
            ptr::null_mut()
        } else {
            bump.next = end;
            bump.allocations += 1;
            start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: alloc::Layout) {
        let mut bump = self.lock();
        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.start;
        }
    }
}
