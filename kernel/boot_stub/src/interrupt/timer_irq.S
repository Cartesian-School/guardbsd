/* kernel/boot_stub/src/interrupt/timer_irq.S
 *
 * x86_64 IRQ0 timer interrupt handler stub for boot_stub.
 *
 * Responsibilities:
 * - Save all GPRs (safe for Rust call)
 * - Call Rust handler: timer_interrupt_handler()
 * - Send EOI to 8259 PIC (master)
 * - Restore registers and return with IRETQ
 *
 * Notes:
 * - IRQ0 is on master PIC, vector 0x20 after PIC remap.
 */

.code64
.section .text
.align 16

/* PIC I/O ports */
.equ PIC1_CMD, 0x20
.equ PIC_EOI,  0x20

.global timer_irq_handler
.type timer_irq_handler, @function
timer_irq_handler:
    cld

    /* Save all GPRs (15 pushes => good alignment for SysV calls). */
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
    call timer_interrupt_handler

    /* Send End-Of-Interrupt to master PIC (IRQ0 is on PIC1). */
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

.size timer_irq_handler, . - timer_irq_handler