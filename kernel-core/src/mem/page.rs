use crate::mem::Phys;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(u64);

impl Id {
    pub(super) fn from_usize(index: usize) -> Self {
        Self(index as u64)
    }

    pub(super) fn into_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<Id> for u64 {
    fn from(Id(id): Id) -> Self {
        id
    }
}

impl From<Phys> for Id {
    fn from(phys: Phys) -> Self {
        Self(u64::from(phys) >> 16)
    }
}
