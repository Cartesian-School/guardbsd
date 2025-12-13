//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_arch_x86_64
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Backend timera HPET/PIT (uproszczony) dla x86_64.

#![no_std]

pub struct ArchTimerImpl;

const HPET_BASE: u64 = 0xFED0_0000;
const HPET_GENERAL_CONFIG: u64 = HPET_BASE + 0x10;
const HPET_MAIN_COUNTER: u64 = HPET_BASE + 0xF0;

const PIT_CH0: u16 = 0x40;
const PIT_CMD: u16 = 0x43;
const PIT_FREQ: u64 = 1_193_182;

fn hpet_available() -> bool {
    // In a production kernel, probe ACPI tables; here we assume not available.
    false
}

fn pit_program(hz: u64) {
    let divisor = (PIT_FREQ / hz).clamp(1, u16::MAX as u64) as u16;
    unsafe {
        outb(PIT_CMD, 0x36);
        outb(PIT_CH0, (divisor & 0xFF) as u8);
        outb(PIT_CH0, (divisor >> 8) as u8);
    }
}

impl ArchTimerImpl {
    pub fn init(hz: u64) -> u64 {
        if hpet_available() {
            // Minimal HPET setup: enable periodic mode on timer 0
            unsafe {
                let cfg = (HPET_GENERAL_CONFIG as *mut u64).as_mut().unwrap();
                *cfg |= 1; // enable overall counter
            }
            hz
        } else {
            pit_program(hz);
            hz
        }
    }

    pub fn monotonic_ns() -> u64 {
        if hpet_available() {
            unsafe { core::ptr::read_volatile(HPET_MAIN_COUNTER as *const u64) }
        } else {
            // Fallback: ticks based estimate (not high accuracy)
            0
        }
    }

    pub fn program_next_tick() {
        // PIT is periodic; HPET would write comparator register here.
    }

    pub fn eoi() {
        // APIC/PIC EOI handled in platform interrupt path
    }
}

unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nostack, preserves_flags));
}
