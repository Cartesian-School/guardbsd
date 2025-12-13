/* 
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: boot_stub
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Wejście syscalls via int 0x80: zapisuje GPR i wraca iretq.
 */

.section .text
.global syscall_int80_entry
.type syscall_int80_entry, @function

/* extern "C" u64 syscall_dispatch_trap(u64 rax, u64 rdi, u64 rsi, u64 rdx); */

syscall_int80_entry:
    /* Interrupt entry already pushed RIP, CS, RFLAGS, RSP, SS */

    /* Save general-purpose registers */
    push %rax
    push %rbx
    push %rcx
    push %rdx
    push %rsi
    push %rdi
    push %rbp
    push %r8
    push %r9
    push %r10
    push %r11
    push %r12
    push %r13
    push %r14
    push %r15

    /* Call Rust dispatcher: args (rax, rdi, rsi, rdx) */
    mov 15*8(%rsp), %rdi   /* rax at top of saved regs */
    mov 10*8(%rsp), %rsi   /* original rdi */
    mov 9*8(%rsp), %rdx    /* original rsi */
    mov 7*8(%rsp), %rcx    /* original rdx */
    call syscall_dispatch_trap

    /* Return value already in RAX */

    /* Restore registers */
    pop %r15
    pop %r14
    pop %r13
    pop %r12
    pop %r11
    pop %r10
    pop %r9
    pop %r8
    pop %rbp
    pop %rdi
    pop %rsi
    pop %rdx
    pop %rcx
    pop %rbx
    pop %rax

    /* Detection-only signal hook: call into Rust before iretq */
    call syscall_signal_check

    iretq

.size syscall_int80_entry, . - syscall_int80_entry
