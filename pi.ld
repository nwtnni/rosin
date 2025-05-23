/* https://forums.raspberrypi.com/viewtopic.php?t=328000 */
__PHYSICAL_ADDRESS = 0x80000;

ENTRY(__PHYSICAL_ADDRESS)

PHDRS {
    segment_code PT_LOAD FLAGS(5);
}

SECTIONS {
    . = __PHYSICAL_ADDRESS;

    .text : {
        KEEP(*(.text._start))
        *( .text .text.* )
    } :segment_code
}
