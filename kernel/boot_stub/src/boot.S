/* 
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: boot_stub
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Minimalny punkt wejścia dla GuaBoot (bez Multiboot).
 */

.code32                                     /* Force 32-bit code */
.section .boot, "ax"
.align 4
.global _boot_entry

_boot_entry:
    cli                                     /* Disable interrupts */
    
    /* Set up stack */
    mov $stack_top, %esp
    mov %esp, %ebp
    
    /* We are entered directly by GuaBoot (no multiboot payload) */
    xor %eax, %eax
    xor %ebx, %ebx
    xor %ecx, %ecx
    xor %edx, %edx
    
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
