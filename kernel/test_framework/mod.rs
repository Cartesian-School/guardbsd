//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_test_framework
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Minimalny framework testowy no_std.

#![no_std]

use core::sync::atomic::{AtomicUsize, Ordering};

pub struct TestStats {
    pub passed: AtomicUsize,
    pub failed: AtomicUsize,
    pub total: AtomicUsize,
}

impl TestStats {
    pub const fn new() -> Self {
        Self {
            passed: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            total: AtomicUsize::new(0),
        }
    }

    pub fn record_pass(&self) {
        self.passed.fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_fail(&self) {
        self.failed.fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn summary(&self) -> (usize, usize, usize) {
        (
            self.passed.load(Ordering::Relaxed),
            self.failed.load(Ordering::Relaxed),
            self.total.load(Ordering::Relaxed),
        )
    }
}

pub static TEST_STATS: TestStats = TestStats::new();

#[macro_export]
macro_rules! assert_eq_test {
    ($left:expr, $right:expr) => {{
        let left_val = $left;
        let right_val = $right;
        if left_val == right_val {
            $crate::TEST_STATS.record_pass();
            true
        } else {
            $crate::TEST_STATS.record_fail();
            false
        }
    }};
}

#[macro_export]
macro_rules! assert_ne_test {
    ($left:expr, $right:expr) => {{
        let left_val = $left;
        let right_val = $right;
        if left_val != right_val {
            $crate::TEST_STATS.record_pass();
            true
        } else {
            $crate::TEST_STATS.record_fail();
            false
        }
    }};
}

#[macro_export]
macro_rules! assert_test {
    ($cond:expr) => {{
        if $cond {
            $crate::TEST_STATS.record_pass();
            true
        } else {
            $crate::TEST_STATS.record_fail();
            false
        }
    }};
}

#[macro_export]
macro_rules! test_case {
    ($name:expr, $body:block) => {{
        $body
    }};
}
