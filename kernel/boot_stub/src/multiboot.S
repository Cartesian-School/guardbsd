/* Multiboot header - BSD 3-Clause License */
/* MUST be in the first 8KB of the file for GRUB to find it */
.section .text
.align 4

#define MULTIBOOT_MAGIC 0x1BADB002
#define MULTIBOOT_FLAGS 0x00000003

.global multiboot_header
multiboot_header:
    .long MULTIBOOT_MAGIC
    .long MULTIBOOT_FLAGS
    .long -(MULTIBOOT_MAGIC + MULTIBOOT_FLAGS)
    
/* Jump to the real entry point (defined in main.rs) */
.global _multiboot_start
_multiboot_start:
    jmp _start
