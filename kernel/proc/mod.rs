// kernel/proc/mod.rs
// Minimal PID model for ETAP 3.2
#![no_std]

use core::sync::atomic::{AtomicU32, Ordering};
use crate::arch::x86_64::enter_user_mode;

static CURRENT_PID: AtomicU32 = AtomicU32::new(1);

/// Initialize the boot task (PID 1).
/// For now this is just a thin wrapper around setting CURRENT_PID = 1.
pub fn init_boot_task() {
    CURRENT_PID.store(1, Ordering::Relaxed);
}

/// Return the current process ID.
/// At this stage there is only a single task (PID 1).
pub fn current_pid() -> u32 {
    CURRENT_PID.load(Ordering::Relaxed)
}

/// Set the current PID.
/// This is provided for future ETAP steps, but is not used yet.
pub fn set_current_pid(pid: u32) {
    CURRENT_PID.store(pid, Ordering::Relaxed);
}

/// Start the first user task at the given entry and user stack pointer.
/// For now, assumes a single task (PID 1) and never returns.
pub fn start_first_user_task(entry: u64, user_sp: u64) -> ! {
    set_current_pid(1);
    unsafe { enter_user_mode(entry, user_sp); }
}

#[cfg(feature = "user_mode_test")]
pub fn test_enter_user_mode() -> ! {
    extern "C" {
        fn dummy_user_entry() -> !;
    }
    const TEST_USER_STACK_TOP: u64 = 0x8000_0000;
    set_current_pid(1);
    unsafe {
        enter_user_mode(dummy_user_entry as u64, TEST_USER_STACK_TOP);
    }
}
