/* kernel/linker-riscv64.ld
 * Minimal linker script for QEMU virt + OpenSBI
 * OpenSBI typically jumps to 0x80200000 for the payload.
 */
OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
  . = 0x80200000;

  .text : ALIGN(16) {
    KEEP(*(.text.boot))
    *(.text .text.*)
  }

  .rodata : ALIGN(16) {
    *(.rodata .rodata.*)
  }

  .data : ALIGN(16) {
    *(.data .data.*)
  }

  .bss : ALIGN(16) {
    __bss_start = .;
    *(.bss .bss.*)
    *(COMMON)
    __bss_end = .;
  }

  /* Simple stack at the end of the image */
  . = ALIGN(16);
  __stack_bottom = .;
  . += 64K;              /* 64 KiB temporary bootstrap stack */
  __stack_top = .;

  /DISCARD/ : {
    *(.eh_frame .comment .note .note.*)
  }
}
