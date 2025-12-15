/* kernel/arch/riscv64/boot.S 
 * Project: GuardBSD Winter Saga version 1.0.0
 * Package: kernel_arch_riscv64
 * Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
 * License: BSD-3-Clause
 * RISC-V 64-bit boot stub for QEMU virt + OpenSBI
 * - Enters in S-mode
 * - a0 = hart id
 * - a1 = device tree blob (DTB) pointer
 */
    .section .text.boot
    .globl _start
    .align  4

_start:
    /* Set up a temporary stack */
    la      sp, __stack_top

    /* Save a0/a1 for Rust entry (hart_id, dtb_ptr) */
    mv      s0, a0
    mv      s1, a1

    /* Clear .bss */
    la      t0, __bss_start
    la      t1, __bss_end
1:
    bgeu    t0, t1, 2f
    sd      zero, 0(t0)
    addi    t0, t0, 8
    j       1b
2:
    /* Call Rust entry */
    mv      a0, s0
    mv      a1, s1
    la      t0, rust_main
    jr      t0

    /* If rust_main ever returns, park the hart */
3:
    wfi
    j 3b
