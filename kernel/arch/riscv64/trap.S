/* kernel/arch/riscv64/trap.S
 * Minimal S-mode trap vector.
 * Saves caller-saved registers, calls Rust handler, restores, sret.
 */

    .section .text.trap
    .globl trap_vector
    .align 4

trap_vector:
    addi sp, sp, -(16*8)

    sd ra,  0*8(sp)
    sd t0,  1*8(sp)
    sd t1,  2*8(sp)
    sd t2,  3*8(sp)
    sd t3,  4*8(sp)
    sd t4,  5*8(sp)
    sd t5,  6*8(sp)
    sd t6,  7*8(sp)
    sd a0,  8*8(sp)
    sd a1,  9*8(sp)
    sd a2, 10*8(sp)
    sd a3, 11*8(sp)
    sd a4, 12*8(sp)
    sd a5, 13*8(sp)
    sd a6, 14*8(sp)
    sd a7, 15*8(sp)

    /* Pass (scause, sepc, stval) to Rust */
    csrr a0, scause
    csrr a1, sepc
    csrr a2, stval
    call trap_handler

    /* trap_handler returns new sepc in a0 */
    csrw sepc, a0

    ld ra,  0*8(sp)
    ld t0,  1*8(sp)
    ld t1,  2*8(sp)
    ld t2,  3*8(sp)
    ld t3,  4*8(sp)
    ld t4,  5*8(sp)
    ld t5,  6*8(sp)
    ld t6,  7*8(sp)
    ld a0,  8*8(sp)
    ld a1,  9*8(sp)
    ld a2, 10*8(sp)
    ld a3, 11*8(sp)
    ld a4, 12*8(sp)
    ld a5, 13*8(sp)
    ld a6, 14*8(sp)
    ld a7, 15*8(sp)

    addi sp, sp, (16*8)
    sret
