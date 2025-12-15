/* kernel/boot_stub/src/interrupt/syscall_entry.S
 *
 * x86_64 INT 0x80 syscall entry stub for boot_stub.
 *
 * Calling convention (user -> kernel):
 *   RAX = syscall number
 *   RDI = arg1
 *   RSI = arg2
 *   RDX = arg3
 *
 * We call Rust:
 *   syscall_dispatch(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize
 *
 * System V ABI args:
 *   RDI, RSI, RDX, RCX
 *
 * Return:
 *   RAX = isize (syscall return)
 *
 * Implementation detail:
 * - We push a full GPR save set INCLUDING RAX.
 * - After call, we overwrite the saved RAX slot on stack with the return value.
 * - Then we restore registers strictly in reverse push order, ending with popq %rax,
 *   which loads the syscall return value into RAX.
 */

.code64
.section .text
.align 16

.global syscall_entry
.type syscall_entry, @function
syscall_entry:
    cld

    /* Preserve incoming syscall args in temporaries (before we start pushing). */
    movq %rax, %r10    /* sysno */
    movq %rdi, %r11    /* arg1  */
    movq %rsi, %r12    /* arg2  */
    movq %rdx, %r13    /* arg3  */

    /* Save all GPRs (push order matters; restore will be exact reverse). */
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

    /* Map to SysV ABI for syscall_dispatch(sysno, arg1, arg2, arg3). */
    movq %r10, %rdi    /* syscall_num */
    movq %r11, %rsi    /* arg1 */
    movq %r12, %rdx    /* arg2 */
    movq %r13, %rcx    /* arg3 */

    call syscall_dispatch
    /* Return value now in RAX. */

    /* -----------------------------------------------------------------
     * Overwrite saved original RAX on the stack with the syscall return.
     *
     * Stack layout after 15 pushes:
     *   [rsp + 0]   = saved r15
     *   ...
     *   [rsp + 14*8]= saved rax   <-- overwrite this slot
     * ----------------------------------------------------------------- */
    movq %rax, 112(%rsp)            /* 14*8 = 112 */

    /* Restore registers strictly in reverse order (standard). */
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
    popq %rax                        /* loads syscall return value */

    iretq

.size syscall_entry, . - syscall_entry
