/* 
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: kernel_interrupt
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Handler IRQ timera (x86_64 asm).
 */

.section .text
.global timer_irq_handler

// IRQ0 (timer) handler (64-bit)
timer_irq_handler:
    pushq %rax
    pushq %rbx
    pushq %rcx
    pushq %rdx
    pushq %rsi
    pushq %rdi
    pushq %rbp
    pushq %r8
    pushq %r9
    pushq %r10
    pushq %r11
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15
    
    call timer_interrupt_handler
    
    // Send EOI to PIC
    mov $0x20, %al
    out %al, $0x20
    
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %r11
    popq %r10
    popq %r9
    popq %r8
    popq %rbp
    popq %rdi
    popq %rsi
    popq %rdx
    popq %rcx
    popq %rbx
    popq %rax
    iretq
