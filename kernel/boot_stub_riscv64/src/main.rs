//! kernel/boot_stub_riscv64/src/main.rs
#![no_std]
#![no_main]

#[path = "../../arch/riscv64/sbi.rs"]
mod sbi;

#[path = "../../arch/riscv64/uart16550.rs"]
mod uart16550;

#[path = "../../arch/riscv64/dtb.rs"]
mod dtb;

#[path = "../../arch/riscv64/csr.rs"]
mod csr;

#[path = "../../arch/riscv64/clint.rs"]
mod clint;

#[path = "../../arch/riscv64/timer.rs"]
mod timer;

#[path = "../../arch/riscv64/trap.rs"]
mod trap;

// Boot entry (_start) from your existing boot.S
core::arch::global_asm!(include_str!("../../arch/riscv64/boot.S"));

// Trap vector assembly
core::arch::global_asm!(include_str!("../../arch/riscv64/trap.S"));

use core::panic::PanicInfo;
use crate::dtb::Console;

struct UartConsole(uart16550::Uart16550);

impl dtb::Console for UartConsole {
    fn putc(&self, ch: u8) { self.0.putc(ch); }
    fn puts(&self, s: &str) { self.0.puts(s); }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let con = UartConsole(uart16550::uart0());
    con.puts("[PANIC] GuardBSD RV64 panic\r\n");
    sbi::system_reset_shutdown();
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_main(hart_id: usize, dtb_ptr: usize) -> ! {
    let con = UartConsole(uart16550::uart0());

    con.puts("GuardBSD (RISC-V RV64) bring-up\r\n");
    con.puts("hart_id = ");
    con.put_dec_usize(hart_id);
    con.puts("\r\n");

    con.puts("dtb_ptr = ");
    con.put_hex_u64(dtb_ptr as u64);
    con.puts("\r\n");

    dtb::parse_and_print(dtb_ptr, &con);

    // Init traps + timer (CLINT mtime as source, stimecmp as compare)
    trap::init_traps_and_timer();
    con.puts("[TIMER] enabled (10Hz default)\r\n");

    // Main idle loop: wait for interrupts
    loop {
        unsafe { core::arch::asm!("wfi", options(nomem, nostack, preserves_flags)) }
    }
}
