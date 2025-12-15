//! kernel/arch/riscv64/mod.rs
//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_arch_riscv64
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
#![no_std]
#![no_main]

// Pull in the boot stub as global asm
core::arch::global_asm!(include_str!("boot.S"));

mod sbi;
mod uart16550;

use core::fmt::{self, Write};

struct KernelConsole;

impl Write for KernelConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // Prefer MMIO UART because it works even if SBI console differs
        uart16550::uart0().puts(s);
        Ok(())
    }
}

macro_rules! kprint {
    ($($arg:tt)*) => {{
        let _ = write!(KernelConsole, $($arg)*);
    }};
}

macro_rules! kprintln {
    () => { kprint!("\n") };
    ($fmt:expr) => { kprint!(concat!($fmt, "\n")) };
    ($fmt:expr, $($arg:tt)*) => { kprint!(concat!($fmt, "\n"), $($arg)*) };
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kprintln!("[PANIC] {}", info);
    sbi::system_reset_shutdown();
}

/// Rust entry called from boot.S
/// a0 = hart_id, a1 = dtb_ptr (FDT)
#[no_mangle]
pub extern "C" fn rust_main(hart_id: usize, dtb_ptr: usize) -> ! {
    kprintln!("GuardBSD (RISC-V) bring-up");
    kprintln!("hart_id = {}", hart_id);
    kprintln!("dtb_ptr = 0x{:x}", dtb_ptr);

    kprintln!("Hello from UART16550 on QEMU virt.");
    kprintln!("Shutting down via SBI...");

    // If you want to keep running instead:
    // loop { unsafe { core::arch::asm!("wfi", options(nostack)) } }

    sbi::system_reset_shutdown();
}
