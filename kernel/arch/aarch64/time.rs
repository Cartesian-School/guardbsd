// kernel/arch/aarch64/time.rs
// ARM Generic Timer backend (simplified)
// BSD 3-Clause License

#![no_std]

pub struct ArchTimerImpl;

impl ArchTimerImpl {
    pub fn init(hz: u64) -> u64 {
        let cntfrq = Self::counter_freq();
        let interval = cntfrq / hz;
        unsafe {
            core::arch::asm!("msr cntv_tval_el0, {}", in(reg) interval);
            core::arch::asm!("msr cntv_ctl_el0, {}", in(reg) 1u64);
        }
        hz
    }

    pub fn monotonic_ns() -> u64 {
        let cntfrq = Self::counter_freq();
        let counter: u64;
        unsafe { core::arch::asm!("mrs {}, cntvct_el0", out(reg) counter) };
        counter * 1_000_000_000u64 / cntfrq
    }

    pub fn program_next_tick() {
        let cntfrq = Self::counter_freq();
        let interval = cntfrq / 100; // default 100 Hz if not overridden
        unsafe {
            core::arch::asm!("msr cntv_tval_el0, {}", in(reg) interval);
        }
    }

    pub fn eoi() {
        // Generic timer does not require explicit EOI; interrupt controller handles it.
    }

    #[inline(always)]
    fn counter_freq() -> u64 {
        let freq: u64;
        unsafe { core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq) };
        freq
    }
}
