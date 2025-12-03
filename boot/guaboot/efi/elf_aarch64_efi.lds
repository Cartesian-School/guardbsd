OUTPUT_FORMAT("elf64-littleaarch64")
OUTPUT_ARCH(aarch64)
ENTRY(efi_main)
SECTIONS
{
  . = 0;
  .text : {
    *(.text)
  }
  .data : {
    *(.data)
    *(.data*)
  }
  .sdata : {
    *(.sdata)
  }
  .rodata : {
    *(.rodata)
    *(.rodata*)
  }
  .dynamic : {
    *(.dynamic)
  }
  .rela : {
    *(.rela)
    *(.rela*)
  }
  .reloc : {
    *(.reloc)
  }
  .bss : {
    *(.bss)
    *(.bss*)
  }
}
