// kernel/tests/preempt_threads_aarch64.rs
// Four EL1 kernel threads to exercise preemption on AArch64.

#![cfg(target_arch = "aarch64")]
#![allow(dead_code)]

use crate::syscalls::sched::{sys_sleep, sys_yield};

const UART0: usize = 0x0900_0000; // QEMU virt PL011

#[inline(always)]
fn serial_putc(c: u8) {
    unsafe {
        // Wait while TX FIFO full (FR bit5)
        while (core::ptr::read_volatile((UART0 + 0x18) as *const u32) & 0x20) != 0 {}
        core::ptr::write_volatile(UART0 as *mut u8, c);
        while (core::ptr::read_volatile((UART0 + 0x18) as *const u32) & 0x20) != 0 {}
        core::ptr::write_volatile(UART0 as *mut u8, b'\n');
    }
}

#[no_mangle]
pub extern "C" fn thread_a() {
    loop {
        serial_putc(b'A');
        let _ = sys_yield(0);
    }
}

#[no_mangle]
pub extern "C" fn thread_b() {
    loop {
        serial_putc(b'B');
        let _ = sys_yield(0);
    }
}

#[no_mangle]
pub extern "C" fn thread_c() {
    loop {
        serial_putc(b'C');
        let _ = sys_sleep(20_000_000, 0); // ~20ms
    }
}

#[no_mangle]
pub extern "C" fn thread_d() {
    loop {
        serial_putc(b'D');
        // Busy loop; timer IRQ must preempt.
    }
}
