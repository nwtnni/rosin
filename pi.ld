/* https://forums.raspberrypi.com/viewtopic.php?t=328000 */
__KERNEL_PHYS = 0x8 << 16;
/* __KERNEL_VIRT = 0xFFFFFFFF << 32; */

ENTRY(__KERNEL_PHYS)

HIDDEN(PAGE_SIZE = 1 << 16);

PHDRS {
    segment_ro PT_LOAD FLAGS(4);
    segment_rx PT_LOAD FLAGS(5);
    segment_rw PT_LOAD FLAGS(6);
}

SECTIONS {
    __STACK_LO = .;
    . += __KERNEL_PHYS;
    __STACK_HI = .;

    __SEGMENT_RX_LO = .;

    .text : {
        KEEP(*(.text.boot))
        *( .text .text.* )
    } :segment_rx

    . = ALIGN(PAGE_SIZE);
    __SEGMENT_RX_HI = .;
    __SEGMENT_RO_LO = .;

    .got : ALIGN(16) {
        *( .got .got.* )
    } :segment_ro

    .rodata : ALIGN(16) {
        *( .rodata .rodata.* )
    } :segment_ro

    . = ALIGN(PAGE_SIZE);
    __SEGMENT_RO_HI = .;
    __SEGMENT_RW_LO = .;

    .data : ALIGN(16) {
        *( .data .data.* )
    } :segment_rw

    .bss (NOLOAD) : ALIGN(16) {
        __BSS_LO = .;
        *( .bss .bss.* )
        . = ALIGN(16);
        __BSS_HI = .;
    } :segment_rw

    . = ALIGN(PAGE_SIZE);
    __SEGMENT_RW_HI = .;

    ASSERT(SIZEOF(.got) == 0, "Unexpected relocation")

    /DISCARD/ : {
        *( .comment .comment.* )
    }
}
