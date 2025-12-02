# ============================================================================
# kernel/linker-x86_64.ld
# Production x86_64 Linker Script
# Enforces W^X and supports Limine Boot Protocol
# ============================================================================
OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

PHDRS
{
    text    PT_LOAD FLAGS(5); /* RX */
    rodata  PT_LOAD FLAGS(4); /* R  */
    data    PT_LOAD FLAGS(6); /* RW */
}

SECTIONS
{
    /* Higher-half kernel mapping base */
    . = 0xFFFFFFFF80000000;

    /* 1. CODE SEGMENT (RX) */
    .text : ALIGN(4K) {
        KEEP(*(.text.boot))
        *(.text .text.*)
    } :text

    /* 2. READ-ONLY DATA SEGMENT (R) */
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        
        /* Limine requests usually live in rodata or data */
        . = ALIGN(8);
        KEEP(*(.limine_reqs))
    } :rodata

    /* 3. READ-WRITE DATA SEGMENT (RW) */
    .data : ALIGN(4K) {
        *(.data .data.*)
    } :data

    /* BSS (Zero-initialized data) */
    .bss : ALIGN(4K) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    } :data

    /* Kernel Stack */
    .bss.stack : ALIGN(16K) {
        __stack_start = .;
        KEEP(*(.bss.stack))
        __stack_end = .;
    } :data

    /DISCARD/ : {
        *(.eh_frame*)
        *(.comment)
        *(.note*)
    }
}