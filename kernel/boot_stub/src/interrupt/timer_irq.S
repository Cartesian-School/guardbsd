/* kernel/boot_stub/src/interrupt/timer_irq.S
 *
 * IRQ0 timer stub for x86_64 boot_stub.
 * Calls Rust: timer_interrupt_handler()
 * Sends PIC EOI and returns with iretq.
 */

.code64
.section .text
.align 16

.global timer_irq_handler
.type timer_irq_handler, @function

.extern timer_interrupt_handler

timer_irq_handler:
    cld

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

    movq %rsp, %r15
    andq $-16, %rsp

    call timer_interrupt_handler

    movq %r15, %rsp

    /* PIC EOI for IRQ0 (master PIC) */
    movb $0x20, %al
    movw $0x20, %dx
    outb %al, %dx

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

.size timer_irq_handler, . - timer_irq_handler
