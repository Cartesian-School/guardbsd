/* 
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: kernel_process
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Przełączanie kontekstu x86_64 (System V AMD64 ABI).
 */

.section .text
.global context_switch
.global context_init
.global jump_to_user

/*
 * Context structure (must match Rust/C definition):
 * 
 * struct Context
 *     uint64_t r15;      // offset 0
 *     uint64_t r14;      // offset 8
 *     uint64_t r13;      // offset 16
 *     uint64_t r12;      // offset 24
 *     uint64_t rbp;      // offset 32
 *     uint64_t rbx;      // offset 40
 *     uint64_t r11;      // offset 48
 *     uint64_t r10;      // offset 56
 *     uint64_t r9;       // offset 64
 *     uint64_t r8;       // offset 72
 *     uint64_t rax;      // offset 80
 *     uint64_t rcx;      // offset 88
 *     uint64_t rdx;      // offset 96
 *     uint64_t rsi;      // offset 104
 *     uint64_t rdi;      // offset 112
 *     uint64_t rip;      // offset 120
 *     uint64_t rsp;      // offset 128
 *     uint64_t rflags;   // offset 136
 *     uint64_t cs;       // offset 144
 *     uint64_t ss;       // offset 152
 * ;
 */

/*
 * context_switch(old_ctx: *mut Context, new_ctx: *const Context)
 *
 * Saves current CPU state to old_ctx, restores state from new_ctx
 *
 * Arguments (System V ABI):
 *   RDI - pointer to old context (to save current state)
 *   RSI - pointer to new context (to load new state)
 *
 * This function does NOT return normally - it "returns" by jumping
 * to the RIP saved in new_ctx
 */
.type context_switch, @function
context_switch:
    /* Save current context to old_ctx (RDI) */
    
    /* Save callee-saved registers first */
    movq %r15, 0(%rdi)
    movq %r14, 8(%rdi)
    movq %r13, 16(%rdi)
    movq %r12, 24(%rdi)
    movq %rbp, 32(%rdi)
    movq %rbx, 40(%rdi)
    
    /* Save caller-saved registers (for completeness) */
    movq %r11, 48(%rdi)
    movq %r10, 56(%rdi)
    movq %r9, 64(%rdi)
    movq %r8, 72(%rdi)
    movq %rax, 80(%rdi)
    movq %rcx, 88(%rdi)
    movq %rdx, 96(%rdi)
    movq %rsi, 104(%rdi)
    
    /* Save RDI itself (first argument) */
    movq %rdi, 112(%rdi)
    
    /* Save return address as RIP */
    movq (%rsp), %rax
    movq %rax, 120(%rdi)
    
    /* Save RSP (before return address was pushed) */
    leaq 8(%rsp), %rax
    movq %rax, 128(%rdi)
    
    /* Save RFLAGS */
    pushfq
    popq %rax
    movq %rax, 136(%rdi)
    
    /* Save segment selectors */
    movq $0x08, 144(%rdi)    /* CS - kernel code segment */
    movq $0x10, 152(%rdi)    /* SS - kernel data segment */
    
    /* Now load new context from new_ctx (RSI) */
    
    /* Restore callee-saved registers */
    movq 0(%rsi), %r15
    movq 8(%rsi), %r14
    movq 16(%rsi), %r13
    movq 24(%rsi), %r12
    movq 32(%rsi), %rbp
    movq 40(%rsi), %rbx
    
    /* Restore caller-saved registers */
    movq 48(%rsi), %r11
    movq 56(%rsi), %r10
    movq 64(%rsi), %r9
    movq 72(%rsi), %r8
    movq 80(%rsi), %rax
    movq 88(%rsi), %rcx
    movq 96(%rsi), %rdx
    
    /* Load new stack pointer */
    movq 128(%rsi), %rsp
    
    /* Push RFLAGS and restore */
    movq 136(%rsi), %r15   /* Temp storage */
    pushq %r15
    popfq
    movq 0(%rsi), %r15     /* Restore R15 */
    
    /* Load RDI last (was saved at offset 112) */
    movq 112(%rsi), %rdi
    
    /* Load RSI last (was saved at offset 104) */
    movq 104(%rsi), %rsi
    
    /* Jump to saved RIP (offset 120 in original new_ctx) */
    /* We need to get it before we clobbered RSI */
    /* Actually, we already clobbered RSI, so we need a different approach */
    /* Let's use the stack to jump */
    
    /* Better approach: push RIP and use ret */
    /* We need to save RSI value first */
    pushq %rsi              /* Save new RSI on stack */
    movq -128(%rsp), %rsi   /* Load back pointer to new_ctx (original RSI arg) */
    movq 120(%rsi), %rax    /* Load RIP from new_ctx */
    movq %rax, -8(%rsp)     /* Place RIP on stack below saved RSI */
    popq %rsi               /* Restore RSI */
    ret                     /* Jump to saved RIP */
    
.size context_switch, . - context_switch

/*
 * context_init(ctx: *mut Context, entry: fn(), stack_top: *mut u8, arg: u64)
 *
 * Initialize a new context for a thread/process
 *
 * Arguments:
 *   RDI - pointer to context structure to initialize
 *   RSI - entry point function
 *   RDX - stack top pointer
 *   RCX - argument to pass to entry point (will be in RDI)
 */
.type context_init, @function
context_init:
    /* Zero out all registers */
    xorq %rax, %rax
    movq %rax, 0(%rdi)      /* R15 */
    movq %rax, 8(%rdi)      /* R14 */
    movq %rax, 16(%rdi)     /* R13 */
    movq %rax, 24(%rdi)     /* R12 */
    movq %rax, 32(%rdi)     /* RBP */
    movq %rax, 40(%rdi)     /* RBX */
    movq %rax, 48(%rdi)     /* R11 */
    movq %rax, 56(%rdi)     /* R10 */
    movq %rax, 64(%rdi)     /* R9 */
    movq %rax, 72(%rdi)     /* R8 */
    movq %rax, 80(%rdi)     /* RAX */
    movq %rax, 88(%rdi)     /* RCX */
    movq %rax, 96(%rdi)     /* RDX */
    movq %rax, 104(%rdi)    /* RSI */
    
    /* Set RDI to the argument */
    movq %rcx, 112(%rdi)
    
    /* Set RIP to entry point */
    movq %rsi, 120(%rdi)
    
    /* Set RSP to stack top (aligned to 16 bytes) */
    movq %rdx, %rax
    andq $~0xF, %rax        /* Align to 16 bytes */
    movq %rax, 128(%rdi)
    
    /* Set default RFLAGS (IF=1, others clear) */
    movq $0x202, 136(%rdi)  /* IF flag set */
    
    /* Set segment selectors */
    movq $0x08, 144(%rdi)   /* CS */
    movq $0x10, 152(%rdi)   /* SS */
    
    ret
.size context_init, . - context_init

/*
 * jump_to_user(ctx: *const Context, user_entry: u64, user_stack: u64)
 *
 * Jump to user mode with given entry point and stack
 * This does NOT return
 *
 * Arguments:
 *   RDI - pointer to context (unused for now, for future)
 *   RSI - user mode entry point
 *   RDX - user mode stack pointer
 */
.type jump_to_user, @function
jump_to_user:
    /* Set up user mode segments */
    movq $0x23, %rax        /* User data segment (RPL=3) */
    movq %rax, %ds
    movq %rax, %es
    movq %rax, %fs
    movq %rax, %gs
    
    /* Build IRET frame on stack */
    /* Stack layout for IRETQ:
     * [RSP+0]  = RIP
     * [RSP+8]  = CS
     * [RSP+16] = RFLAGS
     * [RSP+24] = RSP
     * [RSP+32] = SS
     */
    
    pushq $0x23             /* SS (user data segment, RPL=3) */
    pushq %rdx              /* User RSP */
    pushfq                  /* Current RFLAGS */
    orq $0x200, (%rsp)      /* Ensure IF is set */
    pushq $0x1B             /* CS (user code segment, RPL=3) */
    pushq %rsi              /* User RIP */
    
    /* Clear all registers for security */
    xorq %rax, %rax
    xorq %rbx, %rbx
    xorq %rcx, %rcx
    xorq %rdx, %rdx
    xorq %rsi, %rsi
    xorq %rdi, %rdi
    xorq %rbp, %rbp
    xorq %r8, %r8
    xorq %r9, %r9
    xorq %r10, %r10
    xorq %r11, %r11
    xorq %r12, %r12
    xorq %r13, %r13
    xorq %r14, %r14
    xorq %r15, %r15
    
    /* Jump to user mode */
    iretq
.size jump_to_user, . - jump_to_user

/* Emergency halt if we somehow return from user mode */
.global emergency_halt
emergency_halt:
    cli
    hlt
    jmp emergency_halt
