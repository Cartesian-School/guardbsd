/* kernel/boot_stub/src/interrupt/keyboard_irq.S
 *
 * Minimal x86_64 IRQ1 handler for boot_stub.
 * - Saves GPRs
 * - Calls Rust: keyboard_interrupt_handler()
 * - Sends EOI to PIC master
 * - Returns with iretq
 */

.code64
.section .text
.align 16

.global keyboard_irq_handler
.type keyboard_irq_handler, @function
keyboard_irq_handler:
    cld

    /* Save GPRs (15 pushes).
     * NOTE: With typical 16-byte aligned kernel stack, the CPU interrupt frame
     * makes RSP misaligned by 8, and 15 pushes realign it for a SysV call.
     */
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

    /* Call Rust IRQ handler */
    call keyboard_interrupt_handler

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
.size keyboard_irq_handler, . - keyboard_irq_handler
