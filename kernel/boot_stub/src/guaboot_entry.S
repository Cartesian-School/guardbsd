/* 
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: boot_stub
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Wejście jądra wywoływane przez GuaBoot (bez multiboot).
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

    /* Ultra-early serial bring-up and breadcrumb: confirm we reached _start */
    call init_serial_early
    movb $'S', %al
    call write_breadcrumb

    /* Patch TSS descriptor base into GDT before loading */
    lea tss64(%rip), %rax
    movw %ax, gdt64_tss_base0(%rip)
    shr $16, %rax
    movb %al, gdt64_tss_base1(%rip)
    shr $8, %rax
    movb %al, gdt64_tss_base2(%rip)
    shr $8, %rax
    movl %eax, gdt64_tss_base3(%rip)

    /* Load GDT with user segments and TSS descriptor */
    lgdt gdt64_ptr(%rip)

    /* Install minimal early IDT for diagnostics (GPF/PGF/DF) */
    lea early_idt_ptr(%rip), %rax
    lidt (%rax)

    /* Set up stack early so we can call helpers safely */
    lea stack_top(%rip), %rsp
    mov %rsp, %rbp
    movb $'T', %al
    call write_breadcrumb

    /* Initialize TSS.RSP0 */
    lea tss_stack_top(%rip), %rax
    mov %rax, tss64 + 4(%rip)       /* RSP0 */

    /* Load TSS */
    mov $0x28, %ax                  /* TSS selector (5th entry, index=5 -> 0x28) */
    ltr %ax
    movb $1, tss_ready_flag(%rip)

    /* Save bootinfo pointer */
    mov %edi, guaboot_magic(%rip)
    mov %rsi, guaboot_bootinfo_ptr(%rip)
    movb $'I', %al
    call write_breadcrumb

    /* Validate CPU is really in long mode (LMA set in IA32_EFER) */
    call check_long_mode
    movb $'L', %al
    call write_breadcrumb
    
    /* Zero BSS section */
    lea __bss_start(%rip), %rdi
    lea __bss_end(%rip), %rcx
    sub %rdi, %rcx
    xor %rax, %rax
    rep stosb
    
    /* Call Rust kernel entry */
    movb $'G', %al
    call write_breadcrumb
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

/* Dedicated ring-0 stack for privilege transitions */
.align 16
tss_stack_bottom:
    .skip 8192
.global tss_stack_top
tss_stack_top:

/* BSS markers */
.global __bss_start
.global __bss_end
__bss_start = .
__bss_end = .

/* TSS structure (64-bit) */
.section .data
.align 16
.global tss64
tss64:
    .long 0              /* Reserved */
    .quad 0              /* RSP0 */
    .quad 0              /* RSP1 */
    .quad 0              /* RSP2 */
    .long 0
    .long 0              /* Reserved */
    .quad 0              /* IST1 */
    .quad 0              /* IST2 */
    .quad 0              /* IST3 */
    .quad 0              /* IST4 */
    .quad 0              /* IST5 */
    .quad 0              /* IST6 */
    .quad 0              /* IST7 */
    .long 0              /* Reserved */
    .long 0
    .word 0              /* IO map base */
.set tss64_end, . - tss64

.global tss_ready_flag
tss_ready_flag:
    .byte 0
    .byte 0,0,0

/* GDT with TSS descriptor */
.align 8
.global gdt64
gdt64:
    .quad 0x0000000000000000                  /* Null */
    .quad 0x00AF9A000000FFFF                  /* Kernel code */
    .quad 0x00AF92000000FFFF                  /* Kernel data */
    .quad 0x00AFFA000000FFFF                  /* User code */
    .quad 0x00AFF2000000FFFF                  /* User data */
    /* TSS descriptor (16 bytes) */
gdt64_tss_descriptor:
    .word 0x0067                               /* TSS limit */
gdt64_tss_base0:
    .word 0
gdt64_tss_base1:
    .byte 0
    .byte 0x89                                  /* Available 64-bit TSS */
gdt64_tss_base2:
    .byte 0
    .byte 0x00
gdt64_tss_base3:
    .long 0
    .long 0

.global gdt64_ptr
gdt64_ptr:
    .word gdt64_end - gdt64 - 1
    .quad gdt64
gdt64_end:

/* Early serial helpers and CPU mode validation */
.section .text

init_serial_early:
    /* Configure COM1: 9600 8N1, enable FIFO */
    movb $0x00, %al
    movw $COM1 + 1, %dx
    outb %al, %dx
    movb $0x80, %al
    movw $COM1 + 3, %dx
    outb %al, %dx
    movb $0x03, %al           /* 38400 divisor -> 9600 baud with base clk */
    movw $COM1 + 0, %dx
    outb %al, %dx
    movb $0x00, %al
    movw $COM1 + 1, %dx
    outb %al, %dx
    movb $0x03, %al           /* 8N1 */
    movw $COM1 + 3, %dx
    outb %al, %dx
    movb $0xC7, %al           /* Enable FIFO, clear, 14-byte threshold */
    movw $COM1 + 2, %dx
    outb %al, %dx
    movb $0x0B, %al           /* IRQs off, DTR/RTS set */
    movw $COM1 + 4, %dx
    outb %al, %dx
    ret

write_serial_string:
    /* rsi: pointer to NUL-terminated string */
    push %rax
    push %rdx
1:
    movzbq (%rsi), %rax
    test %al, %al
    je 2f
    movw $COM1 + 5, %dx
3:  inb %dx, %al
    test $0x20, %al
    jz 3b
    movzbq (%rsi), %rax
    movw $COM1, %dx
    outb %al, %dx
    inc %rsi
    jmp 1b
2:
    pop %rdx
    pop %rax
    ret

check_long_mode:
    /* Read IA32_EFER (0xC0000080), check LMA (bit 10) */
    movl $0xC0000080, %ecx
    rdmsr
    testl $0x400, %eax        /* Bit 10 set => long mode active */
    jnz .long_ok

    lea msg_long_mode_fail(%rip), %rsi
    call write_serial_string
.panic_loop:
    cli
    hlt
    jmp .panic_loop

.long_ok:
    lea msg_long_mode_ok(%rip), %rsi
    call write_serial_string
    ret

.section .rodata
.align 8
msg_long_mode_ok:
    .asciz "[OK] CPU is in 64-bit long mode\n"
msg_long_mode_fail:
    .asciz "Bootloader failed to enter 64-bit long mode\n"

.equ COM1, 0x3F8

/* Early breadcrumb buffer (not in .bss so it survives zeroing) */
.section .data
.align 8
.global early_trace
early_trace:
    .space 64
.global early_trace_idx
early_trace_idx:
    .quad 0

.section .text
/* Early breadcrumb buffer (not in .bss so it survives zeroing) */
.section .data
.align 8
.global early_trace
early_trace:
    .space 64
.global early_trace_idx
early_trace_idx:
    .quad 0

.section .text

/* Record a breadcrumb: store in memory buffer and emit to COM1 (best-effort). */
write_breadcrumb:
    push %rax
    push %rbx
    push %rcx
    push %rdx
    mov %al, %bl
    movl early_trace_idx(%rip), %ecx
    cmp $64, %ecx
    jae 1f
    lea early_trace(%rip), %rdx
    mov %bl, (%rdx,%rcx,1)
    inc %ecx
    mov %ecx, early_trace_idx(%rip)
1:
    movw $COM1 + 5, %dx
2:  inb %dx, %al
    test $0x20, %al
    jz 2b
    mov %bl, %al
    movw $COM1, %dx
    outb %al, %dx
    pop %rdx
    pop %rcx
    pop %rbx
    pop %rax
    ret

/* Simple hex printer for diagnostics: prints 0x followed by 16 hex digits from RDI */
early_puthex64:
    push %rax
    push %rbx
    push %rcx
    push %rdx
    mov %rdi, %rax
    lea hex_prefix(%rip), %rsi
    call write_serial_string
    mov $16, %rcx
1:
    mov %rax, %rbx
    and $0xF, %rbx
    shr $4, %rax
    movzbq hex_digits(%rbx,%rip), %rbx
    mov %rbx, %rdx
    movw $COM1 + 5, %dx
2:  inb %dx, %al
    test $0x20, %al
    jz 2b
    mov %bl, %al
    movw $COM1, %dx
    outb %al, %dx
    loop 1b
    movb $'\n', %al
    movw $COM1 + 5, %dx
3:  inb %dx, %al
    test $0x20, %al
    jz 3b
    movw $COM1, %dx
    outb %al, %dx
    pop %rdx
    pop %rcx
    pop %rbx
    pop %rax
    ret

/* Exception handlers */
.global early_gp_handler
.global early_pf_handler
.global early_df_handler

early_gp_handler:
    push %rbp
    mov %rsp, %rbp
    lea msg_gp(%rip), %rsi
    call write_serial_string
    mov %rdi, %rax
    lea msg_err(%rip), %rsi
    call write_serial_string
    mov (%rbp), %rdi
    call early_puthex64
    lea msg_rip(%rip), %rsi
    call write_serial_string
    mov 8(%rbp), %rdi
    call early_puthex64
    lea msg_rsp(%rip), %rsi
    call write_serial_string
    mov 32(%rbp), %rdi
    call early_puthex64
    jmp .Lhalt_forever

early_pf_handler:
    push %rbp
    mov %rsp, %rbp
    lea msg_pf(%rip), %rsi
    call write_serial_string
    lea msg_err(%rip), %rsi
    call write_serial_string
    mov (%rbp), %rdi
    call early_puthex64
    mov %cr2, %rdi
    lea msg_cr2(%rip), %rsi
    call write_serial_string
    call early_puthex64
    lea msg_rip(%rip), %rsi
    call write_serial_string
    mov 8(%rbp), %rdi
    call early_puthex64
    lea msg_rsp(%rip), %rsi
    call write_serial_string
    mov 32(%rbp), %rdi
    call early_puthex64
    jmp .Lhalt_forever

early_df_handler:
    push %rbp
    mov %rsp, %rbp
    lea msg_df(%rip), %rsi
    call write_serial_string
    jmp .Lhalt_forever

.Lhalt_forever:
    cli
    .Lloop:
        pause
        jmp .Lloop

.section .rodata
.align 8
hex_prefix:
    .asciz "0x"
hex_digits:
    .ascii "0123456789abcdef"
msg_gp: .asciz "[EARLY] #GP\n"
msg_pf: .asciz "[EARLY] #PF\n"
msg_df: .asciz "[EARLY] #DF\n"
msg_err: .asciz " err="
msg_rip: .asciz " rip="
msg_rsp: .asciz " rsp="
msg_cr2: .asciz " cr2="
