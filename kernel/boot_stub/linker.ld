OUTPUT_FORMAT(elf32-i386)
OUTPUT_ARCH(i386)
ENTRY(multiboot_header)

SECTIONS
{
    . = 0x00100000;

    /* Multiboot header MUST be at the very beginning of .text */
    .text : ALIGN(4K) {
        *(.text.multiboot_header)
        *(.text .text.*)
    }

    .rodata ALIGN(4K) : {
        *(.rodata .rodata.*)
    }

    .data ALIGN(4K) : {
        *(.data .data.*)
    }

    .bss ALIGN(4K) : {
        *(.bss .bss.*)
        *(COMMON)
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
    }
}
