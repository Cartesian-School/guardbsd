//! kernel/arch/riscv64/csr.rs
//! Minimal CSR helpers for RISC-V S-mode (no_std).

#![allow(dead_code)]

#[inline(always)]
pub fn read_scause() -> usize {
    let v: usize;
    unsafe { core::arch::asm!("csrr {0}, scause", out(reg) v, options(nomem, nostack, preserves_flags)) };
    v
}

#[inline(always)]
pub fn read_sepc() -> usize {
    let v: usize;
    unsafe { core::arch::asm!("csrr {0}, sepc", out(reg) v, options(nomem, nostack, preserves_flags)) };
    v
}

#[inline(always)]
pub fn write_sepc(v: usize) {
    unsafe { core::arch::asm!("csrw sepc, {0}", in(reg) v, options(nomem, nostack, preserves_flags)) };
}

#[inline(always)]
pub fn read_stval() -> usize {
    let v: usize;
    unsafe { core::arch::asm!("csrr {0}, stval", out(reg) v, options(nomem, nostack, preserves_flags)) };
    v
}

#[inline(always)]
pub fn write_stvec(addr: usize) {
    // direct mode (bits[1:0]=0)
    unsafe { core::arch::asm!("csrw stvec, {0}", in(reg) addr, options(nomem, nostack, preserves_flags)) };
}

#[inline(always)]
pub fn enable_supervisor_interrupts() {
    // sstatus.SIE = 1 (bit 1) -> immediate is 2 (OK for csrsi)
    unsafe { core::arch::asm!("csrsi sstatus, 0x2", options(nomem, nostack, preserves_flags)) };
}

#[inline(always)]
pub fn read_sstatus() -> usize {
    let v: usize;
    unsafe { core::arch::asm!("csrr {0}, sstatus", out(reg) v, options(nomem, nostack, preserves_flags)) };
    v
}

#[inline(always)]
pub fn read_sie() -> usize {
    let v: usize;
    unsafe { core::arch::asm!("csrr {0}, sie", out(reg) v, options(nomem, nostack, preserves_flags)) };
    v
}

#[inline(always)]
pub fn enable_stimer_interrupt() {
    // sie.STIE = bit 5 => must use register form (cannot use csrsi because 1<<5 doesn't fit 0..31)
    let mask: usize = 1 << 5;
    unsafe {
        core::arch::asm!(
            "csrrs x0, sie, {0}",
            in(reg) mask,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline(always)]
pub fn disable_stimer_interrupt() {
    let mask: usize = 1 << 5;
    unsafe {
        core::arch::asm!(
            "csrrc x0, sie, {0}",
            in(reg) mask,
            options(nomem, nostack, preserves_flags)
        );
    }
}

/// Sstc: stimecmp CSR = 0x14D on RV64. (Works on your QEMU/OpenSBI: ISA shows sstc.)
#[inline(always)]
pub fn write_stimecmp(value: u64) {
    unsafe {
        core::arch::asm!(
            "csrw 0x14D, {0}",
            in(reg) value,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[inline(always)]
pub fn read_stimecmp() -> u64 {
    let v: u64;
    unsafe {
        core::arch::asm!(
            "csrr {0}, 0x14D",
            out(reg) v,
            options(nomem, nostack, preserves_flags)
        );
    }
    v
}

/// Time source for S-mode: read the `time` CSR / rdtime instruction.
/// This avoids CLINT MMIO, which OpenSBI blocks for S-mode in your log.
#[inline(always)]
pub fn read_time() -> u64 {
    let v: u64;
    unsafe {
        core::arch::asm!(
            "rdtime {0}",
            out(reg) v,
            options(nomem, nostack, preserves_flags)
        );
    }
    v
}
