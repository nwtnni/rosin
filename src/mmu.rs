use core::ops::Deref;
use core::ops::DerefMut;

use aarch64_cpu::asm;
use aarch64_cpu::registers::MAIR_EL1;
use aarch64_cpu::registers::ReadWriteable as _;
use aarch64_cpu::registers::Readable as _;
use aarch64_cpu::registers::SCTLR_EL1;
use aarch64_cpu::registers::TCR_EL1;
use aarch64_cpu::registers::TTBR0_EL1;
use tock_registers::interfaces::Writeable;
use tock_registers::register_bitfields;
use tock_registers::registers::InMemoryRegister;

static PAGE_TABLE: PageTable = PageTable::new();

#[repr(C, align(65536))]
pub struct PageTable {
    l3: [[Page; 8192]; 8],
    l2: [Table; 8],
}

unsafe impl Sync for PageTable {}

impl PageTable {
    const fn new() -> Self {
        Self {
            l3: [const { [const { Page::new() }; 8192] }; 8],
            l2: [const { Table::new() }; 8],
        }
    }
}

#[repr(transparent)]
struct Table(InMemoryRegister<u64, table::Register>);

impl Table {
    const fn new() -> Self {
        Self(InMemoryRegister::new(0))
    }
}

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

impl Page {
    const fn new() -> Self {
        Self(InMemoryRegister::new(0))
    }
}

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

#[inline(always)]
pub fn initialize() {
    MAIR_EL1.write(
        MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_noEarlyWriteAck,
    );

    let kernel_phys: u64;
    let kernel_virt: u64;

    unsafe {
        core::arch::asm!(
            "ldr {}, =__KERNEL_PHYS",
            "ldr {}, =__KERNEL_VIRT",
            out(reg) kernel_phys,
            out(reg) kernel_virt,
        );
    };

    let offset = kernel_virt - kernel_phys;

    let page_table_l2_virt: u64;
    let page_table_l3_virt: u64;

    unsafe {
        core::arch::asm!(
            "ldr {}, =__PAGE_TABLE_L2",
            "ldr {}, =__PAGE_TABLE_L3",
            out(reg) page_table_l2_virt,
            out(reg) page_table_l3_virt,
        );
    };

    let page_table_l2 = (page_table_l2_virt - offset) as *mut Table;
    let page_table_l3 = (page_table_l3_virt - offset) as *mut Page;

    let segment_rx_lo_virt: u64;
    let segment_rx_hi_virt: u64;

    unsafe {
        core::arch::asm!(
            "ldr {}, =__SEGMENT_RX_LO",
            "ldr {}, =__SEGMENT_RX_HI",
            out(reg) segment_rx_lo_virt,
            out(reg) segment_rx_hi_virt,
        );
    };

    for virt in (segment_rx_lo_virt..segment_rx_hi_virt).step_by(1 << 16) {
        let phys = virt - offset;

        let index_l2 = virt >> 29 & ((1 << 13) - 1);
        let index_l3 = (virt >> 16) & ((1 << 13) - 1);

        let l2 = unsafe {
            page_table_l2
                .wrapping_add(index_l2 as usize)
                .as_mut()
                .unwrap()
        };

        let l3 = unsafe {
            page_table_l3
                .wrapping_add(index_l3 as usize)
                .as_mut()
                .unwrap()
        };

        if !l2.is_set(table::VALID) {
            l2.write(
                table::TYPE::Table
                    + table::VALID::Valid
                    + table::NEXT.val(((page_table_l3 as usize as u64) >> 16) & ((1 << 13) - 1)),
            )
        }

        l3.write(
            page::AF::True
                + page::TYPE::Page
                + page::VALID::True
                + page::SH::Inner
                + page::AP::RW_EL1
                + page::INDEX.val(0)
                + page::NEXT.val((phys >> 16) & ((1 << 13) - 1)),
        );
    }

    TTBR0_EL1.write(TTBR0_EL1::BADDR.val(page_table_l3_virt - offset));

    TCR_EL1.write(
        TCR_EL1::EPD0::EnableTTBR0Walks
            + TCR_EL1::HD::Enable
            + TCR_EL1::HA::Enable
            + TCR_EL1::TBI1::Used
            + TCR_EL1::A1::TTBR1
            + TCR_EL1::TG1::KiB_64
            + TCR_EL1::SH1::Inner
            + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
            + TCR_EL1::EPD1::EnableTTBR1Walks
            + TCR_EL1::T1SZ.val(32),
    );

    asm::barrier::isb(asm::barrier::SY);

    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);

    asm::barrier::isb(asm::barrier::SY);
}
