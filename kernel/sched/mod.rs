// kernel/sched/mod.rs
// Preemptive scheduler core with arch contexts
// BSD 3-Clause License

#![no_std]

use core::cmp::min;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

#[cfg(target_arch = "x86_64")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ArchContext {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rax: u64,
    pub rsp: u64,
    pub rip: u64,
    pub rflags: u64,
    pub cs: u64,
    pub ss: u64,
    pub cr3: u64,
    pub mode_flags: u64,
}

#[cfg(target_arch = "aarch64")]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ArchContext {
    pub x: [u64; 31],
    pub sp: u64,
    pub elr: u64,
    pub spsr: u64,
    pub ttbr0: u64,
    pub esr: u64,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ThreadState {
    New,
    Ready,
    Running,
    Blocked,
    BlockedIpcRecv,
    BlockedIpcSend,
    Sleeping,
    Zombie,
}

#[derive(Clone, Copy)]
pub struct Tcb {
    pub tid: usize,
    pub pid: usize,
    pub state: ThreadState,
    pub priority: usize,
    pub time_slice: u64,
    pub wake_tick: u64,
    pub cpu_affinity: usize,
    pub ctx: ArchContext,
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
            wake_tick: 0,
            cpu_affinity: 0,
            ctx: ArchContext::zeroed(),
            next: None,
        }
    }
}

impl ArchContext {
    pub const fn zeroed() -> Self {
        Self {
            #[cfg(target_arch = "x86_64")]
            r15: 0,
            #[cfg(target_arch = "x86_64")]
            r14: 0,
            #[cfg(target_arch = "x86_64")]
            r13: 0,
            #[cfg(target_arch = "x86_64")]
            r12: 0,
            #[cfg(target_arch = "x86_64")]
            r11: 0,
            #[cfg(target_arch = "x86_64")]
            r10: 0,
            #[cfg(target_arch = "x86_64")]
            r9: 0,
            #[cfg(target_arch = "x86_64")]
            r8: 0,
            #[cfg(target_arch = "x86_64")]
            rdi: 0,
            #[cfg(target_arch = "x86_64")]
            rsi: 0,
            #[cfg(target_arch = "x86_64")]
            rbp: 0,
            #[cfg(target_arch = "x86_64")]
            rbx: 0,
            #[cfg(target_arch = "x86_64")]
            rdx: 0,
            #[cfg(target_arch = "x86_64")]
            rcx: 0,
            #[cfg(target_arch = "x86_64")]
            rax: 0,
            #[cfg(target_arch = "x86_64")]
            rsp: 0,
            #[cfg(target_arch = "x86_64")]
            rip: 0,
            #[cfg(target_arch = "x86_64")]
            rflags: 0x202,
            #[cfg(target_arch = "x86_64")]
            cs: 0x8,
            #[cfg(target_arch = "x86_64")]
            ss: 0x10,
            #[cfg(target_arch = "x86_64")]
            cr3: 0,
            #[cfg(target_arch = "x86_64")]
            mode_flags: 0,

            #[cfg(target_arch = "aarch64")]
            x: [0; 31],
            #[cfg(target_arch = "aarch64")]
            sp: 0,
            #[cfg(target_arch = "aarch64")]
            elr: 0,
            #[cfg(target_arch = "aarch64")]
            spsr: 0,
            #[cfg(target_arch = "aarch64")]
            ttbr0: 0,
            #[cfg(target_arch = "aarch64")]
            esr: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Spin lock
// ---------------------------------------------------------------------------

pub struct SpinLock<T> {
    flag: core::sync::atomic::AtomicBool,
    data: core::cell::UnsafeCell<T>,
}

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            flag: core::sync::atomic::AtomicBool::new(false),
            data: core::cell::UnsafeCell::new(value),
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
// Scheduler core
// ---------------------------------------------------------------------------

pub const MAX_CPUS: usize = 64;
pub const MAX_THREADS: usize = 256;
pub const MAX_PRIORITY: usize = 4;
pub const DEFAULT_TIME_SLICE_TICKS: u64 = 5;

#[derive(Copy, Clone)]
struct RunQueue {
    heads: [Option<usize>; MAX_PRIORITY],
    tails: [Option<usize>; MAX_PRIORITY],
}

impl RunQueue {
    const fn new() -> Self {
        Self {
            heads: [None; MAX_PRIORITY],
            tails: [None; MAX_PRIORITY],
        }
    }

    fn push(&mut self, tcb: &mut [Tcb; MAX_THREADS], idx: usize) {
        let p = min(tcb[idx].priority, MAX_PRIORITY - 1);
        tcb[idx].next = None;
        if let Some(t) = self.tails[p] {
            tcb[t].next = Some(idx);
        } else {
            self.heads[p] = Some(idx);
        }
        self.tails[p] = Some(idx);
    }

    fn pop(&mut self, tcb: &mut [Tcb; MAX_THREADS]) -> Option<usize> {
        for prio in (0..MAX_PRIORITY).rev() {
            if let Some(h) = self.heads[prio] {
                let nxt = tcb[h].next;
                self.heads[prio] = nxt;
                if nxt.is_none() {
                    self.tails[prio] = None;
                }
                tcb[h].next = None;
                return Some(h);
            }
        }
        None
    }
}

struct CpuSched {
    current: Option<usize>,
    rq: RunQueue,
}

impl CpuSched {
    const fn new() -> Self {
        Self {
            current: None,
            rq: RunQueue::new(),
        }
    }
}

struct Scheduler {
    tick_hz: u64,
    ticks: AtomicU64,
    next_tid: AtomicUsize,
    tcbs: [Tcb; MAX_THREADS],
    cpus: [CpuSched; MAX_CPUS],
}

static SCHED: SpinLock<Scheduler> = SpinLock::new(Scheduler {
    tick_hz: 0,
    ticks: AtomicU64::new(0),
    next_tid: AtomicUsize::new(1),
    tcbs: [Tcb::empty(); MAX_THREADS],
    cpus: [CpuSched::new(); MAX_CPUS],
});

extern "C" {
    fn arch_context_switch(old: *mut ArchContext, new: *const ArchContext);
}

pub fn init(tick_hz: u64) {
    let mut s = SCHED.lock();
    s.tick_hz = tick_hz;
}

pub fn register_thread(pid: usize, prio: usize, cpu: usize, ctx: ArchContext) -> Option<usize> {
    let mut s = SCHED.lock();
    let tid = s.next_tid.fetch_add(1, Ordering::Relaxed);
    let slot = s
        .tcbs
        .iter()
        .position(|t| matches!(t.state, ThreadState::New | ThreadState::Zombie))?;
    s.tcbs[slot] = Tcb {
        tid,
        pid,
        state: ThreadState::Ready,
        priority: min(prio, MAX_PRIORITY - 1),
        time_slice: DEFAULT_TIME_SLICE_TICKS,
        wake_tick: 0,
        cpu_affinity: cpu,
        ctx,
        next: None,
    };
    s.cpus[cpu].rq.push(&mut s.tcbs, slot);
    Some(slot)
}

/// Called from timer ISR; `frame` holds the interrupted context.
/// Returns pointer to next context if a switch is needed.
pub fn on_tick(cpu_id: usize, frame: &ArchContext) -> Option<*const ArchContext> {
    let mut s = SCHED.lock();
    let tick = s.ticks.fetch_add(1, Ordering::Relaxed) + 1;

    if let Some(cur) = s.cpus[cpu_id].current {
        s.tcbs[cur].ctx = *frame;
        if s.tcbs[cur].time_slice > 0 {
            s.tcbs[cur].time_slice -= 1;
        }
        if s.tcbs[cur].time_slice == 0 && s.tcbs[cur].state == ThreadState::Running {
            s.tcbs[cur].state = ThreadState::Ready;
            s.tcbs[cur].time_slice = DEFAULT_TIME_SLICE_TICKS;
            s.cpus[cpu_id].rq.push(&mut s.tcbs, cur);
            s.cpus[cpu_id].current = None;
        }
    }

    // Wake sleepers
    for (idx, t) in s.tcbs.iter_mut().enumerate() {
        if t.state == ThreadState::Sleeping && t.wake_tick <= tick {
            t.state = ThreadState::Ready;
            t.time_slice = DEFAULT_TIME_SLICE_TICKS;
            s.cpus[t.cpu_affinity].rq.push(&mut s.tcbs, idx);
        }
    }

    if s.cpus[cpu_id].current.is_none() {
        if let Some(next) = s.cpus[cpu_id].rq.pop(&mut s.tcbs) {
            // Skip killed tasks
            if let Some(proc) = crate::syscalls::process_jobctl::find_process_by_pid(s.tcbs[next].pid) {
                if proc.killed {
                    crate::print("[SIGNAL] delivering fatal signal to PID ");
                    crate::print_num(proc.pid as usize);
                    crate::print(" (terminating)\n");
                    s.tcbs[next].state = ThreadState::Zombie;
                    s.cpus[cpu_id].current = None;
                    return None;
                }
            }
            s.tcbs[next].state = ThreadState::Running;
            s.cpus[cpu_id].current = Some(next);
            let next_ctx = &s.tcbs[next].ctx as *const ArchContext;
            return Some(next_ctx);
        }
    }
    None
}

#[no_mangle]
pub extern "C" fn scheduler_handle_tick(cpu_id: usize, frame: *mut ArchContext) -> *const ArchContext {
    let next = on_tick(cpu_id, unsafe { &*frame });
    next.unwrap_or(core::ptr::null())
}

#[no_mangle]
pub extern "C" fn scheduler_handle_yield(cpu_id: usize, frame: *mut ArchContext) -> *const ArchContext {
    let next = yield_now(cpu_id, unsafe { &*frame });
    next.unwrap_or(core::ptr::null())
}

#[no_mangle]
pub extern "C" fn scheduler_handle_sleep(cpu_id: usize, frame: *mut ArchContext, wake_tick: u64) -> *const ArchContext {
    let next = sleep_until(cpu_id, unsafe { &*frame }, wake_tick);
    next.unwrap_or(core::ptr::null())
}

/// Explicit yield from syscall
pub fn yield_now(cpu_id: usize, frame: &ArchContext) -> Option<*const ArchContext> {
    let mut s = SCHED.lock();
    if let Some(cur) = s.cpus[cpu_id].current {
        s.tcbs[cur].ctx = *frame;
        s.tcbs[cur].state = ThreadState::Ready;
        s.tcbs[cur].time_slice = DEFAULT_TIME_SLICE_TICKS;
        s.cpus[cpu_id].rq.push(&mut s.tcbs, cur);
        s.cpus[cpu_id].current = None;
        if let Some(next) = s.cpus[cpu_id].rq.pop(&mut s.tcbs) {
            s.tcbs[next].state = ThreadState::Running;
            s.cpus[cpu_id].current = Some(next);
            return Some(&s.tcbs[next].ctx as *const ArchContext);
        }
    }
    None
}

pub fn sleep_until(cpu_id: usize, frame: &ArchContext, wake_tick: u64) -> Option<*const ArchContext> {
    let mut s = SCHED.lock();
    if let Some(cur) = s.cpus[cpu_id].current {
        s.tcbs[cur].ctx = *frame;
        s.tcbs[cur].state = ThreadState::Sleeping;
        s.tcbs[cur].wake_tick = wake_tick;
        s.cpus[cpu_id].current = None;
        if let Some(next) = s.cpus[cpu_id].rq.pop(&mut s.tcbs) {
            s.tcbs[next].state = ThreadState::Running;
            s.cpus[cpu_id].current = Some(next);
            return Some(&s.tcbs[next].ctx as *const ArchContext);
        }
    }
    None
}

pub fn current_tid(cpu_id: usize) -> Option<usize> {
    let s = SCHED.lock();
    s.cpus[cpu_id].current.map(|idx| s.tcbs[idx].tid)
}

/// Get current thread's context (for fork)
/// Returns a copy of the current thread's ArchContext
pub fn get_current_context(cpu_id: usize) -> Option<ArchContext> {
    let s = SCHED.lock();
    if let Some(idx) = s.cpus[cpu_id].current {
        if idx < MAX_THREADS {
            return Some(s.tcbs[idx].ctx);
        }
    }
    None
}

/// Update thread's context (used after fork to set child's return value)
/// Returns true if successful, false if thread not found
pub fn set_thread_context(tid: usize, ctx: ArchContext) -> bool {
    let mut s = SCHED.lock();
    for tcb in s.tcbs.iter_mut() {
        if tcb.tid == tid && tcb.state != ThreadState::New {
            tcb.ctx = ctx;
            return true;
        }
    }
    false
}

pub fn tick_hz() -> u64 {
    SCHED.lock().tick_hz
}

pub fn ticks() -> u64 {
    SCHED.lock().ticks.load(Ordering::Relaxed)
}

#[inline]
pub fn switch_context(old: &mut ArchContext, new: &ArchContext) {
    unsafe { arch_context_switch(old as *mut _, new as *const _) }
}

/// Timer tick entry wrapper for interrupt handlers (x86_64).
#[cfg(target_arch = "x86_64")]
pub fn timer_tick_entry(cpu_id: usize, tf: &mut crate::trapframe::TrapFrameX86_64) {
    let mut ctx: ArchContext = tf.into();
    ctx.cr3 = read_cr3();
    let next = scheduler_handle_tick(cpu_id, &mut ctx as *mut ArchContext);
    if next.is_null() {
        *tf = crate::trapframe::TrapFrameX86_64::from(&ctx);
    } else {
        unsafe {
            arch_context_switch(&mut ctx as *mut ArchContext, next);
        }
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn read_cr3() -> u64 {
    let val: u64;
    unsafe { core::arch::asm!("mov {}, cr3", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
fn read_ttbr0_el1() -> u64 {
    let val: u64;
    unsafe { core::arch::asm!("mrs {}, ttbr0_el1", out(reg) val, options(nomem, nostack, preserves_flags)) };
    val
}

/// Timer tick entry wrapper for interrupt handlers (AArch64).
#[cfg(target_arch = "aarch64")]
pub fn timer_tick_entry_aarch64(cpu_id: usize, tf: &mut crate::trapframe::TrapFrameAArch64) {
    // Preserve user SP only when returning to EL0; otherwise zero to avoid stale values.
    let ret_el = (tf.spsr_el1 >> 2) & 0b11;
    let user_sp = if ret_el == 0 { tf.sp_el0 } else { 0 };
    let mut ctx: ArchContext = tf.into();
    let next = scheduler_handle_tick(cpu_id, &mut ctx as *mut ArchContext);
    if next.is_null() {
        let mut updated = crate::trapframe::TrapFrameAArch64::from(&ctx);
        updated.sp_el0 = user_sp;
        *tf = updated;
    } else {
        unsafe {
            arch_context_switch(&mut ctx as *mut ArchContext, next);
        }
    }
}

pub fn unpark_thread(idx: usize) {
    let mut s = SCHED.lock();
    if idx < MAX_THREADS
        && matches!(
            s.tcbs[idx].state,
            ThreadState::Blocked | ThreadState::Sleeping | ThreadState::BlockedIpcRecv | ThreadState::BlockedIpcSend
        )
    {
        s.tcbs[idx].state = ThreadState::Ready;
        let cpu = s.tcbs[idx].cpu_affinity;
        s.cpus[cpu].rq.push(&mut s.tcbs, idx);
    }
}

/// Set thread state directly
/// Used by process management syscalls to update thread state
/// Returns true if successful, false if thread index invalid or already terminated
pub fn set_thread_state(tid: usize, new_state: ThreadState) -> bool {
    let mut s = SCHED.lock();
    if tid >= MAX_THREADS {
        return false;
    }
    
    // Don't allow resurrection of terminated threads
    if matches!(s.tcbs[tid].state, ThreadState::New) {
        return false;
    }
    
    let old_state = s.tcbs[tid].state;
    s.tcbs[tid].state = new_state;
    
    // If transitioning to Zombie, remove from run queue if currently scheduled
    if new_state == ThreadState::Zombie {
        // If this thread is currently running on a CPU, clear it
        for cpu in s.cpus.iter_mut() {
            if cpu.current == Some(tid) {
                cpu.current = None;
            }
        }
    }
    
    // If transitioning to Ready from non-Ready, add to run queue
    if new_state == ThreadState::Ready && old_state != ThreadState::Ready {
        let cpu = s.tcbs[tid].cpu_affinity;
        s.cpus[cpu].rq.push(&mut s.tcbs, tid);
    }
    
    true
}

// ---------------------------------------------------------------------------
// IPC block/wake helpers
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
pub fn scheduler_block_current_ipc_recv(_port: u64) {
    let mut s = SCHED.lock();
    if let Some(cur) = s.cpus[0].current {
        s.tcbs[cur].state = ThreadState::BlockedIpcRecv;
        s.cpus[0].current = None;
        if let Some(next) = s.cpus[0].rq.pop(&mut s.tcbs) {
            s.tcbs[next].state = ThreadState::Running;
            s.cpus[0].current = Some(next);
            let old_ptr = &mut s.tcbs[cur].ctx as *mut ArchContext;
            let new_ptr = &s.tcbs[next].ctx as *const ArchContext;
            drop(s);
            unsafe { arch_context_switch(old_ptr, new_ptr) };
        }
    }
}

#[cfg(target_arch = "x86_64")]
pub fn scheduler_block_current_ipc_send(_port: u64) {
    let mut s = SCHED.lock();
    if let Some(cur) = s.cpus[0].current {
        s.tcbs[cur].state = ThreadState::BlockedIpcSend;
        s.cpus[0].current = None;
        if let Some(next) = s.cpus[0].rq.pop(&mut s.tcbs) {
            s.tcbs[next].state = ThreadState::Running;
            s.cpus[0].current = Some(next);
            let old_ptr = &mut s.tcbs[cur].ctx as *mut ArchContext;
            let new_ptr = &s.tcbs[next].ctx as *const ArchContext;
            drop(s);
            unsafe { arch_context_switch(old_ptr, new_ptr) };
        }
    }
}

// ---------------------------------------------------------------------------
// Kernel thread support (x86_64 only)
// ---------------------------------------------------------------------------

pub type ThreadId = usize;

#[cfg(target_arch = "x86_64")]
pub fn spawn_kernel_thread(entry: fn()) -> ThreadId {
    use crate::mm::alloc_page;

    let stack_base = alloc_page().expect("kernel stack alloc failed");
    let stack_top = stack_base as u64 + 4096u64;

    let mut ctx = ArchContext::zeroed();
    ctx.rip = entry as u64;
    ctx.rsp = stack_top;
    ctx.cr3 = read_cr3();
    ctx.cs = 0x08;
    ctx.ss = 0x10;

    register_thread(0, 1, 0, ctx).expect("spawn_kernel_thread failed")
}

#[cfg(target_arch = "aarch64")]
pub fn spawn_kernel_thread_aarch64(entry: fn()) -> ThreadId {
    use crate::mm::alloc_page;

    let stack_base = alloc_page().expect("kernel stack alloc failed");
    let stack_top = stack_base as u64 + 4096u64;

    let mut ctx = ArchContext::zeroed();
    ctx.elr = entry as u64;
    ctx.sp = stack_top;
    ctx.spsr = 0x5; // EL1h, interrupts enabled (DAIF cleared)
    ctx.ttbr0 = read_ttbr0_el1();

    register_thread(0, 1, 0, ctx).expect("spawn_kernel_thread_aarch64 failed")
}

#[cfg(target_arch = "x86_64")]
static mut BOOT_CTX: ArchContext = ArchContext::zeroed();

#[cfg(target_arch = "x86_64")]
pub fn start_first_thread() -> ! {
    let next_ptr = {
        let mut s = SCHED.lock();
        if s.cpus[0].current.is_none() {
            if let Some(next) = s.cpus[0].rq.pop(&mut s.tcbs) {
                s.tcbs[next].state = ThreadState::Running;
                s.cpus[0].current = Some(next);
                &s.tcbs[next].ctx as *const ArchContext
            } else {
                core::ptr::null()
            }
        } else {
            core::ptr::null()
        }
    };

    if next_ptr.is_null() {
        loop {
            unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) };
        }
    } else {
        unsafe {
            arch_context_switch(&mut BOOT_CTX as *mut ArchContext, next_ptr);
        }
        loop {
            unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)) };
        }
    }
}

#[cfg(target_arch = "aarch64")]
static mut BOOT_CTX: ArchContext = ArchContext::zeroed();

#[cfg(target_arch = "aarch64")]
pub fn start_first_thread() -> ! {
    let next_ptr = {
        let mut s = SCHED.lock();
        if s.cpus[0].current.is_none() {
            if let Some(next) = s.cpus[0].rq.pop(&mut s.tcbs) {
                s.tcbs[next].state = ThreadState::Running;
                s.cpus[0].current = Some(next);
                &s.tcbs[next].ctx as *const ArchContext
            } else {
                core::ptr::null()
            }
        } else {
            core::ptr::null()
        }
    };

    if next_ptr.is_null() {
        loop {
            unsafe { core::arch::asm!("wfe", options(nomem, nostack, preserves_flags)) };
        }
    } else {
        unsafe {
            arch_context_switch(&mut BOOT_CTX as *mut ArchContext, next_ptr);
        }
        loop {
            unsafe { core::arch::asm!("wfe", options(nomem, nostack, preserves_flags)) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rr_switch() {
        init(100);
        let ctx = ArchContext::zeroed();
        let a = register_thread(1, 1, 0, ctx).unwrap();
        let b = register_thread(1, 1, 0, ctx).unwrap();
        {
            let mut s = SCHED.lock();
            s.cpus[0].current = Some(a);
            s.tcbs[a].state = ThreadState::Running;
            s.tcbs[a].time_slice = 1;
        }
        let cur_ctx = ArchContext::zeroed();
        let next = on_tick(0, &cur_ctx).unwrap();
        assert_eq!(next as usize != 0, true);
        let mut s = SCHED.lock();
        assert_eq!(s.cpus[0].current, Some(b));
    }
}
