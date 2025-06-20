use core::fmt::Debug;

pub type Byte = Mem<0>;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mem<const SHIFT: usize = 0>(usize);

impl<const SHIFT: usize> Mem<SHIFT> {
    pub const fn new(len: usize) -> Self {
        Self(len)
    }

    pub const fn convert<const TARGET: usize>(&self) -> Mem<TARGET> {
        match TARGET >= SHIFT {
            false => Mem(self.0 << (SHIFT - TARGET)),
            true => Mem(self.0 >> (TARGET - SHIFT)),
        }
    }

    pub const fn value(&self) -> usize {
        self.0
    }
}

impl<const SHIFT: usize> Debug for Mem<SHIFT> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let bytes = self.convert::<0>().value();
        Debug::fmt(&bytes, f)?;
        write!(f, "B")
    }
}
