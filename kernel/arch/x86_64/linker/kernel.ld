/* Project: GuardBSD Winter Saga version 1.0.0
 * Package: kernel_linker_x86_64
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Kernel linker script: link as ET_EXEC at 2 MiB physical, no PIE/RELRO.
 */

OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

KERNEL_PHYS_BASE = 0x00200000; /* 2 MiB */

PHDRS
{
  text PT_LOAD FLAGS(5);  /* R + X */
  ro   PT_LOAD FLAGS(4);  /* R */
  data PT_LOAD FLAGS(6);  /* R + W */
}

SECTIONS
{
  . = KERNEL_PHYS_BASE;

  .text ALIGN(0x1000) : {
    KEEP(*(.text._start))
    KEEP(*(.text.boot))
    *(.text .text.*)
  } :text

  .rodata ALIGN(0x1000) : {
    *(.rodata .rodata.*)
    *(.eh_frame_hdr) *(.eh_frame)
  } :ro

  .data ALIGN(0x1000) : {
    *(.data .data.*)
  } :data

  .bss ALIGN(0x1000) : {
    __bss_start = .;
    *(.bss .bss.*)
    *(COMMON)
    __bss_end = .;
  } :data

  /DISCARD/ : {
    *(.comment)
    *(.note*)
    *(.interp)
    *(.dynamic)
    *(.dynsym) *(.dynstr)
    *(.gnu.hash) *(.hash)
    *(.rela.*) *(.rel.*)
    *(.got*) *(.plt*)
  }
}
