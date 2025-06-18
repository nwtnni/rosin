use core::fmt::Debug;
use core::marker::PhantomData;

pub mod page;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Phys(u64);

impl Phys {
    pub const fn new(address: u64) -> Self {
        assert!(address < (1 << 32));
        Self(address)
    }
}

impl From<Phys> for u64 {
    fn from(phys: Phys) -> Self {
        phys.0
    }
}

impl From<page::Id> for Phys {
    fn from(id: page::Id) -> Self {
        Self::new(u64::from(id) << 16)
    }
}

impl Debug for Phys {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "P")?;
        self.0.fmt(f)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Virt<S> {
    address: u64,
    _space: PhantomData<S>,
}

impl<S: AddressSpace> Virt<S> {
    pub fn new(address: u64) -> Self {
        assert!(S::validate(address));
        Self {
            address,
            _space: PhantomData,
        }
    }
}

impl<S> From<Virt<S>> for u64 {
    fn from(virt: Virt<S>) -> Self {
        virt.address
    }
}

impl<S> Debug for Virt<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "V")?;
        self.address.fmt(f)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Kernel;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct User;

#[expect(private_bounds)]
pub trait AddressSpace: seal::Seal + Copy + Clone + core::fmt::Debug {
    fn validate(address: u64) -> bool;
}

impl AddressSpace for Kernel {
    fn validate(address: u64) -> bool {
        // address >= OFFSET
        true
    }
}

impl AddressSpace for User {
    fn validate(address: u64) -> bool {
        address < OFFSET
    }
}

mod seal {
    pub(super) trait Seal {}
    impl Seal for super::Kernel {}
    impl Seal for super::User {}
}

pub const OFFSET: u64 = 0xFFFF_FFFF_0000_0000;
