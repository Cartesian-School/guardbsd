/* kernel/boot_stub/src/interrupt/keyboard_irq.S
 *
 * IRQ1 keyboard stub for x86_64 boot_stub.
 * Calls Rust: keyboard_interrupt_handler()
 * Sends PIC EOI and returns with iretq.
 */

.code64
.section .text
.align 16

.global keyboard_irq_handler
.type keyboard_irq_handler, @function

.extern keyboard_interrupt_handler

keyboard_irq_handler:
    cld

    /* Save all GP regs (we may interrupt kernel code) */
    pushq %rax
    pushq %rbp
    pushq %rbx
    pushq %rcx
    pushq %rdx
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

    /* Keep saved stack pointer in a callee-saved register across the call */
    movq %rsp, %r15

    /* Align stack for System V ABI call */
    andq $-16, %rsp

    call keyboard_interrupt_handler

    /* Restore original stack */
    movq %r15, %rsp

    /* PIC EOI for IRQ1 (master PIC) */
    movb $0x20, %al
    movw $0x20, %dx
    outb %al, %dx

    /* Restore regs */
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
    popq %rdx
    popq %rcx
    popq %rbx
    popq %rbp
    popq %rax

    iretq

.size keyboard_irq_handler, . - keyboard_irq_handler
