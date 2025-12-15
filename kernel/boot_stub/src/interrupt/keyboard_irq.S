/* kernel/boot_stub/src/interrupt/keyboard_irq.S
 *
 * x86_64 IRQ1 keyboard interrupt handler stub for boot_stub.
 *
 * Responsibilities:
 * - Save all GPRs (safe for Rust call)
 * - Call Rust handler: keyboard_interrupt_handler()
 * - Send EOI to 8259 PIC (master)
 * - Restore registers and return with IRETQ
 *
 * Notes:
 * - We assume legacy PIC is remapped to 0x20..0x2F and IRQ1 is vector 0x21.
 * - This handler is installed in IDT as an interrupt gate (IF cleared on entry).
 */

.code64
.section .text
.align 16

/* PIC I/O ports */
.equ PIC1_CMD, 0x20
.equ PIC_EOI,  0x20

.global keyboard_irq_handler
.type keyboard_irq_handler, @function
keyboard_irq_handler:
    cld

    /* Save all GPRs (15 pushes => keeps stack alignment suitable for SysV calls). */
    pushq %rax
    pushq %rbx
    pushq %rcx
    pushq %rdx
    pushq %rbp
    pushq %rsi
    pushq %rdi
    pushq %r8
    pushq %r9
    pushq %r10
    pushq %r11
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15

    /* Call Rust-level handler (no args). */
    call keyboard_interrupt_handler

    /* Send End-Of-Interrupt to master PIC (IRQ1 is on PIC1). */
    movb $PIC_EOI, %al
    outb %al, $PIC1_CMD

    /* Restore registers. */
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %r11
    popq %r10
    popq %r9
    popq %r8
    popq %rdi
    popq %rsi
    popq %rbp
    popq %rdx
    popq %rcx
    popq %rbx
    popq %rax

    iretq

.size keyboard_irq_handler, . - keyboard_irq_handler