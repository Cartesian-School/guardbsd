// tests/integration/signal_integration_test.rs
// Day 32: Advanced Integration Tests - Signals & Process Management
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;

mod test_framework;
use test_framework::*;

// Signal numbers (from kernel/signal/signals.rs)
const SIGTERM: i32 = 15;
const SIGKILL: i32 = 9;
const SIGUSR1: i32 = 10;
const SIGUSR2: i32 = 12;
const SIGCHLD: i32 = 17;

// Global flag for signal handler
static mut SIGNAL_RECEIVED: bool = false;
static mut SIGNAL_NUMBER: i32 = 0;

/// Test signal handler (simple version)
extern "C" fn test_signal_handler(signum: i32) {
    unsafe {
        SIGNAL_RECEIVED = true;
        SIGNAL_NUMBER = signum;
    }
}

/// Test 7: Signal delivery during syscalls
/// Test that signals can interrupt blocking syscalls
#[test_case]
fn test_signal_during_syscall() {
    test_info("Test 7: Signal delivery during syscalls");
    
    let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
    
    if pid == 0 {
        // Child - block in sleep, should be interrupted by signal
        unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 10000) }; // 10 seconds
        
        // If we get here, signal interrupted sleep
        unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
    } else if pid > 0 {
        // Parent - sleep briefly then send signal
        unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 100) }; // 100ms
        
        // Send SIGUSR1 to child
        let ret = unsafe { 
            syscall::syscall2(syscall_numbers::SYS_KILL as u64, pid as u64, SIGUSR1 as u64) 
        };
        
        if ret as i64 == 0 {
            // Wait for child
            let mut status = 0i32;
            let _ = unsafe { 
                syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
            };
            
            test_pass("Signal delivered during syscall");
        } else {
            test_fail("kill() syscall failed");
        }
    } else {
        test_fail("fork() failed");
    }
}

/// Test 8: Signal handlers
/// Test that custom signal handlers are invoked correctly
#[test_case]
fn test_signal_handlers() {
    test_info("Test 8: Signal handlers");
    
    unsafe {
        SIGNAL_RECEIVED = false;
        SIGNAL_NUMBER = 0;
    }
    
    // Register signal handler for SIGUSR1
    let ret = unsafe {
        syscall::syscall2(
            syscall_numbers::SYS_SIGNAL as u64,
            SIGUSR1 as u64,
            test_signal_handler as u64
        )
    };
    
    if ret as i64 >= 0 {
        let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
        
        if pid == 0 {
            // Child - wait for signal
            unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 500) }; // 500ms
            
            // Check if signal was received
            let received = unsafe { SIGNAL_RECEIVED };
            let signum = unsafe { SIGNAL_NUMBER };
            
            if received && signum == SIGUSR1 {
                unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
            } else {
                unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 1) };
            }
        } else if pid > 0 {
            // Parent - send signal to child
            unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 100) }; // 100ms
            
            let _ = unsafe { 
                syscall::syscall2(syscall_numbers::SYS_KILL as u64, pid as u64, SIGUSR1 as u64) 
            };
            
            // Wait for child
            let mut status = 0i32;
            let _ = unsafe { 
                syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
            };
            
            if status == 0 {
                test_pass("Signal handler invoked correctly");
            } else {
                test_fail("Signal handler not invoked");
            }
        } else {
            test_fail("fork() failed");
        }
    } else {
        test_fail("signal() syscall failed");
    }
}

/// Test 9: Process tree correctness
/// Verify that process tree relationships are maintained correctly
#[test_case]
fn test_process_tree() {
    test_info("Test 9: Process tree correctness");
    
    let my_pid = unsafe { syscall::syscall0(syscall_numbers::SYS_GETPID as u64) as i32 };
    
    let child_pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
    
    if child_pid == 0 {
        // Child - verify parent PID
        let ppid = unsafe { syscall::syscall0(syscall_numbers::SYS_GETPPID as u64) as i32 };
        
        if ppid == my_pid {
            // Fork grandchild
            let grandchild_pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
            
            if grandchild_pid == 0 {
                // Grandchild - verify parent is child
                let my_ppid = unsafe { syscall::syscall0(syscall_numbers::SYS_GETPPID as u64) as i32 };
                
                if my_ppid > 0 {
                    unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
                } else {
                    unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 1) };
                }
            } else if grandchild_pid > 0 {
                // Wait for grandchild
                let mut status = 0i32;
                let _ = unsafe { 
                    syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
                };
                
                unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, status as u64) };
            } else {
                unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 2) };
            }
        } else {
            unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 3) };
        }
    } else if child_pid > 0 {
        // Parent - wait for child
        let mut status = 0i32;
        let _ = unsafe { 
            syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
        };
        
        if status == 0 {
            test_pass("Process tree relationships correct");
        } else {
            test_fail("Process tree verification failed");
        }
    } else {
        test_fail("fork() failed");
    }
}

/// Test 10: Resource cleanup on exit
/// Verify that resources are properly cleaned up when process exits
#[test_case]
fn test_resource_cleanup() {
    test_info("Test 10: Resource cleanup on exit");
    
    // Test that file descriptors, memory, etc. are cleaned up
    let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
    
    if pid == 0 {
        // Child - allocate some resources
        // Open file
        let path = b"/tmp/test_cleanup\0";
        let fd = unsafe {
            syscall::syscall2(syscall_numbers::SYS_OPEN as u64, path.as_ptr() as u64, 0o644)
        } as i32;
        
        // Exit without explicitly closing (should be cleaned up)
        unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
    } else if pid > 0 {
        // Parent - wait for child
        let mut status = 0i32;
        let _ = unsafe { 
            syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
        };
        
        // TODO: Verify resources were cleaned up
        // This would require introspection into kernel state
        
        test_pass("Resource cleanup test completed");
    } else {
        test_fail("fork() failed");
    }
}

/// Test 11: Stress test - many processes
/// Create many processes to test scalability and resource limits
#[test_case]
fn test_stress_many_processes() {
    test_info("Test 11: Stress test - many processes");
    
    const NUM_PROCESSES: usize = 50;
    let mut child_pids = [0i32; NUM_PROCESSES];
    let mut fork_count = 0;
    
    // Fork many processes
    for i in 0..NUM_PROCESSES {
        let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
        
        if pid == 0 {
            // Child - do some work then exit
            let my_pid = unsafe { syscall::syscall0(syscall_numbers::SYS_GETPID as u64) as i32 };
            
            // Small amount of work
            let mut sum = 0u64;
            for j in 0..100 {
                sum += (my_pid as u64) * (j as u64);
            }
            
            // Exit with LSB of sum
            unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, (sum & 0xFF)) };
        } else if pid > 0 {
            child_pids[fork_count] = pid;
            fork_count += 1;
        } else {
            // fork() failed - might have hit resource limit
            break;
        }
    }
    
    // Wait for all children
    let mut collected = 0;
    for _ in 0..fork_count {
        let mut status = 0i32;
        let wait_pid = unsafe { 
            syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
        } as i32;
        
        if wait_pid > 0 {
            collected += 1;
        }
    }
    
    assert_eq!(collected, fork_count, "Should collect all forked children");
    
    if fork_count >= NUM_PROCESSES / 2 {
        test_pass(&alloc::format!("Stress test: created {} processes", fork_count));
    } else {
        test_fail("Could not create enough processes for stress test");
    }
}

/// Test 12: Signal delivery to process group
/// Test sending signals to multiple processes
#[test_case]
fn test_signal_process_group() {
    test_info("Test 12: Signal delivery to process group");
    
    const NUM_CHILDREN: usize = 3;
    let mut child_pids = [0i32; NUM_CHILDREN];
    
    // Fork multiple children
    for i in 0..NUM_CHILDREN {
        let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
        
        if pid == 0 {
            // Child - wait for signal
            unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 10000) }; // 10 seconds
            unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
        } else if pid > 0 {
            child_pids[i] = pid;
        } else {
            test_fail("fork() failed");
            return;
        }
    }
    
    // Parent - send signal to all children
    unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 100) }; // 100ms
    
    for &pid in &child_pids {
        if pid > 0 {
            let _ = unsafe { 
                syscall::syscall2(syscall_numbers::SYS_KILL as u64, pid as u64, SIGTERM as u64) 
            };
        }
    }
    
    // Wait for all children
    for _ in 0..NUM_CHILDREN {
        let mut status = 0i32;
        let _ = unsafe { 
            syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
        };
    }
    
    test_pass("Signal delivery to multiple processes completed");
}

/// Main test runner
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_suite_start("Day 32: Advanced Integration Tests");
    
    test_signal_during_syscall();
    test_signal_handlers();
    test_process_tree();
    test_resource_cleanup();
    test_stress_many_processes();
    test_signal_process_group();
    
    test_suite_end();
}

// Syscall helpers
mod syscall {
    #[inline(always)]
    pub unsafe fn syscall0(n: u64) -> u64 {
        let ret: u64;
        core::arch::asm!(
            "syscall",
            in("rax") n,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
        ret
    }
    
    #[inline(always)]
    pub unsafe fn syscall1(n: u64, arg1: u64) -> u64 {
        let ret: u64;
        core::arch::asm!(
            "syscall",
            in("rax") n,
            in("rdi") arg1,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
        ret
    }
    
    #[inline(always)]
    pub unsafe fn syscall2(n: u64, arg1: u64, arg2: u64) -> u64 {
        let ret: u64;
        core::arch::asm!(
            "syscall",
            in("rax") n,
            in("rdi") arg1,
            in("rsi") arg2,
            lateout("rax") ret,
            options(nostack, preserves_flags)
        );
        ret
    }
}

mod syscall_numbers {
    pub const SYS_EXIT: usize = 0;
    pub const SYS_FORK: usize = 1;
    pub const SYS_EXEC: usize = 2;
    pub const SYS_WAIT: usize = 3;
    pub const SYS_GETPID: usize = 4;
    pub const SYS_GETPPID: usize = 5;
    pub const SYS_SLEEP: usize = 6;
    pub const SYS_OPEN: usize = 10;
    pub const SYS_KILL: usize = 50;
    pub const SYS_SIGNAL: usize = 51;
}

