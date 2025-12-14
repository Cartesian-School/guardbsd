/* kernel/boot_stub/linker.ld */
/* Linker script for GuardBSD kernel boot stub (x86_64) */
/* Copyright © 2025 Cartesian School. */
/* BSD-3-Clause */

OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(x86_64)
ENTRY(_start)

/* Program headers (PT_LOAD segments) - future-proof for ELF loaders */
PHDRS
{
    code PT_LOAD FLAGS(5);  /* R + X */
    data PT_LOAD FLAGS(6);  /* R + W */
}

SECTIONS
{
    /* Kernel base (physical/virtual, depending on your boot model) */
    . = 0x00200000;

    __image_start = .;
    PROVIDE(__image_start = __image_start);

    /* ------------------------------------------------------------------ */
    /* Text - entry point MUST be first                                    */
    /* ------------------------------------------------------------------ */
    .text ALIGN(4K) : {
        KEEP(*(.text.entry))
        KEEP(*(.text.entry.*))
        *(.text .text.*)
    } :code

    /* ------------------------------------------------------------------ */
    /* Read-only data                                                      */
    /* ------------------------------------------------------------------ */
    .rodata ALIGN(4K) : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } :code

    /* ------------------------------------------------------------------ */
    /* Initialized data                                                    */
    /* ------------------------------------------------------------------ */
    .data ALIGN(4K) : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    } :data

    /* ------------------------------------------------------------------ */
    /* BSS (zeroed at runtime)                                             */
    /* NOLOAD => no file bytes allocated for BSS                           */
    /* Single source of truth for __bss_start/__bss_end                    */
    /* ------------------------------------------------------------------ */
    .bss (NOLOAD) ALIGN(4K) : {
        __bss_start = .;
        PROVIDE(__bss_start = __bss_start);

        *(.bss .bss.*)
        *(COMMON)

        /* Helpful for stacks/IDT/TSS alignment expectations */
        . = ALIGN(16);

        __bss_end = .;
        PROVIDE(__bss_end = __bss_end);
    } :data

    __image_end = .;
    PROVIDE(__image_end = __image_end);

    /* Optional safety check (enable if desired) */
    /* ASSERT(__image_end - __image_start <= 32M, "Kernel image exceeds 32 MiB") */

    /* ------------------------------------------------------------------ */
    /* Discard sections not wanted in a freestanding kernel image          */
    /* ------------------------------------------------------------------ */
    /DISCARD/ : {
        *(.eh_frame .eh_frame.*)
        *(.gcc_except_table .gcc_except_table.*)

        *(.note .note.*)
        *(.comment .comment.*)
        *(.interp)

        *(.dynamic .dyn*)
        *(.got .got.*)

        *(.gnu.*)
        *(.rela.* .rel.*)
    }
}
