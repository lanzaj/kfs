ENTRY(start)

SECTIONS {
    .gdt 0x00000800 :
    {
        *(.gdt)
        . = ALIGN(0x800);
    }

    . = 1M;

    .boot :
    {
        /* ensure that the multiboot header is at the beginning */
        KEEP(*(.multiboot_header))
    }

    .text :
    {
        *(.text)
    }

}