// kernel/sched/mod.rs
// Preemptive scheduler core (arch-neutral)
// BSD 3-Clause License

#![no_std]

use core::cmp::min;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

// ---------------------------------------------------------------------------
// Basic Types
// ---------------------------------------------------------------------------

pub const MAX_CPUS: usize = 64;
pub const MAX_THREADS: usize = 256;
pub const MAX_PRIORITY: usize = 4;
pub const DEFAULT_TIME_SLICE_TICKS: u64 = 5;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ThreadState {
    New,
    Ready,
    Running,
    Blocked,
    Sleeping,
    Zombie,
}

#[derive(Copy, Clone)]
pub struct Context {
    pub gpr: [u64; 31],
    pub sp: u64,
    pub pc: u64,
    pub flags: u64,
}

impl Context {
    pub const fn zeroed() -> Self {
        Self {
            gpr: [0; 31],
            sp: 0,
            pc: 0,
            flags: 0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Tcb {
    pub tid: u64,
    pub pid: u64,
    pub state: ThreadState,
    pub priority: usize,
    pub time_slice: u64,
    pub context: Context,
    pub wake_tick: u64,
    pub cpu_affinity: Option<usize>,
    pub next: Option<usize>,
}

impl Tcb {
    pub const fn empty() -> Self {
        Self {
            tid: 0,
            pid: 0,
            state: ThreadState::New,
            priority: 1,
            time_slice: DEFAULT_TIME_SLICE_TICKS,
            context: Context::zeroed(),
            wake_tick: 0,
            cpu_affinity: None,
            next: None,
        }
    }
}

// ---------------------------------------------------------------------------
// SpinLock
// ---------------------------------------------------------------------------

pub struct SpinLock<T> {
    flag: core::sync::atomic::AtomicBool,
    data: core::cell::UnsafeCell<T>,
}

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            flag: core::sync::atomic::AtomicBool::new(false),
            data: core::cell::UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self
            .flag
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.flag.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }
        SpinLockGuard { lock: self }
    }
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.flag.store(false, Ordering::Release);
    }
}

impl<T> core::ops::Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> core::ops::DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

// ---------------------------------------------------------------------------
// Run queue
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
struct RunQueue {
    heads: [Option<usize>; MAX_PRIORITY],
    tails: [Option<usize>; MAX_PRIORITY],
    len: usize,
}

impl RunQueue {
    const fn new() -> Self {
        Self {
            heads: [None; MAX_PRIORITY],
            tails: [None; MAX_PRIORITY],
            len: 0,
        }
    }

    fn push(&mut self, tcb_pool: &mut [Tcb; MAX_THREADS], idx: usize) {
        let prio = min(tcb_pool[idx].priority, MAX_PRIORITY - 1);
        tcb_pool[idx].next = None;
        match self.tails[prio] {
            Some(tail_idx) => {
                tcb_pool[tail_idx].next = Some(idx);
            }
            None => {
                self.heads[prio] = Some(idx);
            }
        }
        self.tails[prio] = Some(idx);
        self.len += 1;
    }

    fn pop(&mut self, tcb_pool: &mut [Tcb; MAX_THREADS]) -> Option<usize> {
        for prio in (0..MAX_PRIORITY).rev() {
            if let Some(head_idx) = self.heads[prio] {
                let next = tcb_pool[head_idx].next;
                self.heads[prio] = next;
                if next.is_none() {
                    self.tails[prio] = None;
                }
                tcb_pool[head_idx].next = None;
                self.len -= 1;
                return Some(head_idx);
            }
        }
        None
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }
}

// ---------------------------------------------------------------------------
// Per-CPU scheduler state
// ---------------------------------------------------------------------------

struct CpuScheduler {
    current: Option<usize>,
    runqueue: RunQueue,
}

impl CpuScheduler {
    const fn new() -> Self {
        Self {
            current: None,
            runqueue: RunQueue::new(),
        }
    }
}

struct Scheduler {
    tick_hz: u64,
    ticks: AtomicU64,
    next_tid: AtomicUsize,
    tcbs: [Tcb; MAX_THREADS],
    cpus: [CpuScheduler; MAX_CPUS],
}

static SCHED: SpinLock<Scheduler> = SpinLock::new(Scheduler {
    tick_hz: 0,
    ticks: AtomicU64::new(0),
    next_tid: AtomicUsize::new(1),
    tcbs: [Tcb::empty(); MAX_THREADS],
    cpus: [CpuScheduler::new(); MAX_CPUS],
});

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn init(tick_hz: u64) {
    let mut sched = SCHED.lock();
    sched.tick_hz = tick_hz;
}

pub fn register_thread(pid: u64, priority: usize, cpu_hint: Option<usize>) -> Option<usize> {
    let mut sched = SCHED.lock();
    let tid = sched.next_tid.fetch_add(1, Ordering::Relaxed) as u64;
    let slot = sched
        .tcbs
        .iter()
        .position(|t| matches!(t.state, ThreadState::New | ThreadState::Zombie))?;
    let cpu = cpu_hint.unwrap_or(0);
    sched.tcbs[slot] = Tcb {
        tid,
        pid,
        state: ThreadState::Ready,
        priority: min(priority, MAX_PRIORITY - 1),
        time_slice: DEFAULT_TIME_SLICE_TICKS,
        context: Context::zeroed(),
        wake_tick: 0,
        cpu_affinity: Some(cpu),
        next: None,
    };
    sched.cpus[cpu].runqueue.push(&mut sched.tcbs, slot);
    Some(slot)
}

pub fn mark_sleeping(tcb_idx: usize, wake_tick: u64) {
    let mut sched = SCHED.lock();
    if let Some(cpu) = sched.tcbs[tcb_idx].cpu_affinity {
        if sched.cpus[cpu].current == Some(tcb_idx) {
            sched.tcbs[tcb_idx].state = ThreadState::Sleeping;
            sched.tcbs[tcb_idx].wake_tick = wake_tick;
        }
    }
}

pub fn wake_ready(now_tick: u64) {
    let mut sched = SCHED.lock();
    for (idx, tcb) in sched.tcbs.iter_mut().enumerate() {
        if tcb.state == ThreadState::Sleeping && tcb.wake_tick <= now_tick {
            tcb.state = ThreadState::Ready;
            tcb.time_slice = DEFAULT_TIME_SLICE_TICKS;
            let cpu = tcb.cpu_affinity.unwrap_or(0);
            sched.cpus[cpu].runqueue.push(&mut sched.tcbs, idx);
        }
    }
}

pub fn on_tick(cpu_id: usize) -> Option<(usize, usize)> {
    let mut sched = SCHED.lock();
    let tick = sched.ticks.fetch_add(1, Ordering::Relaxed) + 1;

    if let Some(current_idx) = sched.cpus[cpu_id].current {
        if sched.tcbs[current_idx].time_slice > 0 {
            sched.tcbs[current_idx].time_slice -= 1;
        }
        if sched.tcbs[current_idx].time_slice == 0 && sched.tcbs[current_idx].state == ThreadState::Running {
            sched.tcbs[current_idx].state = ThreadState::Ready;
            sched.tcbs[current_idx].time_slice = DEFAULT_TIME_SLICE_TICKS;
            sched.cpus[cpu_id]
                .runqueue
                .push(&mut sched.tcbs, current_idx);
            sched.cpus[cpu_id].current = None;
        }
    }

    // Wake sleeping threads if their deadline passed
    for (idx, tcb) in sched.tcbs.iter_mut().enumerate() {
        if tcb.state == ThreadState::Sleeping && tcb.wake_tick <= tick {
            tcb.state = ThreadState::Ready;
            tcb.time_slice = DEFAULT_TIME_SLICE_TICKS;
            let cpu = tcb.cpu_affinity.unwrap_or(cpu_id);
            sched.cpus[cpu].runqueue.push(&mut sched.tcbs, idx);
        }
    }

    if sched.cpus[cpu_id].current.is_none() {
        if let Some(next_idx) = sched.cpus[cpu_id].runqueue.pop(&mut sched.tcbs) {
            sched.tcbs[next_idx].state = ThreadState::Running;
            sched.cpus[cpu_id].current = Some(next_idx);
            return Some((current_idx_or_idle(&sched.cpus[cpu_id]), next_idx));
        }
    }
    None
}

pub fn yield_current(cpu_id: usize) -> Option<(usize, usize)> {
    let mut sched = SCHED.lock();
    if let Some(cur) = sched.cpus[cpu_id].current {
        sched.tcbs[cur].state = ThreadState::Ready;
        sched.tcbs[cur].time_slice = DEFAULT_TIME_SLICE_TICKS;
        sched.cpus[cpu_id].runqueue.push(&mut sched.tcbs, cur);
        sched.cpus[cpu_id].current = None;
        if let Some(next) = sched.cpus[cpu_id].runqueue.pop(&mut sched.tcbs) {
            sched.tcbs[next].state = ThreadState::Running;
            sched.cpus[cpu_id].current = Some(next);
            return Some((cur, next));
        }
    }
    None
}

pub fn current_tid(cpu_id: usize) -> Option<u64> {
    let sched = SCHED.lock();
    sched.cpus[cpu_id]
        .current
        .map(|idx| sched.tcbs[idx].tid)
}

pub fn ticks() -> u64 {
    SCHED.lock().ticks.load(Ordering::Relaxed)
}

pub fn tick_hz() -> u64 {
    SCHED.lock().tick_hz
}

fn current_idx_or_idle(cpu_sched: &CpuScheduler) -> usize {
    cpu_sched.current.unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Tests (run on host with std)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_robin_advances() {
        init(100);
        let a = register_thread(1, 1, Some(0)).unwrap();
        let b = register_thread(1, 1, Some(0)).unwrap();
        {
            let mut sched = SCHED.lock();
            sched.cpus[0].current = Some(a);
            sched.tcbs[a].state = ThreadState::Running;
            sched.tcbs[a].time_slice = 1;
        }
        let sw = on_tick(0).unwrap();
        assert_eq!(sw.0, a);
        assert_eq!(sw.1, b);
    }

    #[test]
    fn sleeping_wakes() {
        init(100);
        let t = register_thread(1, 1, Some(0)).unwrap();
        mark_sleeping(t, 5);
        let mut sched = SCHED.lock();
        sched.tcbs[t].state = ThreadState::Sleeping;
        drop(sched);
        wake_ready(5);
        let mut sched = SCHED.lock();
        assert_eq!(sched.tcbs[t].state, ThreadState::Ready);
    }
}
