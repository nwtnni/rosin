use core::sync::atomic::AtomicU64;

#[repr(transparent)]
pub struct Unsized([AtomicU64]);

#[repr(C)]
pub struct Sized<const LEN: usize>([AtomicU64; LEN]);

macro_rules! impl_bit_set {
    () => {
        pub fn clear_mut(&mut self) {
            self.0
                .iter_mut()
                .map(AtomicU64::get_mut)
                .for_each(|chunk| *chunk = 0);
        }

        pub fn get_mut(&mut self, index: usize) -> bool {
            let (i, j) = Self::split(index);
            *self.0[i].get_mut() & (1 << j) > 0
        }

        pub fn set_mut(&mut self, index: usize) {
            let (i, j) = Self::split(index);
            *self.0[i].get_mut() |= 1 << j;
        }

        pub fn unset_mut(&mut self, index: usize) {
            let (i, j) = Self::split(index);
            *self.0[i].get_mut() &= !(1 << j);
        }

        pub fn peek_mut(&mut self) -> Option<usize> {
            let (i, j) = self.0.iter_mut().enumerate().find_map(|(i, chunk)| {
                match chunk.get_mut().trailing_ones() {
                    64 => None,
                    j => Some((i, j)),
                }
            })?;

            Some(i * 64 + j as usize)
        }

        const fn split(index: usize) -> (usize, u64) {
            (index / 64, (index % 64) as u64)
        }
    };
}

impl Unsized {
    // https://users.rust-lang.org/t/how-to-create-instances-of-user-defined-unsized-types/46128/3
    pub const unsafe fn from_raw_parts<'a>(pointer: *mut u64, len: usize) -> &'a Self {
        unsafe {
            // SAFETY:
            // - `u64` has the same representation as `AtomicU64`
            // - `Unsized` is marked `#[repr(transparent)]`
            core::mem::transmute::<&[AtomicU64], &Self>(core::slice::from_raw_parts(
                pointer.cast::<AtomicU64>(),
                len,
            ))
        }
    }

    pub const unsafe fn from_raw_parts_mut<'a>(pointer: *mut u64, len: usize) -> &'a mut Self {
        unsafe {
            // SAFETY:
            // - `u64` has the same representation as `AtomicU64`
            // - `Unsized` is marked `#[repr(transparent)]`
            core::mem::transmute::<&mut [AtomicU64], &mut Self>(core::slice::from_raw_parts_mut(
                pointer.cast::<AtomicU64>(),
                len,
            ))
        }
    }

    impl_bit_set!();
}

impl<const LEN: usize> Sized<LEN> {
    pub const fn new() -> Self {
        Self([const { AtomicU64::new(0) }; LEN])
    }

    impl_bit_set!();
}
