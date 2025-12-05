/* GuardBSD Kernel Entry Point
 * BSD 3-Clause License
 * Copyright (c) 2025, GuardBSD Project
 *
 * FreeBSD-compatible entry (NO multiboot)
 * Called by GuaBoot loader with bootinfo structure
 */

.code64
.section .text.entry, "ax"
.global _start
.type _start, @function

_start:
    /* Entry from GuaBoot:
     * %rdi: GBSD_MAGIC (0x42534447 = "GBSD")
     * %rsi: pointer to bootinfo structure
     */
    
    cli                         /* Disable interrupts */
    cld                         /* Clear direction flag */
    
    /* Save bootinfo pointer */
    mov %edi, guaboot_magic(%rip)
    mov %rsi, guaboot_bootinfo_ptr(%rip)
    
    /* Set up stack */
    lea stack_top(%rip), %rsp
    mov %rsp, %rbp
    
    /* Zero BSS section */
    lea __bss_start(%rip), %rdi
    lea __bss_end(%rip), %rcx
    sub %rdi, %rcx
    xor %rax, %rax
    rep stosb
    
    /* Call Rust kernel entry */
    call guardbsd_main
    
    /* Kernel returned - halt */
.Lhalt:
    cli
    hlt
    jmp .Lhalt

.size _start, . - _start

/* Boot data */
.section .data
.global guaboot_magic
.global guaboot_bootinfo_ptr

guaboot_magic:
    .long 0

guaboot_bootinfo_ptr:
    .long 0

/* Stack */
.section .bss
.align 16
stack_bottom:
    .skip 65536  /* 64KB kernel stack */
stack_top:

/* BSS markers */
.global __bss_start
.global __bss_end
__bss_start = .
__bss_end = .

