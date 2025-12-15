//! kernel/arch/riscv64/trap.rs
//! Trap handler + S-mode timer using:
//! - time source: rdtime (CSR time)
//! - interrupt source: Sstc stimecmp -> STIP delivered to S-mode
//! Rust 2024 safe (no static mut refs).

use crate::uart16550;
use super::csr;

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static INITED: AtomicBool = AtomicBool::new(false);

// 10Hz tick => print every 10 ticks (~1s).
static INTERVAL_TICKS: AtomicU64 = AtomicU64::new(super::timer::TIMEBASE_HZ_QEMU_VIRT / 10);
static TICKS: AtomicU64 = AtomicU64::new(0);

// scause decoding
const SCAUSE_INTERRUPT_BIT: usize = 1usize << (core::mem::size_of::<usize>() * 8 - 1);
const SCAUSE_CODE_MASK: usize = !SCAUSE_INTERRUPT_BIT;

// Supervisor Timer Interrupt code (interrupt cause)
const SUPERVISOR_TIMER_INTERRUPT: usize = 5;

// Exception codes (for minimal diag)
const ILLEGAL_INSTRUCTION: usize = 2;

pub fn init_traps_and_timer() {
    if INITED.swap(true, Ordering::SeqCst) {
        return;
    }

    unsafe extern "C" {
        fn trap_vector();
    }

    csr::write_stvec(trap_vector as *const () as usize);

    csr::enable_stimer_interrupt();
    csr::enable_supervisor_interrupts();

    // arm first tick after enabling
    arm_next_tick();

    // One-time diagnostics (can be removed later)
    let uart = uart16550::uart0();
    uart.puts("[TIMER] sstatus=");
    print_hex(&uart, csr::read_sstatus() as u64);
    uart.puts(" sie=");
    print_hex(&uart, csr::read_sie() as u64);
    uart.puts(" stimecmp=");
    print_hex(&uart, csr::read_stimecmp());
    uart.puts("\r\n");
}

fn arm_next_tick() {
    let now = csr::read_time();
    let interval = INTERVAL_TICKS.load(Ordering::Relaxed);
    csr::write_stimecmp(now.wrapping_add(interval));
}

#[unsafe(no_mangle)]
pub extern "C" fn trap_handler(scause: usize, sepc: usize, stval: usize) -> usize {
    let uart = uart16550::uart0();

    if (scause & SCAUSE_INTERRUPT_BIT) != 0 {
        let code = scause & SCAUSE_CODE_MASK;

        if code == SUPERVISOR_TIMER_INTERRUPT {
            let tick = TICKS.fetch_add(1, Ordering::Relaxed) + 1;
            arm_next_tick();

            // Log once per second (at 10Hz)
            if tick % 10 == 0 {
                uart.puts("[TIMER] tick=");
                print_dec(&uart, tick as usize);
                uart.puts("\r\n");
            }

            // Auto-shutdown after ~5s (50 ticks @ 10Hz) so QEMU ends by itself
            if tick == 50 {
                uart.puts("[TIMER] auto-shutdown\r\n");
                super::sbi::system_reset_shutdown();
            }

            return sepc;
        }

        uart.puts("[TRAP] irq code=");
        print_hex(&uart, code as u64);
        uart.puts("\r\n");
        return sepc;
    }

    // Exceptions: keep minimal, don't spam
    let code = scause & SCAUSE_CODE_MASK;

    uart.puts("[TRAP] exception code=");
    print_dec(&uart, code);
    uart.puts(" sepc=");
    print_hex(&uart, sepc as u64);
    uart.puts(" stval=");
    print_hex(&uart, stval as u64);
    uart.puts("\r\n");

    if code == ILLEGAL_INSTRUCTION {
        uart.puts("[TRAP] illegal instruction -> shutdown\r\n");
        super::sbi::system_reset_shutdown();
    }

    // Default advance to avoid loops
    sepc.wrapping_add(4)
}

fn print_dec(uart: &uart16550::Uart16550, mut v: usize) {
    if v == 0 {
        uart.putc(b'0');
        return;
    }
    let mut buf = [0u8; 32];
    let mut i = 0usize;
    while v > 0 && i < buf.len() {
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        uart.putc(buf[i]);
    }
}

fn print_hex(uart: &uart16550::Uart16550, v: u64) {
    uart.puts("0x");
    for i in (0..16).rev() {
        let n = ((v >> (i * 4)) & 0xF) as u8;
        uart.putc(if n <= 9 { b'0' + n } else { b'a' + (n - 10) });
    }
}
