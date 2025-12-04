// kernel/tests/mod.rs
// Test kernel threads for preemption checks.

#[cfg(target_arch = "x86_64")]
pub mod preempt_threads;

#[cfg(target_arch = "aarch64")]
pub mod preempt_threads_aarch64;
