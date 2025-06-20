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

pub struct Allocator([u64; 256]);

impl Allocator {
    pub const fn new() -> Self {
        Self([0; 256])
    }

    pub fn clear_mut(&mut self) {
        self.0.fill(0);
    }

    pub fn reserve_mut(&mut self, id: Id) {
        let (i, j) = Self::id_to_index(id);
        self.0[i] |= 1 << j;
    }

    pub fn allocate_mut(&mut self) -> Option<Id> {
        let (i, j) =
            self.0
                .iter()
                .enumerate()
                .find_map(|(i, chunk)| match chunk.trailing_ones() {
                    64 => None,
                    j => Some((i, j)),
                })?;

        let id = Id(i as u64 * 64 + j as u64);
        self.reserve_mut(id);
        Some(id)
    }

    pub fn deallocate_mut(&mut self, id: Id) {
        let (i, j) = Self::id_to_index(id);

        assert!(self.0[i] & (1 << j) > 0, "Double free at page {:?}", id,);

        self.0[i] &= !(1 << j);
    }

    fn id_to_index(id: Id) -> (usize, u64) {
        ((id.0 / 64) as usize, id.0 % 64)
    }
}

impl Default for Allocator {
    fn default() -> Self {
        Self::new()
    }
}
