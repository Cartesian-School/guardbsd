// arch/aarch64/context_switch.S
// Full context switch for GuardBSD (AArch64)
// Saves/restores x0-x30, SP, ELR, SPSR, TTBR0 and returns via eret.
// ArchContext offsets (bytes):
// x0 @0, x1 @8, ..., x30 @240, sp @248, elr @256, spsr @264, ttbr0 @272, esr @280

.section .text
.global arch_context_switch
.type arch_context_switch, %function

// void arch_context_switch(struct ArchContext *old, const struct ArchContext *new)
arch_context_switch:
    // Preserve pointers
    mov x2, x0          // x2 = old_ctx
    mov x3, x1          // x3 = new_ctx

    // Save x0-x30 into old_ctx
    stp x0, x1, [x2, #0]
    stp x2, x3, [x2, #16]
    stp x4, x5, [x2, #32]
    stp x6, x7, [x2, #48]
    stp x8, x9, [x2, #64]
    stp x10,x11,[x2, #80]
    stp x12,x13,[x2, #96]
    stp x14,x15,[x2, #112]
    stp x16,x17,[x2, #128]
    stp x18,x19,[x2, #144]
    stp x20,x21,[x2, #160]
    stp x22,x23,[x2, #176]
    stp x24,x25,[x2, #192]
    stp x26,x27,[x2, #208]
    stp x28,x29,[x2, #224]
    str x30,     [x2, #240]

    // Save SP/ELR/SPSR/TTBR0/ESR
    mov x4, sp
    str x4, [x2, #248]
    mrs x4, elr_el1
    str x4, [x2, #256]
    mrs x4, spsr_el1
    str x4, [x2, #264]
    mrs x4, ttbr0_el1
    str x4, [x2, #272]
    mrs x4, esr_el1
    str x4, [x2, #280]

    // Restore TTBR0 with barriers
    ldr x4, [x3, #272]
    dsb ish
    msr ttbr0_el1, x4
    isb

    // Restore ELR/SPSR and SP
    ldr x4, [x3, #256]
    msr elr_el1, x4
    ldr x4, [x3, #264]
    msr spsr_el1, x4
    ldr x4, [x3, #248]
    mov sp, x4

    // Restore GPRs (reverse-ish order leaves pointer available until end)
    ldr x30, [x3, #240]
    ldp x28, x29, [x3, #224]
    ldp x26, x27, [x3, #208]
    ldp x24, x25, [x3, #192]
    ldp x22, x23, [x3, #176]
    ldp x20, x21, [x3, #160]
    ldp x18, x19, [x3, #144]
    ldp x16, x17, [x3, #128]
    ldp x14, x15, [x3, #112]
    ldp x12, x13, [x3, #96]
    ldp x10, x11, [x3, #80]
    ldp x8,  x9,  [x3, #64]
    ldp x6,  x7,  [x3, #48]
    ldp x4,  x5,  [x3, #32]
    ldp x2,  x3,  [x3, #16]
    ldp x0,  x1,  [x3, #0]

    eret

.size arch_context_switch, .-arch_context_switch
