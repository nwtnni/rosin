use crate::bitset;
use crate::mem::page;

#[repr(transparent)]
pub struct Page(bitset::Unsized);

impl Page {
    pub const unsafe fn from_raw_parts_mut<'a>(pointer: *mut u64, len: usize) -> &'a mut Self {
        unsafe {
            core::mem::transmute(bitset::Unsized::from_raw_parts_mut(
                pointer.cast::<u64>(),
                len,
            ))
        }
    }

    pub const fn size_of(len: usize) -> usize {
        bitset::Unsized::size_of(len)
    }

    pub fn fill(&mut self) {
        self.0.fill_mut()
    }

    pub fn len(&mut self) -> usize {
        self.0.count_ones_mut()
    }

    pub fn is_empty(&mut self) -> bool {
        self.len() == 0
    }

    pub fn allocate(&mut self) -> Option<page::Id> {
        let page = self.0.peek_mut()?;
        self.0.unset_mut(page);
        Some(page::Id::from_usize(page))
    }

    pub fn reserve(&mut self, page: page::Id) {
        let page = page.into_usize();
        assert!(self.0.get_mut(page));
        self.0.unset_mut(page);
    }

    pub fn deallocate(&mut self, page: page::Id) {
        let page = page.into_usize();
        assert!(!self.0.get_mut(page));
        self.0.set_mut(page);
    }
}
