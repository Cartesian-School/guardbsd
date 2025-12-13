/* 
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: boot_stub
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Nagłówek multiboot – musi być w pierwszych 8KB dla GRUB.
 */
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
