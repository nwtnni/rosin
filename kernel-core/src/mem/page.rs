#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(u64);

pub struct Allocator([u64; 256]);

impl Allocator {
    pub const fn new() -> Self {
        Self([0; 256])
    }

    pub fn allocate(&mut self) -> Option<Id> {
        let (i, j) =
            self.0
                .iter()
                .enumerate()
                .find_map(|(i, chunk)| match chunk.trailing_ones() {
                    64 => None,
                    j => Some((i, j)),
                })?;

        self.0[i] |= 1 << j;
        Some(Id(i as u64 * 64 + j as u64))
    }

    pub fn deallocate(&mut self, id: Id) {
        let i = id.0 / 64;
        let j = id.0 % 64;

        assert!(
            self.0[i as usize] & (1 << j) > 0,
            "Double free at page {:?}",
            id,
        );

        self.0[i as usize] &= !(1 << j);
    }
}

impl Default for Allocator {
    fn default() -> Self {
        Self::new()
    }
}
