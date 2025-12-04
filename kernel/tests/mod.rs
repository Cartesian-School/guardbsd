// kernel/tests/mod.rs
// Test kernels threads for preemption checks (x86_64 only).

#![cfg(target_arch = "x86_64")]

pub mod preempt_threads;
