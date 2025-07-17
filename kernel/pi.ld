/* https://forums.raspberrypi.com/viewtopic.php?t=328000 */
__KERNEL_PHYS = 0x8 << 16;
__KERNEL_OFFSET = 0xFFFFFFFF << 32;
__KERNEL_VIRT = __KERNEL_PHYS + __KERNEL_OFFSET;

ENTRY(__KERNEL_VIRT)

HIDDEN(PAGE_SIZE = 1 << 16);

PHDRS {
    segment_ro PT_LOAD FLAGS(4);
    segment_rx PT_LOAD FLAGS(5);
    segment_rw PT_LOAD FLAGS(6);
}

SECTIONS {
    . = __KERNEL_VIRT;
    __KERNEL_LO = .;

    .text : AT(__KERNEL_PHYS) {
        KEEP(*(.text.boot))
        KEEP(*(.text.start))
        *( .text .text.* )
    } :segment_rx

    . = ALIGN(PAGE_SIZE);

    .got : ALIGN(16) {
        *( .got .got.* )
    } :segment_ro

    .rodata : ALIGN(16) {
        *( .rodata .rodata.* )
    } :segment_ro

    . = ALIGN(PAGE_SIZE);

    .data : ALIGN(16) {
        *( .data .data.* )
    } :segment_rw

    .bss (NOLOAD) : ALIGN(16) {
        __BSS_LO = .;
        *( .bss .bss.* )
        . = ALIGN(16);
        __BSS_HI = .;
    } :segment_rw

    .stack (NOLOAD) : ALIGN(16) {
        __STACK_LO = .;
        . += PAGE_SIZE;
        __STACK_HI = .;
    } :segment_rw

    . = ALIGN(PAGE_SIZE);
    __KERNEL_HI = .;

    ASSERT(SIZEOF(.got) == 0, "Unexpected relocation")

    /DISCARD/ : {
        *( .comment .comment.* )
    }
}
