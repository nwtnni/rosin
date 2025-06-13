__TEXT = 0x8 << 16;
__TEXT_LO = 1 << 28;
__ELF = 1 << 29;

ENTRY(__TEXT)

PHDRS {
    segment_ro PT_LOAD FLAGS(4);
    segment_rx PT_LOAD FLAGS(5);
    segment_rw PT_LOAD FLAGS(6);
}

SECTIONS {
    . = __TEXT_LO - __TEXT;

    __STACK_LO = .;
    . += __TEXT;
    __STACK_HI = .;

    .text : AT(__TEXT_LO) {
        KEEP(*(.text.boot))
        KEEP(*(.text.start))
        *( .text .text.* )
    } :segment_rx

    .got : ALIGN(16) {
        *( .got .got.* )
    } :segment_ro
    ASSERT(SIZEOF(.got) == 0, "Unexpected relocation")

    .rodata : {
        *( .rodata .rodata.* )
    } :segment_ro

    .data : ALIGN(16) {
        *( .data .data.* )
    } :segment_rw
    ASSERT(SIZEOF(.data) == 0, "Unexpected data")

    .bss (NOLOAD) : {
        *( .bss .bss.* )
    } :segment_rw
    ASSERT(SIZEOF(.bss) == 0, "Unexpected bss")

    /DISCARD/ : {
        *( .comment .comment.* )
    }

    . = ALIGN(16);
    __TEXT_HI = .;
}
