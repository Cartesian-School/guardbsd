// kernel/arch/aarch64/mod.rs
// AArch64 architecture glue: exception vectors, timer, and scheduler integration.

#![cfg(target_arch = "aarch64")]
#![allow(dead_code)]

pub mod interrupts;
pub mod trap_frame;

/// Main entry point for AArch64 kernel bring-up.
#[no_mangle]
pub extern "C" fn kernel_main_aarch64() -> ! {
    interrupts::init_exceptions_and_timer();
    loop {
        unsafe { core::arch::asm!("wfe", options(nomem, nostack, preserves_flags)) };
    }
}
