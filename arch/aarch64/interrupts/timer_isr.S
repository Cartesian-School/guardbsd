// arch/aarch64/interrupts/timer_isr.S
// Timer ISR that drives scheduler tick and context switch

.section .text
.global timer_isr
.type timer_isr, %function

// EL1 IRQ entry (vector should branch here)
timer_isr:
    // Allocate space for ArchContext on stack
    sub sp, sp, #(8*35)
    mov x2, sp

    // Save x0-x30
    stp x0, x1, [x2, #(8*28)]
    stp x2, x3, [x2, #(8*26)]
    stp x4, x5, [x2, #(8*24)]
    stp x6, x7, [x2, #(8*22)]
    stp x8, x9, [x2, #(8*20)]
    stp x10,x11,[x2, #(8*18)]
    stp x12,x13,[x2, #(8*16)]
    stp x14,x15,[x2, #(8*14)]
    stp x16,x17,[x2, #(8*12)]
    stp x18,x19,[x2, #(8*10)]
    stp x20,x21,[x2, #(8*8)]
    stp x22,x23,[x2, #(8*6)]
    stp x24,x25,[x2, #(8*4)]
    stp x26,x27,[x2, #(8*2)]
    stp x28,x29,[x2, #(8*0)]
    str x30, [x2, #(8*30)]

    // Save SP/ELR/SPSR/TTBR0
    mov x3, sp
    str x3, [x2, #(8*31)]
    mrs x3, elr_el1
    str x3, [x2, #(8*32)]
    mrs x3, spsr_el1
    str x3, [x2, #(8*33)]
    mrs x3, ttbr0_el1
    str x3, [x2, #(8*34)]

    // Call scheduler
    mov x0, #0          // cpu_id (placeholder)
    mov x1, x2          // frame pointer
    bl scheduler_handle_tick
    cbz x0, 1f
    // Switch context: x0 has new ctx
    mov x1, x0          // new ctx
    mov x0, x2          // old ctx
    bl arch_context_switch

1:
    // Restore stack and return
    add sp, sp, #(8*35)
    // Acknowledge generic timer
    mov x0, #0
    msr cntv_ctl_el0, x0
    eret

.size timer_isr, .-timer_isr
