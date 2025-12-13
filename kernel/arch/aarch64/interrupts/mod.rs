//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_arch_aarch64
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Integracja wektorów wyjątków i timera dla AArch64.

#![cfg(target_arch = "aarch64")]

pub mod vectors;

use core::sync::atomic::{AtomicU64, Ordering};

use crate::sched::{self, ArchContext};
use crate::trapframe::TrapFrameAArch64;

extern "C" {
    fn arch_context_switch(old: *mut ArchContext, new: *const ArchContext);
}

// Stored tick interval used to re-arm the virtual timer on each IRQ.
static TICK_INTERVAL: AtomicU64 = AtomicU64::new(0);

#[inline(always)]
fn gic_ack_eoi() {
    unsafe {
        let intid: u64;
        core::arch::asm!("mrs {}, ICC_IAR1_EL1", out(reg) intid, options(nostack));
        core::arch::asm!("msr ICC_EOIR1_EL1, {}", in(reg) intid, options(nostack, preserves_flags));
        core::arch::asm!("msr ICC_DIR_EL1, {}", in(reg) intid, options(nostack, preserves_flags));
    }
}

#[no_mangle]
pub extern "C" fn aarch64_timer_interrupt_handler(tf: &mut TrapFrameAArch64) {
    // Re-arm the generic timer for the next tick (one-shot programming).
    let interval = TICK_INTERVAL.load(Ordering::Relaxed);
    if interval != 0 {
        unsafe {
            core::arch::asm!("msr cntv_tval_el0, {}", in(reg) interval);
        }
    }

    let cpu_id = 0;
    sched::timer_tick_entry_aarch64(cpu_id, tf);

    // Minimal GICv3/v4 EOIR/Deactivate path (safe on QEMU virt).
    gic_ack_eoi();
}

#[no_mangle]
pub extern "C" fn aarch64_syscall_entry(tf: &mut TrapFrameAArch64) {
    let nr = tf.x[0] as usize;
    match nr {
        crate::syscalls::sched::SYS_YIELD => handle_yield(tf),
        crate::syscalls::sched::SYS_SLEEP => handle_sleep(tf),
        _ => handle_legacy(tf),
    }
}

fn handle_yield(tf: &mut TrapFrameAArch64) {
    let ret_el = (tf.spsr_el1 >> 2) & 0b11;
    let user_sp = if ret_el == 0 { tf.sp_el0 } else { 0 };
    let mut ctx: ArchContext = tf.into();
    let next = sched::scheduler_handle_yield(0, &mut ctx as *mut ArchContext);
    if next.is_null() {
        let mut updated = TrapFrameAArch64::from(&ctx);
        updated.sp_el0 = user_sp;
        *tf = updated;
    } else {
        unsafe { arch_context_switch(&mut ctx as *mut ArchContext, next) };
    }
}

fn handle_sleep(tf: &mut TrapFrameAArch64) {
    let ret_el = (tf.spsr_el1 >> 2) & 0b11;
    let user_sp = if ret_el == 0 { tf.sp_el0 } else { 0 };
    let ns = tf.x[1];
    let ticks_per_sec = sched::tick_hz();
    let now = sched::ticks();
    let ticks = ((ns as u128 * ticks_per_sec as u128) / 1_000_000_000u128) as u64;
    let wake = now.saturating_add(ticks.max(1));
    let mut ctx: ArchContext = tf.into();
    let next = sched::scheduler_handle_sleep(0, &mut ctx as *mut ArchContext, wake);
    if next.is_null() {
        let mut updated = TrapFrameAArch64::from(&ctx);
        updated.sp_el0 = user_sp;
        *tf = updated;
    } else {
        unsafe { arch_context_switch(&mut ctx as *mut ArchContext, next) };
    }
}

fn handle_legacy(tf: &mut TrapFrameAArch64) {
    let ret = crate::syscall::syscall_handler(
        tf.x[0] as usize,
        tf.x[1] as usize,
        tf.x[2] as usize,
        tf.x[3] as usize,
    );
    tf.x[0] = ret as u64;
}

pub fn init_exceptions_and_timer() {
    unsafe {
        extern "C" {
            static __exception_vectors_start: u8;
        }
        // Set VBAR_EL1 to vector table
        core::arch::asm!(
            "msr vbar_el1, {}",
            in(reg) &__exception_vectors_start as *const _ as u64,
            options(nostack, preserves_flags)
        );
        // Configure generic timer for periodic interrupts (~100 Hz)
        let freq: u64;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        let interval = freq / 100;
        TICK_INTERVAL.store(interval, Ordering::Relaxed);
        core::arch::asm!("msr cntv_tval_el0, {}", in(reg) interval);
        core::arch::asm!("msr cntv_ctl_el0, {}", in(reg) 1u64);
    }
    // Advertise tick frequency to the scheduler core.
    sched::init(100);
    // Unmask IRQs so the generic timer can fire.
    unsafe {
        core::arch::asm!("msr daifclr, #2", options(nostack, preserves_flags));
        // Minimal GICv3 enablement: set priority mask and enable group1.
        core::arch::asm!("msr ICC_PMR_EL1, {}", in(reg) 0xFFu64, options(nostack, preserves_flags));
        core::arch::asm!("msr ICC_IGRPEN1_EL1, {}", in(reg) 1u64, options(nostack, preserves_flags));
    }
}
