//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: tests_system
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Framework testów systemowych (współdzielony z integracyjnymi).

extern crate alloc;
use alloc::string::String;

// Test statistics
static mut TESTS_RUN: usize = 0;
static mut TESTS_PASSED: usize = 0;
static mut TESTS_FAILED: usize = 0;

/// Mark function as test case
pub use core::prelude::v1::*;

#[macro_export]
macro_rules! test_case {
    () => {};
}

/// Print test message
pub fn test_print(msg: &str) {
    // Use syscall to write to stdout
    let bytes = msg.as_bytes();
    unsafe {
        syscall_write(1, bytes.as_ptr(), bytes.len());
    }
}

/// Print test message with newline
pub fn test_println(msg: &str) {
    test_print(msg);
    test_print("\n");
}

/// Start test suite
pub fn test_suite_start(name: &str) {
    test_println("\n╔══════════════════════════════════════════════════════════════════╗");
    test_print("║  ");
    test_print(name);
    for _ in 0..(60 - name.len()) {
        test_print(" ");
    }
    test_println("║");
    test_println("╚══════════════════════════════════════════════════════════════════╝\n");
    
    unsafe {
        TESTS_RUN = 0;
        TESTS_PASSED = 0;
        TESTS_FAILED = 0;
    }
}

/// End test suite and print summary
pub fn test_suite_end() -> ! {
    let (run, passed, failed) = unsafe { (TESTS_RUN, TESTS_PASSED, TESTS_FAILED) };
    
    test_println("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    test_println("TEST SUMMARY");
    test_println("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    test_print("Total tests:  ");
    test_println(&format_number(run));
    
    test_print("Passed:       ");
    test_println(&format_number(passed));
    
    test_print("Failed:       ");
    test_println(&format_number(failed));
    
    test_println("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if failed == 0 {
        test_println("\n✅ ALL TESTS PASSED!\n");
        unsafe { syscall_exit(0); }
    } else {
        test_println("\n❌ SOME TESTS FAILED!\n");
        unsafe { syscall_exit(1); }
    }
}

/// Print test info
pub fn test_info(msg: &str) {
    unsafe { TESTS_RUN += 1; }
    test_print("🔵 INFO: ");
    test_println(msg);
}

/// Mark test as passed
pub fn test_pass(msg: &str) {
    unsafe { TESTS_PASSED += 1; }
    test_print("✅ PASS: ");
    test_println(msg);
}

/// Mark test as failed
pub fn test_fail(msg: &str) -> ! {
    unsafe { TESTS_FAILED += 1; }
    test_print("❌ FAIL: ");
    test_println(msg);
    test_suite_end();
}

/// Assert equality
pub fn assert_eq<T: PartialEq + core::fmt::Debug>(left: T, right: T, msg: &str) {
    if left != right {
        test_fail(msg);
    }
}

/// Assert true
pub fn assert(condition: bool, msg: &str) {
    if !condition {
        test_fail(msg);
    }
}

/// Format number to string
pub fn format_number(n: usize) -> String {
    if n == 0 {
        return String::from("0");
    }
    
    let mut result = String::new();
    let mut num = n;
    
    while num > 0 {
        let digit = (num % 10) as u8 + b'0';
        result.insert(0, digit as char);
        num /= 10;
    }
    
    result
}

// Syscall wrappers
unsafe fn syscall_write(fd: u64, buf: *const u8, count: usize) {
    core::arch::asm!(
        "syscall",
        in("rax") 13u64, // SYS_WRITE
        in("rdi") fd,
        in("rsi") buf,
        in("rdx") count,
        lateout("rax") _,
        options(nostack, preserves_flags)
    );
}

unsafe fn syscall_exit(code: i32) -> ! {
    core::arch::asm!(
        "syscall",
        in("rax") 0u64, // SYS_EXIT
        in("rdi") code,
        options(noreturn, nostack)
    );
}

// Global allocator for test framework
use core::alloc::{GlobalAlloc, Layout};

struct TestAllocator;

unsafe impl GlobalAlloc for TestAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Simple bump allocator for tests
        static mut HEAP: [u8; 131072] = [0; 131072]; // 128KB for system tests
        static mut HEAP_POS: usize = 0;
        
        let size = layout.size();
        let align = layout.align();
        
        // Align heap position
        let aligned_pos = (HEAP_POS + align - 1) & !(align - 1);
        
        if aligned_pos + size > HEAP.len() {
            return core::ptr::null_mut();
        }
        
        let ptr = HEAP.as_mut_ptr().add(aligned_pos);
        HEAP_POS = aligned_pos + size;
        
        ptr
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // No deallocation in simple bump allocator
    }
}

#[global_allocator]
static ALLOCATOR: TestAllocator = TestAllocator;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    test_fail("PANIC occurred during test");
}
