use core::marker::PhantomData;
use core::ops::Deref;
use core::ops::DerefMut;

use aarch64_cpu::registers::MAIR_EL1;
use aarch64_cpu::registers::TCR_EL1;
use aarch64_cpu::registers::TTBR0_EL1;
use aarch64_cpu::registers::TTBR1_EL1;
use tock_registers::interfaces::Writeable;
use tock_registers::register_bitfields;
use tock_registers::registers::InMemoryRegister;

#[repr(C)]
pub struct PageTable<S> {
    l3: [[Page; 8192]; 8],
    l2: [Table; 8],
    _space: PhantomData<S>,
}

pub fn init() {
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Device::nonGathering_nonReordering_noEarlyWriteAck
            + MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc,
    );

    TCR_EL1.write(
        // TCR_EL1::HD::Enable
        // TCR_EL1::HA::Enable
        TCR_EL1::TBI0::Used
            + TCR_EL1::TBI1::Used
            + TCR_EL1::A1::TTBR0
            + TCR_EL1::TG0::KiB_64
            + TCR_EL1::TG1::KiB_64
            + TCR_EL1::SH0::Inner
            + TCR_EL1::SH1::Inner
            + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::EPD0::EnableTTBR0Walks
            + TCR_EL1::EPD1::EnableTTBR1Walks
            + TCR_EL1::IPS::Bits_32
            + TCR_EL1::AS::ASID8Bits
            + TCR_EL1::T0SZ.val(32)
            + TCR_EL1::T1SZ.val(32),
    );
}

impl<S: crate::mem::AddressSpace> PageTable<S> {
    pub fn init(&mut self, offset: u64) {
        for (l2, l3) in self.l2.iter_mut().zip(&self.l3) {
            l2.write(
                table::TYPE::Table
                    + table::VALID::Valid
                    + table::NEXT.val((l3.as_ptr() as u64) >> 16),
            )
        }

        // FIXME: set up device MMIO in kernel
        for (virt, phys) in (0x3F00_0000..0x4001_0000)
            .step_by(1 << 16)
            .map(|phys| (phys + offset, phys))
            .map(|(virt, phys)| (crate::mem::Virt::new(virt), crate::mem::Phys::new(phys)))
        {
            self.map(virt, phys, Attr::Device);
        }

        match offset {
            0 => TTBR0_EL1.set_baddr(self.l2.as_ptr() as u64),
            _ => {
                for (virt, phys) in (self as *mut _ as u64..)
                    .step_by(1 << 16)
                    .take(self.l3.len() + 1)
                    .map(|phys| (phys + offset, phys))
                    .map(|(virt, phys)| (crate::mem::Virt::new(virt), crate::mem::Phys::new(phys)))
                {
                    self.map(
                        virt,
                        phys,
                        Attr::Normal {
                            read: true,
                            write: true,
                            execute: false,
                        },
                    );
                }

                TTBR1_EL1.set_baddr(self.l2.as_ptr() as u64);
            }
        }
    }

    pub fn map(&mut self, virt: crate::mem::Virt<S>, phys: crate::mem::Phys, attr: Attr) {
        let index_l2 = u64::from(virt) >> 29 & ((1 << 3) - 1);
        let index_l3 = (u64::from(virt) >> 16) & ((1 << 13) - 1);

        let entry = &mut self.l3[index_l2 as usize][index_l3 as usize];

        let mut flags = page::AF::True
            + page::TYPE::Page
            + page::VALID::True
            + page::NEXT.val(u64::from(phys) >> 16);

        match attr {
            Attr::Device => flags += page::SH::Outer + page::AP::RW_EL1 + page::INDEX.val(0),
            Attr::Normal {
                read,
                write,
                execute,
            } => {
                flags += page::SH::Inner + page::INDEX.val(1);

                if execute {
                    flags += page::PXN::CLEAR;
                } else {
                    flags += page::PXN::SET;
                }

                flags += page::UXN::SET;

                flags += match (read, write) {
                    (true, true) => page::AP::RW_EL1,
                    (true, false) => page::AP::RO_EL1,
                    _ => unimplemented!(),
                };
            }
        }

        entry.write(flags);
    }
}

#[derive(Debug)]
pub enum Attr {
    Device,
    Normal {
        read: bool,
        write: bool,
        execute: bool,
    },
}

#[repr(transparent)]
struct Table(InMemoryRegister<u64, table::Register>);

impl Deref for Table {
    type Target = InMemoryRegister<u64, table::Register>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Table {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[repr(transparent)]
struct Page(InMemoryRegister<u64, page::Register>);

impl Deref for Page {
    type Target = InMemoryRegister<u64, page::Register>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Page {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

register_bitfields! {
    u64,
    table [
        NEXT OFFSET(16) NUMBITS(32) [],

        TYPE OFFSET(1) NUMBITS(1) [
            Block = 0,
            Table = 1,
        ],

        VALID OFFSET(0) NUMBITS(1) [
            Invalid = 0,
            Valid = 1,
        ],
    ],

    page [
        UXN OFFSET(54) NUMBITS(1) [],

        PXN OFFSET(53) NUMBITS(1) [],

        NEXT OFFSET(16) NUMBITS(32) [],

        AF OFFSET(10) NUMBITS(1) [
            False = 0,
            True = 1,
        ],

        SH OFFSET(8) NUMBITS(2) [
            No = 0b00,
            Reserved = 0b01,
            Outer = 0b10,
            Inner = 0b11,
        ],

        AP OFFSET(6) NUMBITS(2) [
            RW_EL1 = 0b00,
            RW_EL0 = 0b01,
            RO_EL1 = 0b10,
            RO_EL0 = 0b11,
        ],

        INDEX OFFSET(2) NUMBITS(3) [],

        TYPE OFFSET(1) NUMBITS(1) [
            Reserved = 0,
            Page = 1,
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1,
        ],
    ],
}
