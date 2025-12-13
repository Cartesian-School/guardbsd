//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_arch_x86_64
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Dyspozytor syscalls na TrapFrameX86_64 z integracją schedulera.

#![cfg(target_arch = "x86_64")]

use crate::sched::{self, ArchContext};
use crate::syscalls::sched::{SYS_SLEEP, SYS_YIELD};
use crate::trapframe::TrapFrameX86_64;

extern "C" {
    fn arch_context_switch(old: *mut ArchContext, new: *const ArchContext);
}

#[no_mangle]
pub extern "C" fn x86_64_syscall_entry(tf: &mut TrapFrameX86_64) {
    let nr = tf.rax as usize;

    match nr {
        SYS_YIELD => handle_yield(tf),
        SYS_SLEEP => handle_sleep(tf),
        _ => handle_legacy(tf),
    }
}

fn handle_yield(tf: &mut TrapFrameX86_64) {
    let mut ctx: ArchContext = tf.into();
    ctx.cr3 = read_cr3();
    let next = unsafe { sched::scheduler_handle_yield(0, &mut ctx as *mut _) };
    if next.is_null() {
        *tf = TrapFrameX86_64::from(&ctx);
    } else {
        unsafe { arch_context_switch(&mut ctx as *mut ArchContext, next) };
    }
}

fn handle_sleep(tf: &mut TrapFrameX86_64) {
    let mut ctx: ArchContext = tf.into();
    ctx.cr3 = read_cr3();

    // arg0 in rdi = nanoseconds
    let ns = tf.rdi;
    let ticks_per_sec = sched::tick_hz();
    let now = sched::ticks();
    let ticks = ((ns as u128 * ticks_per_sec as u128) / 1_000_000_000u128) as u64;
    let wake = now.saturating_add(ticks.max(1));

    let next = unsafe { sched::scheduler_handle_sleep(0, &mut ctx as *mut _, wake) };
    if next.is_null() {
        *tf = TrapFrameX86_64::from(&ctx);
    } else {
        unsafe { arch_context_switch(&mut ctx as *mut ArchContext, next) };
    }
}

fn handle_legacy(tf: &mut TrapFrameX86_64) {
    // Fallback to existing syscall handler signature (syscall_num, arg1, arg2, arg3)
    let ret = crate::syscall::syscall_handler(
        tf.rax as usize,
        tf.rdi as usize,
        tf.rsi as usize,
        tf.rdx as usize,
    );
    tf.rax = ret as u64;
}

#[inline(always)]
fn read_cr3() -> u64 {
    let val: u64;
    unsafe { core::arch::asm!("mov {}, cr3", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}
