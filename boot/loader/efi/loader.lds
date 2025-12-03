/* GuardBSD UEFI Loader Linker Script
 * BSD 3-Clause License
 */

OUTPUT_FORMAT("elf64-x86-64")
OUTPUT_ARCH(i386:x86-64)
ENTRY(efi_main)

SECTIONS {
    . = 0;
    ImageBase = .;
    
    .text : {
        *(.text .text.*)
    }
    
    .data : {
        *(.data .data.*)
        *(.rodata .rodata.*)
    }
    
    .dynamic : { *(.dynamic) }
    .dynsym : { *(.dynsym) }
    .rel : { *(.rel*) }
    .reloc : { *(.reloc) }
    
    /DISCARD/ : {
        *(.comment)
        *(.note*)
    }
}
