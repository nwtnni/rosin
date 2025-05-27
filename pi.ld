/* https://forums.raspberrypi.com/viewtopic.php?t=328000 */
__PHYSICAL_ADDRESS = 0x80000;

ENTRY(__PHYSICAL_ADDRESS)

PHDRS {
    segment_stack PT_LOAD FLAGS(6);
    segment_code PT_LOAD FLAGS(5);
    segment_rodata PT_LOAD FLAGS(4);
    segment_data PT_LOAD FLAGS(6);
    segment_bss PT_LOAD FLAGS(6);
}

SECTIONS {
    .stack (NOLOAD) : ALIGN(16) {
        __STACK_LO = .;
        . += __PHYSICAL_ADDRESS;
        __STACK_HI = .;
    } :segment_stack

    . = __PHYSICAL_ADDRESS;

    .text : ALIGN(4) {
        HIDDEN(__TEXT_LO = .);
        KEEP(*(.text._start))
        *( .text .text.* )
    } :segment_code

    .rodata : ALIGN(16) {
        *( .rodata .rodata.* )
    } :segment_rodata

    .data : ALIGN(16) {
        *( .data .data.* )
    } :segment_data

    .bss (NOLOAD) : ALIGN(16) {
        __BSS_LO = .;
        *( .bss .bss.* )
        . = ALIGN(16);
        __BSS_HI = .;
    } :segment_bss

    .got : {
        *( .got .got.* )
    }

    ASSERT(SIZEOF(.got) == 0, "Unexpected relocation")

    /DISCARD/ : {
        *( .comment .comment.* )
    }
}
