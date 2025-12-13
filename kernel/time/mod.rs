//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_time
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Abstrakcja timera z backendami architektury.

#![no_std]

use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(target_arch = "x86_64")]
#[path = "../arch/x86_64/time.rs"]
mod arch_backend;

#[cfg(target_arch = "aarch64")]
#[path = "../arch/aarch64/time.rs"]
mod arch_backend;

pub use arch_backend::ArchTimerImpl;

pub trait ArchTimer {
    fn init(hz: u64) -> u64;
    fn monotonic_ns() -> u64;
    fn program_next_tick();
    fn eoi();
}

pub struct TimerState {
    hz: u64,
    ticks: AtomicU64,
    boot_ns: AtomicU64,
}

static TIMER: TimerState = TimerState {
    hz: 0,
    ticks: AtomicU64::new(0),
    boot_ns: AtomicU64::new(0),
};

pub fn init(requested_hz: u64) {
    let actual = ArchTimerImpl::init(requested_hz);
    TIMER.hz = actual;
    TIMER.boot_ns.store(ArchTimerImpl::monotonic_ns(), Ordering::Relaxed);
}

/// Called from ISR fast path.
pub fn tick() -> u64 {
    let t = TIMER.ticks.fetch_add(1, Ordering::Relaxed) + 1;
    t
}

pub fn get_ticks() -> u64 {
    TIMER.ticks.load(Ordering::Relaxed)
}

pub fn get_time_ns() -> u64 {
    let start = TIMER.boot_ns.load(Ordering::Relaxed);
    let now = ArchTimerImpl::monotonic_ns();
    now.saturating_sub(start)
}

pub fn program_next_tick() {
    ArchTimerImpl::program_next_tick();
}

pub fn ack() {
    ArchTimerImpl::eoi();
}
