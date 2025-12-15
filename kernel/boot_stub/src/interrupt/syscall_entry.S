/* kernel/boot_stub/src/interrupt/syscall_entry.S
 *
 * int 0x80 entry for boot_stub (x86_64).
 *
 * ABI (boot_stub convention):
 * - rax = syscall_nr
 * - rdi = arg1
 * - rsi = arg2
 * - rdx = arg3
 * Return:
 * - rax = isize result
 *
 * Calls Rust:
 *   syscall_dispatch(syscall_nr, arg1, arg2, arg3) -> isize
 */

.code64
.section .text
.align 16

.global syscall_entry
.type syscall_entry, @function
syscall_entry:
    cld

    /* Save GPRs (15 pushes). */
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

    /* Build SysV call args for:
     * syscall_dispatch(syscall_nr, arg1, arg2, arg3)
     * SysV: rdi, rsi, rdx, rcx
     */
    mov %rdi, %r8          /* save arg1 */
    mov %rsi, %r9          /* save arg2 */
    mov %rdx, %r10         /* save arg3 */

    mov %rax, %rdi         /* rdi = syscall_nr */
    mov %r8,  %rsi         /* rsi = arg1 */
    mov %r9,  %rdx         /* rdx = arg2 */
    mov %r10, %rcx         /* rcx = arg3 */

    call syscall_dispatch

    /* Store return value into the saved-RAX slot on the stack.
     * Layout after pushes:
     *   rsp -> r15
     *   ...
     *   rsp+112 -> saved rax   (14 * 8)
     */
    mov %rax, 112(%rsp)

    /* Restore GPRs (reverse order). pop %rax will now load the return value. */
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
.size syscall_entry, . - syscall_entry
