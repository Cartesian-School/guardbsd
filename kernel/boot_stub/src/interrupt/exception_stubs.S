/* kernel/boot_stub/src/interrupt/exception_stubs.S
 *
 * Minimal x86_64 exception stubs for boot_stub IDT (vectors 0..31).
 *
 * Design:
 * - Each exc_stub_N pushes the vector number and jumps to common_exception_handler.
 * - common_exception_handler prints diagnostics to COM1 and halts forever.
 *
 * Supports:
 * - Exceptions with or without error code (auto-detected by CS position).
 * - Optional CPL change stack extension (old RSP/SS) if CS == 0x1B.
 */

.code64
.section .text
.align 16

.equ COM1, 0x3F8

/* ------------------------------------------------------------
 * Serial helpers (COM1)
 * Contract: character is passed in BL.
 * ---------------------------------------------------------- */

write_char_bl:
    /* BL = char */
    push %rax
    push %rdx
1:  movw $(COM1 + 5), %dx           /* LSR */
    inb  %dx, %al
    testb $0x20, %al                /* THR empty */
    jz   1b
    movw $COM1, %dx
    mov %bl, %al
    outb %al, %dx
    pop %rdx
    pop %rax
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
    mov $60, %ecx                   /* shift count */
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
    push %rdi
    shl $48, %rdi
    shr $48, %rdi

    push %rax
    push %rbx
    push %rcx
    push %rdx
    push %rsi
    lea hex_digits(%rip), %rsi
    mov $12, %ecx                   /* shifts 12,8,4,0 */
1:
    mov %rdi, %rax
    shr %cl, %rax
    and $0xF, %rax
    movzbq (%rsi,%rax,1), %rbx
    call write_char_bl
    sub $4, %ecx
    jns 1b
    pop %rsi
    pop %rdx
    pop %rcx
    pop %rbx
    pop %rax
    pop %rdi
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

    /* Save original stack pointer (points to VEC) */
    mov %rsp, %r12

    /* Save some regs for stable output (we never return) */
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

    /* Read vector */
    movq 0(%r12), %r13              /* r13 = vec */

    /* Determine layout by inspecting CS positions.
     * No-error layout after vec:
     *   +8  RIP
     *   +16 CS
     *   +24 RFLAGS
     *   [+32 old RSP, +40 old SS if CPL change]
     *
     * Error-code layout after vec:
     *   +8  ERROR
     *   +16 RIP
     *   +24 CS
     *   +32 RFLAGS
     *   [+40 old RSP, +48 old SS if CPL change]
     */
    movq 16(%r12), %rax             /* maybe CS (noerr) */
    movq 24(%r12), %rbx             /* maybe CS (err) */
    movw %ax, %ax
    movw %bx, %bx

    mov $0, %r14d                   /* r14d = has_err? 0/1 */
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

    /* Extract:
     *   r15 = err (or 0)
     *   r11 = rip
     *   r10 = cs
     *   r9  = rflags
     *   r8  = cr2
     */
    cmp $0, %r14d
    je  .Lextract_noerr

.Lextract_err:
    movq 8(%r12),  %r15             /* error code */
    movq 16(%r12), %r11             /* RIP */
    movq 24(%r12), %r10             /* CS */
    movq 32(%r12), %r9              /* RFLAGS */
    jmp .Lextract_done

.Lextract_noerr:
    xor %r15, %r15
    movq 8(%r12),  %r11
    movq 16(%r12), %r10
    movq 24(%r12), %r9

.Lextract_done:
    mov %cr2, %r8

    /* Print header */
    lea msg_exc(%rip), %rsi
    call write_str

    /* vec */
    lea msg_vec(%rip), %rsi
    call write_str
    mov %r13, %rdi
    call write_hex64

    /* err */
    lea msg_err(%rip), %rsi
    call write_str
    mov %r15, %rdi
    call write_hex64

    /* rip */
    lea msg_rip(%rip), %rsi
    call write_str
    mov %r11, %rdi
    call write_hex64

    /* cs */
    lea msg_cs(%rip), %rsi
    call write_str
    mov %r10, %rdi
    call write_hex16

    /* rflags */
    lea msg_rflags(%rip), %rsi
    call write_str
    mov %r9, %rdi
    call write_hex64

    /* rsp(frame) */
    lea msg_rsp(%rip), %rsi
    call write_str
    mov %r12, %rdi
    call write_hex64

    /* cr2 */
    lea msg_cr2(%rip), %rsi
    call write_str
    mov %r8, %rdi
    call write_hex64

    /* If exception came from user mode (CS=0x1B), print old RSP/SS too. */
    movw %r10w, %ax
    cmpw $0x1B, %ax
    jne .Ldone_user_extra

    cmp $0, %r14d
    je  .Luser_noerr

.Luser_err:
    lea msg_oldrsp(%rip), %rsi
    call write_str
    movq 40(%r12), %rdi
    call write_hex64

    lea msg_oldss(%rip), %rsi
    call write_str
    movq 48(%r12), %rdi
    call write_hex16
    jmp .Ldone_user_extra

.Luser_noerr:
    lea msg_oldrsp(%rip), %rsi
    call write_str
    movq 32(%r12), %rdi
    call write_hex64

    lea msg_oldss(%rip), %rsi
    call write_str
    movq 40(%r12), %rdi
    call write_hex16

.Ldone_user_extra:
    call write_nl

.Lhalt:
    hlt
    jmp .Lhalt

.size common_exception_handler, . - common_exception_handler

/* ------------------------------------------------------------
 * Per-vector stubs: exc_stub_0 .. exc_stub_31
 * Each pushes vector number and jumps to common handler.
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
hex_digits:
    .ascii "0123456789abcdef"

msg_exc:     .asciz "[EXC]"
msg_vec:     .asciz " vec=0x"
msg_err:     .asciz " err=0x"
msg_rip:     .asciz " rip=0x"
msg_cs:      .asciz " cs=0x"
msg_rflags:  .asciz " rflags=0x"
msg_rsp:     .asciz " rsp=0x"
msg_cr2:     .asciz " cr2=0x"
msg_oldrsp:  .asciz " old_rsp=0x"
msg_oldss:   .asciz " old_ss=0x"
