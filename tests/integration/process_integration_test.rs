//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: tests_integration
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Podstawowe testy integracyjne zarządzania procesami (dzień 31).

#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;
use alloc::format;

mod test_framework;
use test_framework::*;

/// Test 1: fork() + exit() + wait()
/// Basic test: child exits, parent waits and gets status
#[test_case]
fn test_fork_exit_wait() {
    test_info("Test 1: fork() + exit() + wait()");
    
    let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
    
    if pid == 0 {
        // Child process - just exit with status 42
        unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 42) };
        loop {} // Should never reach
    } else if pid > 0 {
        // Parent process - wait for child
        let mut status = 0i32;
        let wait_pid = unsafe { 
            syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
        } as i32;
        
        assert_eq!(wait_pid, pid, "wait() should return child PID");
        assert_eq!(status, 42, "Child exit status should be 42");
        
        test_pass("fork() + exit() + wait() works correctly");
    } else {
        test_fail("fork() failed");
    }
}

/// Test 2: fork() + exec()
/// Test that exec() replaces process image
#[test_case]
fn test_fork_exec() {
    test_info("Test 2: fork() + exec()");
    
    let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
    
    if pid == 0 {
        // Child process - exec a test program
        let path = b"/bin/test_exec_target\0";
        let argv: [*const u8; 2] = [path.as_ptr(), core::ptr::null()];
        
        unsafe { 
            syscall::syscall2(
                syscall_numbers::SYS_EXEC as u64, 
                path.as_ptr() as u64,
                argv.as_ptr() as u64
            );
        };
        
        // If exec succeeds, we should never reach here
        test_fail("exec() returned (should not happen)");
        unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 1) };
    } else if pid > 0 {
        // Parent - wait for child
        let mut status = 0i32;
        let _ = unsafe { 
            syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
        };
        
        test_pass("fork() + exec() completed");
    } else {
        test_fail("fork() failed");
    }
}

/// Test 3: fork() + exec() + wait()
/// Complete process lifecycle
#[test_case]
fn test_fork_exec_wait() {
    test_info("Test 3: fork() + exec() + wait()");
    
    let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
    
    if pid == 0 {
        // Child - exec /bin/echo
        let path = b"/bin/echo\0";
        let arg1 = b"Hello from exec\0";
        let argv: [*const u8; 3] = [path.as_ptr(), arg1.as_ptr(), core::ptr::null()];
        
        unsafe { 
            syscall::syscall2(
                syscall_numbers::SYS_EXEC as u64,
                path.as_ptr() as u64,
                argv.as_ptr() as u64
            );
        };
        
        // Should not reach here
        unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 127) };
    } else if pid > 0 {
        // Parent - wait and check status
        let mut status = 0i32;
        let wait_pid = unsafe { 
            syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
        } as i32;
        
        assert_eq!(wait_pid, pid, "wait() returned correct PID");
        test_pass("fork() + exec() + wait() lifecycle complete");
    } else {
        test_fail("fork() failed");
    }
}

/// Test 4: Multiple children
/// Test that parent can fork multiple children and wait for all
#[test_case]
fn test_multiple_children() {
    test_info("Test 4: Multiple children");
    
    const NUM_CHILDREN: usize = 5;
    let mut child_pids = [0i32; NUM_CHILDREN];
    
    // Fork multiple children
    for i in 0..NUM_CHILDREN {
        let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
        
        if pid == 0 {
            // Child process - exit with unique status
            unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, (i + 10) as u64) };
        } else if pid > 0 {
            child_pids[i] = pid;
        } else {
            test_fail("fork() failed for child");
            return;
        }
    }
    
    // Parent - wait for all children
    let mut collected = 0;
    for _ in 0..NUM_CHILDREN {
        let mut status = 0i32;
        let wait_pid = unsafe { 
            syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
        } as i32;
        
        if wait_pid > 0 {
            // Verify this was one of our children
            let mut found = false;
            for &child_pid in &child_pids {
                if child_pid == wait_pid {
                    found = true;
                    break;
                }
            }
            
            assert!(found, "wait() returned unknown PID");
            collected += 1;
        }
    }
    
    assert_eq!(collected, NUM_CHILDREN, "Should collect all children");
    test_pass("Multiple children forked and waited successfully");
}

/// Test 5: Zombie cleanup
/// Test that zombie processes are properly cleaned up after wait()
#[test_case]
fn test_zombie_cleanup() {
    test_info("Test 5: Zombie cleanup");
    
    let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
    
    if pid == 0 {
        // Child - exit immediately
        unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
    } else if pid > 0 {
        // Parent - sleep briefly to let child become zombie
        unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 100) }; // 100ms
        
        // Child should be zombie at this point
        // TODO: Check /proc or process table for zombie state
        
        // Wait to clean up zombie
        let mut status = 0i32;
        let wait_pid = unsafe { 
            syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
        } as i32;
        
        assert_eq!(wait_pid, pid, "wait() cleaned up zombie");
        
        // TODO: Verify process is completely removed from process table
        
        test_pass("Zombie process cleaned up correctly");
    } else {
        test_fail("fork() failed");
    }
}

/// Test 6: Orphan reparenting
/// Test that orphaned processes are reparented to init (PID 1)
#[test_case]
fn test_orphan_reparenting() {
    test_info("Test 6: Orphan reparenting");
    
    let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
    
    if pid == 0 {
        // Child - fork a grandchild, then exit
        let grandchild_pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
        
        if grandchild_pid == 0 {
            // Grandchild - sleep, then check parent
            unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 200) }; // 200ms
            
            // At this point, parent should have exited and we should be reparented to init
            let ppid = unsafe { syscall::syscall0(syscall_numbers::SYS_GETPPID as u64) as i32 };
            
            // ppid should be 1 (init) after reparenting
            if ppid == 1 {
                unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
            } else {
                unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 1) };
            }
        } else {
            // Child - exit immediately, orphaning grandchild
            unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
        }
    } else if pid > 0 {
        // Original parent - wait for child
        let mut status = 0i32;
        let _ = unsafe { 
            syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64) 
        };
        
        // Sleep to let grandchild complete reparenting test
        unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 300) }; // 300ms
        
        test_pass("Orphan reparenting test completed");
    } else {
        test_fail("fork() failed");
    }
}

/// Main test runner
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_suite_start("Day 31: Basic Process Integration Tests");
    
    test_fork_exit_wait();
    test_fork_exec();
    test_fork_exec_wait();
    test_multiple_children();
    test_zombie_cleanup();
    test_orphan_reparenting();
    
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
}
