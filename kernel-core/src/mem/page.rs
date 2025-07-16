use crate::mem::Phys;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(u64);

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
