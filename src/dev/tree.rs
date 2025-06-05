//! https://github.com/devicetree-org/devicetree-specification/releases/tag/v0.4

use core::fmt::Debug;
use core::marker::PhantomData;
use core::ptr::NonNull;

use arrayvec::ArrayVec;

use crate::unit;

pub struct Blob<'dtb>(&'dtb [u8]);

impl<'dtb> Blob<'dtb> {
    pub const fn new(dtb: &'dtb [u8]) -> Self {
        Self(dtb)
    }

    pub fn header(&self) -> &'dtb Header<'dtb> {
        unsafe { self.as_ptr().cast::<Header>().as_ref() }
    }

    pub fn iter(&self) -> impl Iterator<Item = Token<'dtb>> {
        let structs = u32::from(self.header().off_dt_struct);
        let mut walk = unsafe { self.as_ptr().byte_add(structs as usize).cast::<Be32>() };
        let mut done = false;
        let mut stack = ArrayVec::<Context, 16>::new();

        core::iter::from_fn(move || {
            if done {
                return None;
            }

            loop {
                let (next, len) = match u32::from(unsafe { walk.read() }) {
                    1 => unsafe {
                        let name = Self::str_pointer(walk.add(1).cast::<u8>());
                        let aligned = (name.len() + 1 + 3) >> 2;
                        stack.push(Context::default());
                        (Some(Token::Begin { name }), 1 + aligned)
                    },
                    2 => {
                        stack.pop();
                        (Some(Token::End), 1)
                    }
                    3 => unsafe {
                        let len = u32::from(walk.add(1).read()) as usize;
                        let nameoff = u32::from(walk.add(2).read());

                        let name = self.str_offset(nameoff);
                        let value =
                            core::slice::from_raw_parts(walk.add(3).cast::<u8>().as_ptr(), len);

                        let aligned = (len + 3) >> 2;
                        (
                            Some(Token::Prop(Prop::new(&mut stack, name, value))),
                            3 + aligned,
                        )
                    },
                    4 => (None, 1),
                    9 => {
                        done = true;
                        return None;
                    }
                    unknown => unreachable!("Unknown token: {:#x}", unknown),
                };

                unsafe {
                    walk = walk.add(len);
                }

                if let Some(next) = next {
                    break Some(next);
                }
            }
        })
    }

    fn str_offset(&self, offset: u32) -> &'dtb str {
        let strings = u32::from(self.header().off_dt_strings);
        let base = unsafe {
            self.as_ptr()
                .byte_add(strings as usize)
                .byte_add(offset as usize)
        };
        Self::str_pointer(base)
    }

    fn str_pointer(base: NonNull<u8>) -> &'dtb str {
        let len = (0..)
            .map(|offset| unsafe { base.byte_add(offset).read() })
            .position(|byte| byte == 0)
            .unwrap();

        str::from_utf8(unsafe { core::slice::from_raw_parts(base.as_ptr(), len) })
            .expect("Invalid UTF-8 in device tree")
    }

    fn str_slice(slice: &'dtb [u8]) -> &'dtb str {
        let Some((0, slice)) = slice.split_last() else {
            panic!("Malformed device tree string: {:?}", slice);
        };

        str::from_utf8(slice).expect("Expected UTF-8 device tree string")
    }

    fn int_cell(cells: Cell, data: &'dtb [u8]) -> u64 {
        match cells.value() {
            0 => 0,
            1 => u32::from_be_bytes(data.try_into().unwrap()) as u64,
            2 => u64::from_be_bytes(data.try_into().unwrap()),
            _ => unimplemented!(),
        }
    }

    fn as_ptr(&self) -> NonNull<u8> {
        NonNull::from(self.0).cast::<u8>()
    }
}

#[repr(C, align(8))]
#[derive(Debug)]
pub struct Header<'dtb> {
    /// This field shall contain the value 0xd00dfeed (big-endian)
    magic: Be32,

    /// This field shall contain the total size in bytes of the devicetree data structure.
    /// This size shall encompass all sections of the structure: the header, the memory
    /// reservation block, structure block and strings block, as well as any free
    /// space gaps between the blocks or after the final block.
    total_size: Be32,

    /// This field shall contain the offset in bytes of the structure block (see Section 5.4)
    /// from the beginning of the header.
    off_dt_struct: Be32,

    /// This field shall contain the offset in bytes of the strings block (see Section 5.5)
    /// from the beginning of the header.
    off_dt_strings: Be32,

    /// This field shall contain the offset in bytes of the memory reservation block
    /// (see Section 5.3) from the beginning of the header.
    off_mem_rsvmap: Be32,

    /// This field shall contain the version of the devicetree data structure.
    /// The version is 17 if using the structure as defined in this document.
    /// An DTSpec boot program may provide the devicetree of a later version,
    /// in which case this field shall contain the version number defined in
    /// whichever later document gives the details of that version.
    version: Be32,

    /// This field shall contain the lowest version of the devicetree data structure
    /// with which the version used is backwards compatible. So, for the structure
    /// as defined in this document (version 17), this field shall contain 16 because version
    /// 17 is backwards compatible with version 16, but not earlier versions.
    /// As per Section 5.1, a DTSpec boot program should provide a devicetree in a
    /// format which is backwards compatible with version 16, and thus this field
    /// shall always contain 16.
    last_comp_version: Be32,

    /// This field shall contain the physical ID of the system’s boot CPU.
    /// It shall be identical to the physical ID given in the reg property of
    /// that CPU node within the devicetree.
    boot_cpuid_phys: Be32,

    /// This field shall contain the length in bytes of the strings block section
    /// of the devicetree blob.
    size_dt_strings: Be32,

    /// This field shall contain the length in bytes of the structure block section
    /// of the devicetree blob.
    size_dt_struct: Be32,

    _dtb: PhantomData<&'dtb ()>,
}

#[repr(C, align(8))]
#[derive(Copy, Clone, Debug)]
pub struct Reservation {
    address: Be64,
    size: Be64,
}

#[derive(Debug)]
struct Context {
    address: Cell,
    size: Cell,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            address: unit::Byte::new(2).convert(),
            size: unit::Byte::new(2).convert(),
        }
    }
}

pub enum Token<'dtb> {
    Begin { name: &'dtb str },
    Prop(Prop<'dtb>),
    End,
}

pub enum Prop<'dtb> {
    Compatible(StrList<'dtb>),
    Model(&'dtb str),
    AddressCells(Cell),
    SizeCells(Cell),
    Reg(RegList<'dtb>),
    Ranges(RangeList<'dtb>),
    Any { name: &'dtb str, value: &'dtb [u8] },
}

impl<'dtb> Prop<'dtb> {
    fn new(context: &mut [Context], name: &'dtb str, value: &'dtb [u8]) -> Self {
        match name {
            "compatible" => Prop::Compatible(StrList(value)),
            "model" => Prop::Model(Blob::str_slice(value)),
            "#address-cells" => {
                let address = Cell::new(u32::from_be_bytes(value.try_into().unwrap()) as usize);
                context.last_mut().unwrap().address = address;
                Prop::AddressCells(address)
            }
            "#size-cells" => {
                let size = Cell::new(u32::from_be_bytes(value.try_into().unwrap()) as usize);
                context.last_mut().unwrap().size = size;
                Prop::SizeCells(size)
            }
            "reg" => {
                let parent = &context[context.len() - 2];
                Prop::Reg(RegList {
                    address: parent.address.convert(),
                    size: parent.address.convert(),
                    data: value,
                })
            }
            "ranges" => {
                let parent = &context[context.len() - 2];
                Prop::Ranges(RangeList {
                    address: parent.address.convert(),
                    size: parent.address.convert(),
                    data: value,
                })
            }
            name => Prop::Any { name, value },
        }
    }
}

impl Debug for Prop<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let name = match self {
            Prop::Compatible(_) => "compatible",
            Prop::Model(_) => "model",
            Prop::AddressCells(_) => "#address-cells",
            Prop::SizeCells(_) => "#size-cells",
            Prop::Reg(_) => "reg",
            Prop::Ranges(_) => "ranges",
            Prop::Any { name, value: _ } => name,
        };

        write!(f, "{}: ", name)?;

        match self {
            Prop::Compatible(list) => {
                let mut iter = list.iter();
                if let Some(string) = iter.next() {
                    write!(f, "{}", string)?;
                }
                for string in iter {
                    write!(f, "; {}", string)?;
                }
                Ok(())
            }
            Prop::Model(model) => write!(f, "{}", model),
            Prop::AddressCells(cells) | Prop::SizeCells(cells) => write!(f, "{:?}", cells),
            Prop::Reg(ranges) => {
                let mut iter = ranges.iter();
                if let Some(range) = iter.next() {
                    write!(f, "{:?}", range)?;
                }
                for range in iter {
                    write!(f, "; {:?}", range)?;
                }
                Ok(())
            }
            Prop::Ranges(ranges) => {
                let mut iter = ranges.iter();
                if let Some(range) = iter.next() {
                    write!(f, "{:?}", range)?;
                }
                for range in iter {
                    write!(f, "; {:?}", range)?;
                }
                Ok(())
            }
            Prop::Any { name: _, value } => {
                write!(f, "{:?}", value)
            }
        }
    }
}

#[derive(Clone)]
pub struct RangeList<'dtb> {
    address: Cell,
    size: Cell,
    data: &'dtb [u8],
}

impl<'dtb> RangeList<'dtb> {
    pub fn iter(&self) -> impl Iterator<Item = Range> {
        let address: unit::Byte = self.address.convert();
        let len: unit::Byte = self.size.convert();

        self.data
            .chunks_exact(address.value() * 2 + len.value())
            .map(move |chunk| {
                let (child, chunk) = chunk.split_at(address.value());
                let (parent, len) = chunk.split_at(address.value());
                let parent = Blob::int_cell(self.address, parent);
                let child = Blob::int_cell(self.address, child);
                let len = Blob::int_cell(self.size, len);
                Range {
                    child,
                    parent,
                    len: unit::Byte::new(len as usize),
                }
            })
    }
}

#[derive(Copy, Clone)]
pub struct Range {
    pub child: u64,
    pub parent: u64,
    pub len: unit::Byte,
}

impl Debug for Range {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:#x} = {:#x} ({:#x})",
            self.child, self.parent, self.len,
        )
    }
}

#[derive(Clone)]
pub struct RegList<'dtb> {
    address: Cell,
    size: Cell,
    data: &'dtb [u8],
}

impl<'dtb> RegList<'dtb> {
    pub fn iter(&self) -> impl Iterator<Item = Reg> {
        let address: unit::Byte = self.address.convert();
        let len: unit::Byte = self.size.convert();

        self.data
            .chunks_exact(address.value() + len.value())
            .map(move |chunk| {
                let (address, len) = chunk.split_at(address.value());
                let address = Blob::int_cell(self.address, address);
                let len = Blob::int_cell(self.size, len);
                Reg {
                    address,
                    len: unit::Byte::new(len as usize),
                }
            })
    }
}

#[derive(Copy, Clone)]
pub struct Reg {
    pub address: u64,
    pub len: unit::Byte,
}

impl Debug for Reg {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:#x} - {:#x} ({:#x})",
            self.address,
            self.address + self.len.value() as u64,
            self.len,
        )
    }
}

#[derive(Clone)]
pub struct StrList<'dtb>(&'dtb [u8]);

impl<'dtb> StrList<'dtb> {
    pub fn iter(&self) -> impl Iterator<Item = &'dtb str> {
        self.0
            .split(|byte| *byte == 0)
            .filter(|str| !str.is_empty())
            .map(str::from_utf8)
            .map(Result::unwrap)
    }
}

type Cell = unit::Mem<2>;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Be32(u32);

impl From<Be32> for u32 {
    fn from(Be32(value): Be32) -> Self {
        if cfg!(target_endian = "little") {
            value.swap_bytes()
        } else {
            value
        }
    }
}

impl Debug for Be32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        u32::from(*self).fmt(f)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Be64(u64);

impl From<Be64> for u64 {
    fn from(Be64(value): Be64) -> Self {
        if cfg!(target_endian = "little") {
            value.swap_bytes()
        } else {
            value
        }
    }
}

impl Debug for Be64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        u64::from(*self).fmt(f)
    }
}
