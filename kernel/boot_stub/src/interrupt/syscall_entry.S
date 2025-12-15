/* kernel/boot_stub/src/interrupt/syscall_entry.S
 *
 * int 0x80 syscall entry for x86_64 boot_stub.
 *
 * User ABI (as assumed by your usertest and Rust dispatcher):
 *   rax = syscall number
 *   rdi = arg1
 *   rsi = arg2
 *   rdx = arg3
 *
 * Kernel ABI to Rust:
 *   syscall_dispatch(syscall_num, arg1, arg2, arg3) -> isize
 *   args in: rdi, rsi, rdx, rcx
 *   return:  rax
 *
 * IMPORTANT FIX:
 * - We do NOT push the original RAX, because RAX must return syscall result.
 * - Therefore there is no "skip RAX" and no restore-order trap.
 */

.code64
.section .text
.align 16

.global syscall_entry
.type syscall_entry, @function

.extern syscall_dispatch

syscall_entry:
    cld

    /* Save all regs EXCEPT RAX (RAX holds return value) */
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

    /* Save current stack pointer across the call in a callee-saved register */
    movq %rsp, %r15

    /* Align stack for System V ABI call */
    andq $-16, %rsp

    /* Marshal args for syscall_dispatch(sysnum,arg1,arg2,arg3):
     * original rdi/rsi/rdx were saved on stack at known offsets from r15
     *
     * Stack layout at r15 (top):
     *   0  r15
     *   8  r14
     *   16 r13
     *   24 r12
     *   32 r11
     *   40 r10
     *   48 r9
     *   56 r8
     *   64 rdi (arg1)
     *   72 rsi (arg2)
     *   80 rdx (arg3)
     *   88 rcx (saved)
     *   96 rbx
     *   104 rbp
     */
    movq 64(%r15), %rsi     /* arg1 */
    movq 72(%r15), %rdx     /* arg2 */
    movq 80(%r15), %rcx     /* arg3 */
    movq %rax, %rdi         /* syscall_num */

    call syscall_dispatch    /* rax = isize result */

    /* Restore original stack pointer */
    movq %r15, %rsp

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

    iretq

.size syscall_entry, . - syscall_entry
