/* Multiboot header - BSD 3-Clause License */
.section .multiboot
.align 4

#define MULTIBOOT_MAGIC 0x1BADB002
#define MULTIBOOT_FLAGS 0x00000003

multiboot_header:
    .long MULTIBOOT_MAGIC
    .long MULTIBOOT_FLAGS
    .long -(MULTIBOOT_MAGIC + MULTIBOOT_FLAGS)
