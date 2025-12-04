// kernel/trapframe.rs
// Canonical trap frame definitions for GuardBSD (x86_64 and AArch64)
// These mirror the registers saved by interrupt/trap entry prologues and
// context switch routines. Keep this layout in sync with assembly save/restore
// code to guarantee offset correctness.

#![no_std]

use crate::sched::ArchContext;

// ---------------------------------------------------------------------------
// x86_64 Trap Frame
// ---------------------------------------------------------------------------

/// Trap frame capturing all general-purpose state on x86_64.
/// Order matches typical interrupt/ISR prologue that pushes GPRs then the
/// hardware-pushed RIP/CS/RFLAGS/SS, plus optional error_code.
#[cfg(target_arch = "x86_64")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct TrapFrameX86_64 {
    // Caller- and callee-saved GPRs
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

    /// Stack pointer at trap entry (before hardware pushed RIP/CS/RFLAGS/SS)
    pub rsp: u64,
    /// Instruction pointer at trap entry
    pub rip: u64,
    /// RFLAGS at trap entry
    pub rflags: u64,
    /// Code segment selector
    pub cs: u64,
    /// Stack segment selector (if privilege change)
    pub ss: u64,
    /// Optional error code pushed by some faults/interrupts
    pub error_code: u64,
    /// Mode flags for kernel/user boundary bookkeeping (optional)
    pub mode_flags: u64,
}

impl TrapFrameX86_64 {
    #[inline]
    pub const fn new() -> Self {
        Self {
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            r11: 0,
            r10: 0,
            r9: 0,
            r8: 0,
            rdi: 0,
            rsi: 0,
            rbp: 0,
            rbx: 0,
            rdx: 0,
            rcx: 0,
            rax: 0,
            rsp: 0,
            rip: 0,
            rflags: 0,
            cs: 0,
            ss: 0,
            error_code: 0,
            mode_flags: 0,
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl From<&TrapFrameX86_64> for ArchContext {
    fn from(tf: &TrapFrameX86_64) -> Self {
        let mut ctx = ArchContext::zeroed();
        ctx.r15 = tf.r15;
        ctx.r14 = tf.r14;
        ctx.r13 = tf.r13;
        ctx.r12 = tf.r12;
        ctx.r11 = tf.r11;
        ctx.r10 = tf.r10;
        ctx.r9 = tf.r9;
        ctx.r8 = tf.r8;
        ctx.rdi = tf.rdi;
        ctx.rsi = tf.rsi;
        ctx.rbp = tf.rbp;
        ctx.rbx = tf.rbx;
        ctx.rdx = tf.rdx;
        ctx.rcx = tf.rcx;
        ctx.rax = tf.rax;
        ctx.rsp = tf.rsp;
        ctx.rip = tf.rip;
        ctx.rflags = tf.rflags;
        ctx.cs = tf.cs;
        ctx.ss = tf.ss;
        // ArchContext::cr3/mode are left for caller to populate.
        ctx
    }
}

#[cfg(target_arch = "x86_64")]
impl From<&ArchContext> for TrapFrameX86_64 {
    fn from(ctx: &ArchContext) -> Self {
        Self {
            r15: ctx.r15,
            r14: ctx.r14,
            r13: ctx.r13,
            r12: ctx.r12,
            r11: ctx.r11,
            r10: ctx.r10,
            r9: ctx.r9,
            r8: ctx.r8,
            rdi: ctx.rdi,
            rsi: ctx.rsi,
            rbp: ctx.rbp,
            rbx: ctx.rbx,
            rdx: ctx.rdx,
            rcx: ctx.rcx,
            rax: ctx.rax,
            rsp: ctx.rsp,
            rip: ctx.rip,
            rflags: ctx.rflags,
            cs: ctx.cs,
            ss: ctx.ss,
            error_code: 0,
            mode_flags: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// AArch64 Trap Frame
// ---------------------------------------------------------------------------

/// Trap frame capturing AArch64 general registers and EL1 state.
/// Layout mirrors a vector-table prologue saving x0..x30 plus SP/ELR/SPSR.
#[cfg(target_arch = "aarch64")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct TrapFrameAArch64 {
    pub x: [u64; 31],     // x0..x30
    pub sp_el0: u64,      // User stack pointer
    pub sp_el1: u64,      // Kernel stack pointer at trap entry
    pub elr_el1: u64,     // Return address
    pub spsr_el1: u64,    // Saved PSTATE
    pub esr_el1: u64,     // Optional: exception syndrome
}

impl TrapFrameAArch64 {
    #[inline]
    pub const fn new() -> Self {
        Self {
            x: [0; 31],
            sp_el0: 0,
            sp_el1: 0,
            elr_el1: 0,
            spsr_el1: 0,
            esr_el1: 0,
        }
    }
}

#[cfg(target_arch = "aarch64")]
impl From<&TrapFrameAArch64> for ArchContext {
    fn from(tf: &TrapFrameAArch64) -> Self {
        let mut ctx = ArchContext::zeroed();
        ctx.x = tf.x;
        ctx.sp = tf.sp_el1;
        ctx.elr = tf.elr_el1;
        ctx.spsr = tf.spsr_el1;
        ctx.ttbr0 = 0; // caller should populate address space
        ctx
    }
}

#[cfg(target_arch = "aarch64")]
impl From<&ArchContext> for TrapFrameAArch64 {
    fn from(ctx: &ArchContext) -> Self {
        Self {
            x: ctx.x,
            sp_el0: 0,
            sp_el1: ctx.sp,
            elr_el1: ctx.elr,
            spsr_el1: ctx.spsr,
            esr_el1: 0,
        }
    }
}
