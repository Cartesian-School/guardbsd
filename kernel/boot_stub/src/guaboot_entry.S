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

    /* Set up stack early so we can call helpers safely */
    lea stack_top(%rip), %rsp
    mov %rsp, %rbp

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

    /* Early serial for validation output */
    call init_serial_early

    /* Validate CPU is really in long mode (LMA set in IA32_EFER) */
    call check_long_mode
    
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

/* Dedicated ring-0 stack for privilege transitions */
.align 16
tss_stack_bottom:
    .skip 8192
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
    .word (tss64_end - tss64 - 1)
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
