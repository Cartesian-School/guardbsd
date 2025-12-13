//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_tests
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Cztery wątki kernelowe do ćwiczenia preempcji i syscalls.

#![cfg(target_arch = "x86_64")]
#![allow(dead_code)]

use crate::syscalls::sched::{sys_sleep, sys_yield};

const COM1: u16 = 0x3F8;

fn serial_putc(c: u8) {
    unsafe {
        while (inb(COM1 + 5) & 0x20) == 0 {}
        outb(COM1, c);
        outb(COM1, b'\n');
    }
}

#[inline(always)]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nostack, preserves_flags));
}

#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    core::arch::asm!("in al, dx", out("al") ret, in("dx") port, options(nostack, preserves_flags));
    ret
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
        let _ = sys_sleep(20_000_000, 0); // ~20ms at 1GHz tick reference
    }
}

#[no_mangle]
pub extern "C" fn thread_d() {
    loop {
        serial_putc(b'D');
        // Busy loop, no yield/sleep; must be preempted by timer.
    }
}
