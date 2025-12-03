// Syscall Entry Point - x86_64 Assembly
// BSD 3-Clause License

.section .text
.global syscall_entry

// Syscall entry from int 0x80 in 64-bit mode.
// Inputs (System V ABI conventions for user mode):
//   rax = syscall number
//   rdi = arg1
//   rsi = arg2
//   rdx = arg3
// Clobbers are handled to preserve kernel context before returning with iretq.
syscall_entry:
    // Preserve callee-saved registers
    pushq %rbp
    movq %rsp, %rbp
    pushq %rbx
    pushq %r12
    pushq %r13
    pushq %r14
    pushq %r15

    // Stash incoming arguments before re-mapping to syscall_dispatch ABI
    movq %rdi, %r10    // arg1
    movq %rsi, %r11    // arg2
    movq %rdx, %r12    // arg3

    // Prepare arguments: (syscall_num, arg1, arg2, arg3)
    movq %rax, %rdi    // syscall number
    movq %r10, %rsi    // arg1
    movq %r11, %rdx    // arg2
    movq %r12, %rcx    // arg3

    // Align stack to 16 bytes before call (we've pushed 6 registers: 48 bytes).
    // Stack is currently misaligned by 8 bytes relative to call entry, so adjust.
    subq $8, %rsp
    call syscall_dispatch
    addq $8, %rsp

    // Restore callee-saved registers
    popq %r15
    popq %r14
    popq %r13
    popq %r12
    popq %rbx
    leave

    // Return to caller
    iretq
