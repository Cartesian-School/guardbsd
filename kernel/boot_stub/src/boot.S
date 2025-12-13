/* 
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: boot_stub
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Punkt wejścia z nagłówkiem Multiboot; GRUB musi znaleźć go w pierwszych 8KB.
 */

.code32                                     /* Force 32-bit code */
.section .boot, "ax"
.align 4
.global _boot_entry

/* Multiboot header - MUST be in first 8KB */
multiboot_header:
    .long 0x1BADB002                        /* Magic number */
    .long 0x00000003                        /* Flags: align modules, want memory map */
    .long -(0x1BADB002 + 0x00000003)       /* Checksum */

_boot_entry:
    cli                                     /* Disable interrupts */
    
    /* Set up stack */
    mov $stack_top, %esp
    mov %esp, %ebp
    
    /* Save multiboot info */
    push %ebx                               /* Multiboot info pointer */
    push %eax                               /* Multiboot magic */
    
    /* Jump to Rust entry point */
    call _start
    
    /* Should never return, but just in case */
halt:
    cli
    hlt
    jmp halt

.section .bss
.align 16
stack_bottom:
    .skip 16384  /* 16KB stack */
stack_top:
