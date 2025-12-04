// kernel/syscalls/sched.rs
// Sleep and yield syscalls
// BSD 3-Clause License

#![no_std]

use crate::sched;
use crate::time;

pub const SYS_SLEEP: usize = 6;
pub const SYS_YIELD: usize = 7;

pub fn sys_sleep(ns: u64, cpu_id: usize) -> isize {
    let ticks_per_sec = sched::tick_hz();
    if ticks_per_sec == 0 {
        return -1;
    }
    let cur = sched::ticks();
    let ticks = ((ns as u128 * ticks_per_sec as u128) / 1_000_000_000u128) as u64;
    let wake_tick = cur.saturating_add(ticks.max(1));
    if let Some(cur_tid) = sched::current_tid(cpu_id) {
        let idx = cur_tid as usize;
        sched::mark_sleeping(idx, wake_tick);
        // Context switch will be driven by timer tick
        0
    } else {
        -1
    }
}

pub fn sys_yield(cpu_id: usize) -> isize {
    if sched::yield_current(cpu_id).is_some() {
        0
    } else {
        -1
    }
}
