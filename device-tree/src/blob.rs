//! https://github.com/devicetree-org/devicetree-specification/releases/tag/v0.4

use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::NonNull;

use crate::Be32;
use crate::Prop;
use crate::StrList;

pub struct Blob<'dtb>(&'dtb [u8]);

impl<'dtb> Blob<'dtb> {
    pub const fn new(dtb: &'dtb [u8]) -> Self {
        Self(dtb)
    }

    /// # Safety
    ///
    /// Caller must guarantee `pointer` is a pointer to a valid device tree blob.
    pub unsafe fn from_ptr(pointer: NonNull<u8>) -> Self {
        let header = unsafe { pointer.cast::<Header>().as_ref() };
        assert_eq!(u32::from(header.magic), Header::MAGIC);
        let len = u32::from(header.total_size) as usize;
        Self(unsafe { core::slice::from_raw_parts(pointer.as_ptr(), len) })
    }

    pub fn header(&self) -> &'dtb Header<'dtb> {
        unsafe { self.as_ptr().cast::<Header>().as_ref() }
    }

    pub fn root(&self) -> Node {
        let struct_offset = u32::from(self.header().off_dt_struct);
        let walk = unsafe {
            self.as_ptr()
                .byte_add(struct_offset as usize)
                .cast::<Be32>()
        };

        NodeIter(Cursor { dtb: self, walk })
            .next()
            .expect("Missing root node")
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

    #[expect(unused)]
    fn str_slice(slice: &'dtb [u8]) -> &'dtb str {
        let Some((0, slice)) = slice.split_last() else {
            panic!("Malformed device tree string: {:?}", slice);
        };

        str::from_utf8(slice).expect("Expected UTF-8 device tree string")
    }

    pub fn as_ptr(&self) -> NonNull<u8> {
        NonNull::from(self.0).cast::<u8>()
    }
}

#[derive(Clone, Debug)]
pub struct Root<'dtb>(Node<'dtb>);

impl<'dtb> Deref for Root<'dtb> {
    type Target = Node<'dtb>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'dtb> Root<'dtb> {
    pub fn aliases(&self) -> PropIter<'dtb> {
        let mut cursor = self.0.cursor.clone();
        cursor.seek_child(|name| name == "aliases");
        cursor.next();
        PropIter(cursor)
    }

    pub fn memory(&self) -> Node<'dtb> {
        let mut cursor = self.0.cursor.clone();
        cursor.seek_child(|name| name == "memory");
        NodeIter(cursor).next().expect("Missing /memory node")
    }

    pub fn reserved_memory(&self) -> Option<Node<'dtb>> {
        let mut cursor = self.0.cursor.clone();
        cursor.seek_child(|name| name == "memory");
        NodeIter(cursor).next()
    }

    pub fn cpus(&self) -> Node<'dtb> {
        let mut cursor = self.0.cursor.clone();
        cursor.seek_child(|name| name == "cpus");
        NodeIter(cursor).next().expect("Missing /cpus node")
    }
}

#[derive(Clone)]
pub struct Node<'dtb> {
    name: &'dtb str,
    compatible: StrList<'dtb>,
    phandle: u32,
    address_cells: u32,
    size_cells: u32,
    // Positioned at first property within node
    cursor: Cursor<'dtb>,
}

impl<'dtb> core::fmt::Debug for Node<'dtb> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct(self.name)
            .field("compatible", &self.compatible)
            .field("phandle", &self.phandle)
            .finish_non_exhaustive()
    }
}

impl<'dtb> Node<'dtb> {
    pub fn name(&self) -> &'dtb str {
        self.name
    }

    pub fn compatible(&self) -> StrList<'dtb> {
        self.compatible.clone()
    }

    pub fn phandle(&self) -> u32 {
        self.phandle
    }

    pub fn address_cells(&self) -> u32 {
        self.address_cells
    }

    pub fn size_cells(&self) -> u32 {
        self.size_cells
    }

    pub fn props(&self) -> PropIter<'dtb> {
        PropIter(self.cursor.clone())
    }

    pub fn children(&self) -> NodeIter<'dtb> {
        let mut cursor = self.cursor.clone();
        cursor.seek_child(|_| true);
        NodeIter(cursor)
    }
}

// Invariant: cursor always positioned at `Token::Begin` or `Token::End`
pub struct NodeIter<'dtb>(Cursor<'dtb>);

impl<'dtb> Iterator for NodeIter<'dtb> {
    type Item = Node<'dtb>;

    fn next(&mut self) -> Option<Self::Item> {
        let name = match self.0.peek()? {
            Token::Begin { name } => {
                self.0.next();
                name
            }
            Token::Prop { .. } => unreachable!(),
            Token::End => return None,
        };

        let compatible = self
            .0
            .clone()
            .find_prop("compatible")
            .map(StrList::new)
            .unwrap_or(StrList::new(&[]));

        let phandle = self
            .0
            .clone()
            .find_prop("phandle")
            .map(|value| u32::from_be_bytes(value.try_into().expect("Invalid phandle prop")))
            .unwrap_or(u32::MAX);

        let address_cells = self
            .0
            .clone()
            .find_prop("#address-cells")
            .map(|value| u32::from_be_bytes(value.try_into().expect("Invalid #address-cells prop")))
            .unwrap_or(2);

        let size_cells = self
            .0
            .clone()
            .find_prop("#size-cells")
            .map(|value| u32::from_be_bytes(value.try_into().expect("Invalid #size-cells prop")))
            .unwrap_or(1);

        let next = Node {
            name,
            compatible,
            phandle,
            address_cells,
            size_cells,
            cursor: self.0.clone(),
        };

        self.0.seek_sibling();
        Some(next)
    }
}

pub struct PropIter<'dtb>(Cursor<'dtb>);

impl<'dtb> Iterator for PropIter<'dtb> {
    type Item = Prop<'dtb>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.0.peek()? {
            Token::Begin { .. } | Token::End => None,
            Token::Prop(prop) => Some(prop),
        }
    }
}

#[derive(Clone)]
struct Cursor<'dtb> {
    dtb: &'dtb Blob<'dtb>,
    walk: NonNull<Be32>,
}

impl<'dtb> Cursor<'dtb> {
    fn find_prop(&mut self, name: &str) -> Option<&'dtb [u8]> {
        self.take_while(|token| matches!(token, Token::Prop { .. }))
            .find_map(|token| match token {
                Token::Prop(prop) if prop.name == name => Some(prop.value),
                Token::Begin { .. } | Token::Prop { .. } | Token::End => None,
            })
    }

    fn seek_child<F: FnMut(&'dtb str) -> bool>(&mut self, mut filter: F) {
        let mut depth = 0usize;
        loop {
            match self.peek() {
                Some(Token::Begin { name }) if depth == 0 && filter(name) => return,
                Some(Token::Begin { .. }) => depth += 1,
                None => {
                    assert_eq!(depth, 0);
                    return;
                }
                Some(Token::End) if depth == 0 => return,
                Some(Token::End) => depth -= 1,
                Some(Token::Prop(_)) => (),
            }

            self.next();
        }
    }

    fn seek_sibling(&mut self) {
        let mut depth = 0usize;
        loop {
            match self.next() {
                None => return,
                Some(Token::End) if depth == 0 => return,
                Some(Token::Begin { .. }) => depth += 1,
                Some(Token::End) => depth = depth.checked_sub(1).unwrap(),
                Some(Token::Prop { .. }) => (),
            }
        }
    }

    fn peek(&self) -> Option<Token<'dtb>> {
        self.peek_full().and_then(|(token, _)| token)
    }

    fn peek_full(&self) -> Option<(Option<Token<'dtb>>, usize)> {
        let (next, len) = match u32::from(unsafe { self.walk.read() }) {
            1 => unsafe {
                let name = Blob::str_pointer(self.walk.add(1).cast::<u8>());
                let aligned = (name.len() + 1 + 3) >> 2;
                (Some(Token::Begin { name }), 1 + aligned)
            },
            2 => (Some(Token::End), 1),
            3 => unsafe {
                let len = u32::from(self.walk.add(1).read()) as usize;
                let nameoff = u32::from(self.walk.add(2).read());

                let name = self.dtb.str_offset(nameoff);
                let value =
                    core::slice::from_raw_parts(self.walk.add(3).cast::<u8>().as_ptr(), len);

                let aligned = len.next_multiple_of(4) / 4;
                (Some(Token::Prop(Prop { name, value })), 3 + aligned)
            },
            4 => (None, 1),
            9 => {
                return None;
            }
            unknown => unreachable!("Unknown token: {:#x}", unknown),
        };

        Some((next, len))
    }
}

impl<'dtb> Iterator for Cursor<'dtb> {
    type Item = Token<'dtb>;

    fn next(&mut self) -> Option<Self::Item> {
        // Loop to skip past empty tokens
        loop {
            let (next, len) = self.peek_full()?;

            unsafe {
                self.walk = self.walk.add(len);
            }

            if let Some(next) = next {
                break Some(next);
            }
        }
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

impl Header<'_> {
    const MAGIC: u32 = 0xd00dfeed;

    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        u32::from(self.total_size) as usize
    }
}

#[derive(Clone)]
pub enum Token<'dtb> {
    Begin { name: &'dtb str },
    Prop(Prop<'dtb>),
    End,
}
