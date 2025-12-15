//! kernel/arch/riscv64/sbi.rs
//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_arch_riscv64
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//! Minimal SBI v0.2 calls for QEMU/OpenSBI.
//! We use it for debug output and clean shutdown.

#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> (usize, usize) {
    let error: usize;
    let value: usize;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a2") arg2,
            in("a3") 0usize,
            in("a4") 0usize,
            in("a5") 0usize,
            in("a6") fid,
            in("a7") eid,
            options(nostack)
        );
    }
    (error, value)
}

/// Legacy console putchar exists in older SBI, but in SBI v0.2 there is EID=0x01 (Console).
/// OpenSBI still supports the legacy extension on many setups.
/// For maximal compatibility, we provide both:
#[allow(dead_code)]
pub fn legacy_console_putchar(ch: u8) {
    let _ = sbi_call(0x01 /* legacy */, 0, ch as usize, 0, 0);
}

/// System reset (SBI v0.2): EID=0x53525354 ("SRST")
pub fn system_reset_shutdown() -> ! {
    const SBI_EID_SRST: usize = 0x5352_5354;
    const SBI_FID_RESET: usize = 0;
    const RESET_TYPE_SHUTDOWN: usize = 0;
    const RESET_REASON_NONE: usize = 0;

    let _ = sbi_call(
        SBI_EID_SRST,
        SBI_FID_RESET,
        RESET_TYPE_SHUTDOWN,
        RESET_REASON_NONE,
        0,
    );

    loop {
        unsafe { core::arch::asm!("wfi", options(nostack)) }
    }
}
