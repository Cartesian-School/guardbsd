/* kernel/boot_stub/src/interrupt/timer_irq.S
 *
 * Minimal x86_64 IRQ0 handler for boot_stub.
 * - Saves GPRs
 * - Calls Rust: timer_interrupt_handler()
 * - Sends EOI to PIC master
 * - Returns with iretq
 */

.code64
.section .text
.align 16

.global timer_irq_handler
.type timer_irq_handler, @function
timer_irq_handler:
    cld

    /* Save GPRs (15 pushes) for SysV stack alignment, same rationale as keyboard. */
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

    /* PIC EOI (master) */
    movb $0x20, %al
    movw $0x20, %dx
    outb %al, %dx

    /* Restore GPRs */
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
.size timer_irq_handler, . - timer_irq_handler
