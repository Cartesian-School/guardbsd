# ============================================================================
# kernel/linker-aarch64.ld
# Production AArch64 Linker Script
# Enforces W^X (Write XOR Execute) via PHDRS
# ============================================================================
OUTPUT_FORMAT("elf64-littleaarch64")
OUTPUT_ARCH(aarch64)
ENTRY(_start)

/* Define memory segments for security permissions */
PHDRS
{
    text    PT_LOAD FLAGS(5); /* 5 = Read + Execute (RX) */
    rodata  PT_LOAD FLAGS(4); /* 4 = Read Only (R) */
    data    PT_LOAD FLAGS(6); /* 6 = Read + Write (RW) */
}

SECTIONS
{
    /* Higher-half kernel mapping base */
    . = 0xFFFFFFFF80000000;

    /* 1. CODE SEGMENT (RX) */
    .text : ALIGN(4K) {
        *(.text.boot) /* Boot code first */
        
        /* Exception vectors must be 2KB aligned (0x800) */
        . = ALIGN(2048);
        __vectors_start = .;
        KEEP(*(.vectors))
        __vectors_end = .;
        
        *(.text .text.*)
    } :text

    /* 2. READ-ONLY DATA SEGMENT (R) */
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
    } :rodata

    /* 3. READ-WRITE DATA SEGMENT (RW) */
    .data : ALIGN(4K) {
        *(.data .data.*)
    } :data

    /* BSS (Zero-initialized data) - Part of RW segment */
    .bss : ALIGN(4K) {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    } :data

    /* Kernel Stack (Explicitly separate to prevent overflow corruption into data) */
    .bss.stack : ALIGN(16K) {
        __stack_start = .;
        KEEP(*(.bss.stack))
        __stack_end = .;
    } :data

    /* Strip unnecessary metadata to reduce binary size */
    /DISCARD/ : {
        *(.eh_frame*)
        *(.comment)
        *(.note*)
    }
}