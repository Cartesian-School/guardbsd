//! kernel/arch/riscv64/trap.rs
//! Trap handler + S-mode timer + PLIC external interrupt handling.
//!
//! - Timer: rdtime + stimecmp (Sstc)
//! - External IRQ: PLIC claim/complete (SEI)
//! - Demo source: UART RX interrupt -> press keys to trigger IRQ.
//!
//! Notes:
//! - QEMU virt wiring commonly uses UART0 external IRQ = 10.
//! - We enable only IRQ 10 for now to keep it minimal.

use crate::uart16550;
use super::{csr, plic, uart_irq};

use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static INITED: AtomicBool = AtomicBool::new(false);

// 10Hz tick => print every 10 ticks (~1s)
static INTERVAL_TICKS: AtomicU64 = AtomicU64::new(super::timer::TIMEBASE_HZ_QEMU_VIRT / 10);
static TICKS: AtomicU64 = AtomicU64::new(0);

// scause decoding (MSB indicates interrupt)
const SCAUSE_INTERRUPT_BIT: usize = 1usize << (core::mem::size_of::<usize>() * 8 - 1);
const SCAUSE_CODE_MASK: usize = !SCAUSE_INTERRUPT_BIT;

// Interrupt cause codes in S-mode (RISC-V privileged spec)
const SUPERVISOR_TIMER_INTERRUPT: usize = 5;
const SUPERVISOR_EXTERNAL_INTERRUPT: usize = 9;

// Exception codes (minimal diag)
const ILLEGAL_INSTRUCTION: usize = 2;

pub fn init_traps_and_timer() {
    // Debug marker: helps prove init happens once.
    // Remove after you confirm no double-init.
    let uart = uart16550::uart0();
    uart.puts("[INIT] init_traps_and_timer entered\r\n");

    if INITED.swap(true, Ordering::SeqCst) {
        let uart = uart16550::uart0();
        uart.puts("[INIT] already inited\r\n");
        return;
    }

    unsafe extern "C" {
        fn trap_vector();
    }

    // Rust 2024: cast fn item -> pointer -> usize
    csr::write_stvec(trap_vector as *const () as usize);

    // Enable interrupts in CSR
    csr::enable_stimer_interrupt();
    csr::enable_sext_interrupt();
    csr::enable_supervisor_interrupts();

    // Enable only UART IRQ=10 in PLIC (QEMU virt UART0)
    plic::init_smode_hart0_enable_irq(10);

    // Enable UART RX IRQ so external interrupts can be triggered by typing keys
    uart_irq::init_rx_irq_mode();

    // Arm first timer tick
    arm_next_tick();

    let uart = uart16550::uart0();
    uart.puts("[PLIC] enabled UART IRQ=10; press keys to trigger\r\n");
    uart.puts("[TIMER] enabled (10Hz default)\r\n");
}

fn arm_next_tick() {
    let now = csr::read_time();
    let interval = INTERVAL_TICKS.load(Ordering::Relaxed);
    csr::write_stimecmp(now.wrapping_add(interval));
}

#[unsafe(no_mangle)]
pub extern "C" fn trap_handler(scause: usize, sepc: usize, stval: usize) -> usize {
    let uart = uart16550::uart0();

    // Interrupts
    if (scause & SCAUSE_INTERRUPT_BIT) != 0 {
        let code = scause & SCAUSE_CODE_MASK;

        // Supervisor Timer Interrupt (STI)
        if code == SUPERVISOR_TIMER_INTERRUPT {
            let tick = TICKS.fetch_add(1, Ordering::Relaxed) + 1;

            // Re-arm next tick immediately
            arm_next_tick();

            // Log once per second (10Hz -> every 10 ticks)
            if tick % 10 == 0 {
                uart.puts("[TIMER] tick=");
                print_dec(&uart, tick as usize);
                uart.puts("\r\n");
            }

            // Auto-shutdown after ~20s so you don't have to kill QEMU manually
            if tick == 200 {
                uart.puts("[TIMER] auto-shutdown\r\n");
                super::sbi::system_reset_shutdown();
            }

            return sepc;
        }

        // Supervisor External Interrupt (SEI) -> PLIC claim/complete
        if code == SUPERVISOR_EXTERNAL_INTERRUPT {
            loop {
                let irq = plic::claim();
                if irq == 0 {
                    break;
                }

                // Drain UART RX to clear the source (if it was UART)
                let drained = uart_irq::drain_rx();

                uart.puts("[PLIC] irq=");
                print_dec(&uart, irq as usize);
                uart.puts(" uart_rx_bytes=");
                print_dec(&uart, drained);
                uart.puts("\r\n");

                plic::complete(irq);
            }
            return sepc;
        }

        // Other interrupt types (not enabled yet)
        uart.puts("[TRAP] irq code=");
        print_hex(&uart, code as u64);
        uart.puts("\r\n");
        return sepc;
    }

    // Exceptions
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

    // Default: advance PC to avoid infinite loops on synchronous exceptions
    let _ = stval;
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
