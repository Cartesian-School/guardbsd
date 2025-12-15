/* kernel/boot_stub/src/interrupt/exception_stubs.S
 *
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: boot_stub
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 *  License: BSD-3-Clause
 *
 * Minimal x86_64 exception stubs for boot_stub IDT (vectors 0..31).
 *
 * Convention:
 * - write_char_bl expects character in BL (explicit, unambiguous).
 * - exc_stub_N pushes vector number and jumps to common_exception_handler.
 *
 * Handler additionally supports CPL change (user->kernel) and prints old_rsp/old_ss.
 */

.code64
.section .text
.align 16

.equ COM1, 0x3F8

/* ------------------------------------------------------------
 * Serial helpers (COM1)
 * ---------------------------------------------------------- */

write_char_bl:
    /* BL = char */
    push %rax
    push %rdx
1:
    movw $(COM1 + 5), %dx           /* LSR */
    inb  %dx, %al
    testb $0x20, %al                /* THR empty */
    jz   1b
    movw $COM1, %dx
    mov  %bl, %al
    outb %al, %dx
    pop  %rdx
    pop  %rax
    ret

write_str:
    /* RSI -> NUL-terminated string */
    push %rax
    push %rbx
    push %rdx
1:
    movb (%rsi), %bl
    testb %bl, %bl
    je 2f
    inc %rsi
    call write_char_bl
    jmp 1b
2:
    pop %rdx
    pop %rbx
    pop %rax
    ret

write_hex64:
    /* RDI = value, prints 16 hex digits (fixed width) */
    push %rax
    push %rbx
    push %rcx
    push %rdx
    push %rsi

    lea hex_digits(%rip), %rsi
    mov $60, %ecx
1:
    mov %rdi, %rax
    shr %cl, %rax
    and $0xF, %rax
    movzbq (%rsi,%rax,1), %rbx      /* BL = digit */
    call write_char_bl
    sub $4, %ecx
    jns 1b

    pop %rsi
    pop %rdx
    pop %rcx
    pop %rbx
    pop %rax
    ret

write_hex16:
    /* DI = value, prints 4 hex digits */
    push %rax
    push %rbx
    push %rcx
    push %rdx
    push %rsi
    push %rdi

    lea hex_digits(%rip), %rsi
    mov $12, %ecx
1:
    mov %rdi, %rax
    shr %cl, %rax
    and $0xF, %rax
    movzbq (%rsi,%rax,1), %rbx
    call write_char_bl
    sub $4, %ecx
    jns 1b

    pop %rdi
    pop %rsi
    pop %rdx
    pop %rcx
    pop %rbx
    pop %rax
    ret

write_nl:
    push %rbx
    movb $'\n', %bl
    call write_char_bl
    pop %rbx
    ret

/* ------------------------------------------------------------
 * Common exception handler
 * ---------------------------------------------------------- */

.global common_exception_handler
.type common_exception_handler, @function
common_exception_handler:
    cli
    cld

    /* r12 = frame base (points to VEC pushed by stub) */
    mov %rsp, %r12

    /* Save volatile regs (we won't return, but keep output stable) */
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
    push %r13
    push %r14
    push %r15

    /* vec */
    movq 0(%r12), %r13

    /* Detect has_err by probing CS positions. */
    movq 16(%r12), %rax             /* maybe CS (noerr) */
    movq 24(%r12), %rbx             /* maybe CS (err) */
    movw %ax, %ax
    movw %bx, %bx

    mov $0, %r14d                   /* r14d = has_err */

    cmpw $0x08, %ax
    je  .Lnoerr
    cmpw $0x1B, %ax
    je  .Lnoerr

    cmpw $0x08, %bx
    je  .Lerr
    cmpw $0x1B, %bx
    je  .Lerr

.Lerr:
    mov $1, %r14d
    jmp .Llayout_done

.Lnoerr:
    mov $0, %r14d

.Llayout_done:
    /* Extract err/rip/cs/rflags */
    cmp $0, %r14d
    je  .Lextract_noerr

.Lextract_err:
    movq 8(%r12),  %r15             /* err */
    movq 16(%r12), %r11             /* rip */
    movq 24(%r12), %r10             /* cs */
    movq 32(%r12), %r9              /* rflags */
    jmp .Lextract_done

.Lextract_noerr:
    xor %r15, %r15
    movq 8(%r12),  %r11
    movq 16(%r12), %r10
    movq 24(%r12), %r9

.Lextract_done:
    mov %cr2, %r8                   /* cr2 */

    /* from_user? (CS == 0x1B) */
    xor %rbx, %rbx                  /* rbx = 0/1 flag */
    cmpw $0x1B, %r10w
    jne .Lno_user
    mov $1, %rbx
.Lno_user:

    /* old_rsp/old_ss if from_user, else 0 */
    xor %rax, %rax                  /* old_rsp */
    xor %rcx, %rcx                  /* old_ss */
    cmp $0, %rbx
    je  .Luser_done

    cmp $0, %r14d
    je  .Luser_noerr

.Luser_err:
    movq 40(%r12), %rax             /* old_rsp */
    movq 48(%r12), %rcx             /* old_ss */
    jmp .Luser_done

.Luser_noerr:
    movq 32(%r12), %rax
    movq 40(%r12), %rcx

.Luser_done:

    /* Print */
    lea msg_exc(%rip), %rsi
    call write_str

    lea msg_vec(%rip), %rsi
    call write_str
    mov %r13, %rdi
    call write_hex64

    lea msg_err(%rip), %rsi
    call write_str
    mov %r15, %rdi
    call write_hex64

    lea msg_rip(%rip), %rsi
    call write_str
    mov %r11, %rdi
    call write_hex64

    lea msg_cs(%rip), %rsi
    call write_str
    mov %r10, %rdi
    call write_hex16

    lea msg_rflags(%rip), %rsi
    call write_str
    mov %r9, %rdi
    call write_hex64

    lea msg_rsp(%rip), %rsi
    call write_str
    mov %r12, %rdi
    call write_hex64

    lea msg_cr2(%rip), %rsi
    call write_str
    mov %r8, %rdi
    call write_hex64

    cmp $0, %rbx
    je  .Lprint_done

    lea msg_old_rsp(%rip), %rsi
    call write_str
    mov %rax, %rdi
    call write_hex64

    lea msg_old_ss(%rip), %rsi
    call write_str
    mov %rcx, %rdi
    call write_hex16

.Lprint_done:
    call write_nl

.Lhalt:
    hlt
    jmp .Lhalt

.size common_exception_handler, . - common_exception_handler

/* ------------------------------------------------------------
 * Per-vector stubs: exc_stub_0 .. exc_stub_31
 * ---------------------------------------------------------- */

.macro EXC_STUB vec
    .global exc_stub_\vec
    .type exc_stub_\vec, @function
exc_stub_\vec:
    pushq $\vec
    jmp common_exception_handler
    .size exc_stub_\vec, . - exc_stub_\vec
.endm

EXC_STUB 0
EXC_STUB 1
EXC_STUB 2
EXC_STUB 3
EXC_STUB 4
EXC_STUB 5
EXC_STUB 6
EXC_STUB 7
EXC_STUB 8
EXC_STUB 9
EXC_STUB 10
EXC_STUB 11
EXC_STUB 12
EXC_STUB 13
EXC_STUB 14
EXC_STUB 15
EXC_STUB 16
EXC_STUB 17
EXC_STUB 18
EXC_STUB 19
EXC_STUB 20
EXC_STUB 21
EXC_STUB 22
EXC_STUB 23
EXC_STUB 24
EXC_STUB 25
EXC_STUB 26
EXC_STUB 27
EXC_STUB 28
EXC_STUB 29
EXC_STUB 30
EXC_STUB 31

.section .rodata
.align 8
hex_digits:     .ascii "0123456789abcdef"
msg_exc:        .asciz "[EXC]"
msg_vec:        .asciz " vec=0x"
msg_err:        .asciz " err=0x"
msg_rip:        .asciz " rip=0x"
msg_cs:         .asciz " cs=0x"
msg_rflags:     .asciz " rflags=0x"
msg_rsp:        .asciz " rsp=0x"
msg_cr2:        .asciz " cr2=0x"
msg_old_rsp:    .asciz " old_rsp=0x"
msg_old_ss:     .asciz " old_ss=0x"
