/* 
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: kernel_arch_aarch64
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 *
 * Wektory wyjątków EL1 z handlerami IRQ/SVC.
 */

.section .text
.align 11
.global __exception_vectors_start
__exception_vectors_start:
    // Synchronous EL1t
    b .
    // IRQ EL1t
    b .
    // FIQ EL1t
    b .
    // Error EL1t
    b .

    // Synchronous EL1h
    b .
    // IRQ EL1h (timer, etc.)
    bl aarch64_irq_entry
    // FIQ EL1h
    b .
    // Error EL1h
    b .

    // Synchronous from EL0
    bl aarch64_sync_entry
    // IRQ from EL0
    bl aarch64_irq_entry
    // FIQ from EL0
    b .
    // Error from EL0
    b .

    // Synchronous from EL0 32-bit (unused)
    b .
    // IRQ from EL0 32-bit
    b .
    // FIQ from EL0 32-bit
    b .
    // Error from EL0 32-bit
    b .

.global aarch64_irq_entry
.type aarch64_irq_entry, %function
aarch64_irq_entry:
    sub sp, sp, #(8*36)
    mov x2, sp

    // Save x0-x30
    stp x0, x1, [x2, #(8*0)]
    stp x2, x3, [x2, #(8*2)]
    stp x4, x5, [x2, #(8*4)]
    stp x6, x7, [x2, #(8*6)]
    stp x8, x9, [x2, #(8*8)]
    stp x10,x11,[x2, #(8*10)]
    stp x12,x13,[x2, #(8*12)]
    stp x14,x15,[x2, #(8*14)]
    stp x16,x17,[x2, #(8*16)]
    stp x18,x19,[x2, #(8*18)]
    stp x20,x21,[x2, #(8*20)]
    stp x22,x23,[x2, #(8*22)]
    stp x24,x25,[x2, #(8*24)]
    stp x26,x27,[x2, #(8*26)]
    stp x28,x29,[x2, #(8*28)]
    str x30,     [x2, #(8*30)]

    // Save SP_EL0 and SP_EL1
    mrs x3, sp_el0
    str x3, [x2, #(8*31)]
    add x3, sp, #(8*36)
    str x3, [x2, #(8*32)]
    mrs x3, elr_el1
    str x3, [x2, #(8*33)]
    mrs x3, spsr_el1
    str x3, [x2, #(8*34)]
    mrs x3, esr_el1
    str x3, [x2, #(8*35)]

    mov x0, x2
    bl aarch64_timer_interrupt_handler

    // Restore from trap frame
    ldr x3, [x2, #(8*33)]
    msr elr_el1, x3
    ldr x3, [x2, #(8*34)]
    msr spsr_el1, x3
    ldr x3, [x2, #(8*31)]
    msr sp_el0, x3
    ldr x3, [x2, #(8*32)]
    mov sp, x3

    ldp x28,x29,[x2, #(8*28)]
    ldp x26,x27,[x2, #(8*26)]
    ldp x24,x25,[x2, #(8*24)]
    ldp x22,x23,[x2, #(8*22)]
    ldp x20,x21,[x2, #(8*20)]
    ldp x18,x19,[x2, #(8*18)]
    ldp x16,x17,[x2, #(8*16)]
    ldp x14,x15,[x2, #(8*14)]
    ldp x12,x13,[x2, #(8*12)]
    ldp x10,x11,[x2, #(8*10)]
    ldp x8, x9, [x2, #(8*8)]
    ldp x6, x7, [x2, #(8*6)]
    ldp x4, x5, [x2, #(8*4)]
    ldp x2, x3, [x2, #(8*2)]
    ldp x0, x1, [x2, #(8*0)]
    add sp, sp, #(8*36)
    eret

.global aarch64_sync_entry
.type aarch64_sync_entry, %function
aarch64_sync_entry:
    sub sp, sp, #(8*36)
    mov x2, sp

    stp x0, x1, [x2, #(8*0)]
    stp x2, x3, [x2, #(8*2)]
    stp x4, x5, [x2, #(8*4)]
    stp x6, x7, [x2, #(8*6)]
    stp x8, x9, [x2, #(8*8)]
    stp x10,x11,[x2, #(8*10)]
    stp x12,x13,[x2, #(8*12)]
    stp x14,x15,[x2, #(8*14)]
    stp x16,x17,[x2, #(8*16)]
    stp x18,x19,[x2, #(8*18)]
    stp x20,x21,[x2, #(8*20)]
    stp x22,x23,[x2, #(8*22)]
    stp x24,x25,[x2, #(8*24)]
    stp x26,x27,[x2, #(8*26)]
    stp x28,x29,[x2, #(8*28)]
    str x30,     [x2, #(8*30)]

    mrs x3, sp_el0
    str x3, [x2, #(8*31)]
    add x3, sp, #(8*36)
    str x3, [x2, #(8*32)]
    mrs x3, elr_el1
    str x3, [x2, #(8*33)]
    mrs x3, spsr_el1
    str x3, [x2, #(8*34)]
    mrs x3, esr_el1
    str x3, [x2, #(8*35)]

    mov x0, x2
    bl aarch64_syscall_entry

    ldr x3, [x2, #(8*33)]
    msr elr_el1, x3
    ldr x3, [x2, #(8*34)]
    msr spsr_el1, x3
    ldr x3, [x2, #(8*31)]
    msr sp_el0, x3
    ldr x3, [x2, #(8*32)]
    mov sp, x3

    ldp x28,x29,[x2, #(8*28)]
    ldp x26,x27,[x2, #(8*26)]
    ldp x24,x25,[x2, #(8*24)]
    ldp x22,x23,[x2, #(8*22)]
    ldp x20,x21,[x2, #(8*20)]
    ldp x18,x19,[x2, #(8*18)]
    ldp x16,x17,[x2, #(8*16)]
    ldp x14,x15,[x2, #(8*14)]
    ldp x12,x13,[x2, #(8*12)]
    ldp x10,x11,[x2, #(8*10)]
    ldp x8, x9, [x2, #(8*8)]
    ldp x6, x7, [x2, #(8*6)]
    ldp x4, x5, [x2, #(8*4)]
    ldp x2, x3, [x2, #(8*2)]
    ldp x0, x1, [x2, #(8*0)]
    add sp, sp, #(8*36)
    eret
