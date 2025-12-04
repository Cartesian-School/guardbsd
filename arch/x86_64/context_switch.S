// arch/x86_64/context_switch.S
// Full context switch for GuardBSD (x86_64)
// Saves/restores GPRs, switches CR3, returns with iretq using ArchContext layout.
// Offsets (bytes) in ArchContext:
// r15 0, r14 8, r13 16, r12 24, r11 32, r10 40, r9 48, r8 56,
// rdi 64, rsi 72, rbp 80, rbx 88, rdx 96, rcx 104, rax 112,
// rsp 120, rip 128, rflags 136, cs 144, ss 152, cr3 160, mode_flags 168.

.intel_syntax noprefix
.section .text
.global arch_context_switch
.type arch_context_switch,@function

// void arch_context_switch(struct ArchContext *old, const struct ArchContext *new)
arch_context_switch:
    // rdi = old_ctx, rsi = new_ctx
    lea r8, [rsp]                // original RSP (points to return RIP)
    push rax                     // save original RAX
    push rcx                     // save original RCX
    push rdx                     // save original RDX

    mov rdx, rdi                 // rdx -> old_ctx

    // Save current GPRs into old_ctx
    mov [rdx + 0],  r15
    mov [rdx + 8],  r14
    mov [rdx + 16], r13
    mov [rdx + 24], r12
    mov [rdx + 32], r11
    mov [rdx + 40], r10
    mov [rdx + 48], r9          // save original r9
    mov r9,  rsi                // r9  -> new_ctx pointer
    mov [rdx + 56], r8          // original rsp snapshot
    mov [rdx + 64], rdi
    mov [rdx + 72], rsi
    mov [rdx + 80], rbp
    mov [rdx + 88], rbx
    mov rax, [rsp]               // original rdx
    mov [rdx + 96], rax
    mov rax, [rsp + 8]           // original rcx
    mov [rdx + 104], rax
    mov rax, [rsp + 16]          // original rax
    mov [rdx + 112], rax
    mov [rdx + 120], r8          // original rsp
    mov rax, [r8]                // return RIP
    mov [rdx + 128], rax
    pushfq
    pop qword ptr [rdx + 136]
    mov rax, cs
    mov [rdx + 144], rax
    mov rax, ss
    mov [rdx + 152], rax
    mov rax, cr3
    mov [rdx + 160], rax
    mov qword ptr [rdx + 168], 0 // mode_flags placeholder

    // Restore stack (drop saved rax/rcx/rdx)
    add rsp, 24

    // Load new CR3 (no compare; higher layer ensures correctness)
    mov rax, [r9 + 160]
    mov cr3, rax
    mfence

    // Use r9 as new_ctx pointer throughout restore
    // Build iretq frame on new stack using new_ctx values
    mov rsp, [r9 + 120]          // set new RSP first
    push qword ptr [r9 + 152]    // SS
    push qword ptr [r9 + 120]    // RSP
    push qword ptr [r9 + 136]    // RFLAGS
    push qword ptr [r9 + 144]    // CS
    push qword ptr [r9 + 128]    // RIP

    // Restore GPRs (order chosen to keep pointer available)
    mov r15, [r9 + 0]
    mov r14, [r9 + 8]
    mov r13, [r9 + 16]
    mov r12, [r9 + 24]
    mov r11, [r9 + 32]
    mov r10, [r9 + 40]
    mov r8,  [r9 + 56]
    mov rdi, [r9 + 64]
    mov rsi, [r9 + 72]
    mov rbp, [r9 + 80]
    mov rbx, [r9 + 88]
    mov rdx, [r9 + 96]
    mov rcx, [r9 + 104]
    mov rax, [r9 + 112]
    mov r9,  [r9 + 48]           // restore r9 last

    swapgs
    iretq
.size arch_context_switch,.-arch_context_switch
