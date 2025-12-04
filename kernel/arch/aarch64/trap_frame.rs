// kernel/arch/aarch64/trap_frame.rs
// Thin wrapper to expose the canonical AArch64 trap frame within the arch namespace.

#![cfg(target_arch = "aarch64")]

// Re-export the canonical definition from kernel/trapframe.rs to avoid divergent layouts.
pub use crate::trapframe::TrapFrameAArch64;
