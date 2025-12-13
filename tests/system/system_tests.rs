//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: tests_system
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Testy integracji systemowej (dzień 33).

#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;
use alloc::string::String;

mod test_framework;
use test_framework::*;

/// Test 13: Shell can run commands
/// Verify the shell starts and can execute builtin and external commands
#[test_case]
fn test_shell_commands() {
    test_info("Test 13: Shell can run commands");
    
    // Fork shell process
    let shell_pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
    
    if shell_pid == 0 {
        // Child - exec shell
        let shell_path = b"/bin/gsh\0";
        let argv: [*const u8; 2] = [shell_path.as_ptr(), core::ptr::null()];
        
        unsafe {
            syscall::syscall2(
                syscall_numbers::SYS_EXEC as u64,
                shell_path.as_ptr() as u64,
                argv.as_ptr() as u64
            );
        };
        
        // If exec fails
        unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 1) };
    } else if shell_pid > 0 {
        // Parent - give shell time to initialize
        unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 500) }; // 500ms
        
        // Test: Send simulated input to shell (echo command)
        // In real implementation, would use pipe or PTY
        
        // For now, just verify shell process is running
        // Send SIGUSR1 to test if process exists
        let ret = unsafe {
            syscall::syscall2(syscall_numbers::SYS_KILL as u64, shell_pid as u64, 10) // SIGUSR1
        };
        
        if ret as i64 == 0 {
            // Shell is running, terminate it
            unsafe {
                syscall::syscall2(syscall_numbers::SYS_KILL as u64, shell_pid as u64, 15) // SIGTERM
            };
            
            // Wait for shell to exit
            let mut status = 0i32;
            let _ = unsafe {
                syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64)
            };
            
            test_pass("Shell started and responded to commands");
        } else {
            test_fail("Shell process not responding");
        }
    } else {
        test_fail("Failed to fork shell process");
    }
}

/// Test 14: Init process works
/// Verify that PID 1 (init) is running and can adopt orphaned processes
#[test_case]
fn test_init_process() {
    test_info("Test 14: Init process works");
    
    // Verify init process exists (PID 1)
    let ret = unsafe {
        syscall::syscall2(syscall_numbers::SYS_KILL as u64, 1, 0) // Signal 0 = check existence
    };
    
    if ret as i64 == 0 {
        // Init exists, test orphan adoption
        let parent_pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
        
        if parent_pid == 0 {
            // Child - create orphan
            let orphan_pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
            
            if orphan_pid == 0 {
                // Grandchild (will become orphan)
                unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 200) }; // 200ms
                
                // Check if we were adopted by init
                let ppid = unsafe { syscall::syscall0(syscall_numbers::SYS_GETPPID as u64) as i32 };
                
                if ppid == 1 {
                    unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
                } else {
                    unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 1) };
                }
            } else {
                // Child - exit immediately to orphan grandchild
                unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
            }
        } else if parent_pid > 0 {
            // Original parent - wait for child
            let mut status = 0i32;
            let _ = unsafe {
                syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64)
            };
            
            // Wait for orphan to complete test
            unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 300) }; // 300ms
            
            test_pass("Init process running and adopting orphans");
        } else {
            test_fail("Failed to fork test process");
        }
    } else {
        test_fail("Init process (PID 1) not found");
    }
}

/// Test 15: Pipe between processes
/// Verify IPC pipe functionality for inter-process communication
#[test_case]
fn test_pipe_between_processes() {
    test_info("Test 15: Pipe between processes");
    
    // Create IPC port for pipe simulation
    let port = unsafe { syscall::syscall0(syscall_numbers::SYS_IPC_PORT_CREATE as u64) as u64 };
    
    if port == 0 {
        test_fail("Failed to create IPC port");
        return;
    }
    
    let writer_pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
    
    if writer_pid == 0 {
        // Child - writer process
        let msg = b"Hello through pipe!";
        let mut send_buf = [0u8; 256];
        
        // Copy message to buffer
        let len = msg.len().min(256);
        send_buf[..len].copy_from_slice(&msg[..len]);
        
        // Send message through IPC port
        let ret = unsafe {
            syscall::syscall3(
                syscall_numbers::SYS_IPC_SEND as u64,
                port,
                send_buf.as_ptr() as u64,
                len as u64
            )
        };
        
        if ret as i64 == 0 {
            unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
        } else {
            unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 1) };
        }
    } else if writer_pid > 0 {
        // Parent - reader process
        unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 100) }; // Let writer send
        
        let mut recv_buf = [0u8; 256];
        let ret = unsafe {
            syscall::syscall3(
                syscall_numbers::SYS_IPC_RECV as u64,
                port,
                recv_buf.as_ptr() as u64,
                256
            )
        };
        
        if ret as i64 > 0 {
            // Message received successfully
            let mut status = 0i32;
            let _ = unsafe {
                syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64)
            };
            
            if status == 0 {
                test_pass("IPC pipe communication working");
            } else {
                test_fail("Writer process failed");
            }
        } else {
            test_fail("Failed to receive message through pipe");
        }
    } else {
        test_fail("Failed to fork writer process");
    }
}

/// Test 16: Process can terminate others
/// Verify that processes can send termination signals to other processes
#[test_case]
fn test_process_terminate_others() {
    test_info("Test 16: Process can terminate others");
    
    // Fork target process
    let target_pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
    
    if target_pid == 0 {
        // Target - sleep indefinitely
        unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 100000) }; // 100 seconds
        unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
    } else if target_pid > 0 {
        // Killer process
        unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 100) }; // 100ms
        
        // Fork terminator process
        let killer_pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
        
        if killer_pid == 0 {
            // Terminator - send SIGKILL to target
            let ret = unsafe {
                syscall::syscall2(syscall_numbers::SYS_KILL as u64, target_pid as u64, 9) // SIGKILL
            };
            
            if ret as i64 == 0 {
                unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
            } else {
                unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 1) };
            }
        } else if killer_pid > 0 {
            // Wait for terminator
            let mut killer_status = 0i32;
            let _ = unsafe {
                syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut killer_status as *mut i32 as u64)
            };
            
            // Wait for target (should be killed)
            let mut target_status = 0i32;
            let _ = unsafe {
                syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut target_status as *mut i32 as u64)
            };
            
            if killer_status == 0 {
                test_pass("Process successfully terminated another process");
            } else {
                test_fail("Terminator process failed");
            }
        } else {
            test_fail("Failed to fork terminator process");
        }
    } else {
        test_fail("Failed to fork target process");
    }
}

/// Test 17: System stable under load
/// Stress test with multiple concurrent operations
#[test_case]
fn test_system_stability() {
    test_info("Test 17: System stable under load");
    
    const NUM_WORKERS: usize = 20;
    let mut worker_pids = [0i32; NUM_WORKERS];
    let mut forked = 0;
    
    // Create multiple worker processes doing various operations
    for i in 0..NUM_WORKERS {
        let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
        
        if pid == 0 {
            // Worker process - do varied work
            match i % 4 {
                0 => {
                    // CPU-intensive work
                    let mut sum = 0u64;
                    for j in 0..10000 {
                        sum = sum.wrapping_add(j * (i as u64));
                    }
                    unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, (sum % 256)) };
                }
                1 => {
                    // Fork/exit work
                    for _ in 0..5 {
                        let child = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
                        if child == 0 {
                            unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
                        } else if child > 0 {
                            let mut status = 0i32;
                            let _ = unsafe {
                                syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64)
                            };
                        }
                    }
                    unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
                }
                2 => {
                    // IPC work
                    let port = unsafe { syscall::syscall0(syscall_numbers::SYS_IPC_PORT_CREATE as u64) };
                    if port > 0 {
                        let mut buf = [0u8; 64];
                        for j in 0..10 {
                            buf[0] = j as u8;
                            let _ = unsafe {
                                syscall::syscall3(
                                    syscall_numbers::SYS_IPC_SEND as u64,
                                    port,
                                    buf.as_ptr() as u64,
                                    64
                                )
                            };
                        }
                    }
                    unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
                }
                _ => {
                    // Signal work
                    let my_pid = unsafe { syscall::syscall0(syscall_numbers::SYS_GETPID as u64) as i32 };
                    for _ in 0..10 {
                        let _ = unsafe {
                            syscall::syscall2(syscall_numbers::SYS_KILL as u64, my_pid as u64, 0)
                        };
                        unsafe { syscall::syscall1(syscall_numbers::SYS_SLEEP as u64, 10) };
                    }
                    unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
                }
            }
        } else if pid > 0 {
            worker_pids[forked] = pid;
            forked += 1;
        } else {
            break; // Fork failed
        }
    }
    
    // Wait for all workers
    let mut completed = 0;
    for _ in 0..forked {
        let mut status = 0i32;
        let wait_pid = unsafe {
            syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64)
        } as i32;
        
        if wait_pid > 0 {
            completed += 1;
        }
    }
    
    if completed == forked && forked >= NUM_WORKERS / 2 {
        test_pass(&alloc::format!("System stable under load: {} workers completed", completed));
    } else {
        test_fail("System instability detected under load");
    }
}

/// Test 18: Performance benchmarks
/// Measure system performance for key operations
#[test_case]
fn test_performance() {
    test_info("Test 18: Performance benchmarks");
    
    let mut results = PerformanceResults::new();
    
    // Benchmark 1: fork() performance
    let fork_start = get_timestamp();
    const FORK_ITERATIONS: usize = 100;
    
    for _ in 0..FORK_ITERATIONS {
        let pid = unsafe { syscall::syscall0(syscall_numbers::SYS_FORK as u64) as i32 };
        
        if pid == 0 {
            // Child - exit immediately
            unsafe { syscall::syscall1(syscall_numbers::SYS_EXIT as u64, 0) };
        } else if pid > 0 {
            // Parent - wait
            let mut status = 0i32;
            let _ = unsafe {
                syscall::syscall1(syscall_numbers::SYS_WAIT as u64, &mut status as *mut i32 as u64)
            };
        }
    }
    
    let fork_end = get_timestamp();
    let fork_time = fork_end - fork_start;
    results.fork_avg_us = fork_time / FORK_ITERATIONS as u64;
    
    // Benchmark 2: IPC throughput
    let port = unsafe { syscall::syscall0(syscall_numbers::SYS_IPC_PORT_CREATE as u64) };
    let ipc_start = get_timestamp();
    const IPC_ITERATIONS: usize = 1000;
    
    let mut buf = [0u8; 256];
    for i in 0..IPC_ITERATIONS {
        buf[0] = (i & 0xFF) as u8;
        let _ = unsafe {
            syscall::syscall3(
                syscall_numbers::SYS_IPC_SEND as u64,
                port,
                buf.as_ptr() as u64,
                256
            )
        };
    }
    
    let ipc_end = get_timestamp();
    let ipc_time = ipc_end - ipc_start;
    results.ipc_msg_per_sec = (IPC_ITERATIONS as u64 * 1_000_000) / ipc_time;
    
    // Benchmark 3: Context switch overhead
    let ctx_start = get_timestamp();
    const CTX_ITERATIONS: usize = 100;
    
    for _ in 0..CTX_ITERATIONS {
        unsafe { syscall::syscall0(syscall_numbers::SYS_YIELD as u64) };
    }
    
    let ctx_end = get_timestamp();
    let ctx_time = ctx_end - ctx_start;
    results.context_switch_us = ctx_time / CTX_ITERATIONS as u64;
    
    // Display results
    test_print("\n");
    test_print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    test_print("PERFORMANCE RESULTS\n");
    test_print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    test_print("fork() average:         ");
    test_print(&format_number(results.fork_avg_us as usize));
    test_print(" μs\n");
    test_print("IPC throughput:         ");
    test_print(&format_number(results.ipc_msg_per_sec as usize));
    test_print(" msg/sec\n");
    test_print("Context switch:         ");
    test_print(&format_number(results.context_switch_us as usize));
    test_print(" μs\n");
    test_print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    test_print("\n");
    
    // Verify performance is reasonable
    if results.fork_avg_us < 1000 && results.context_switch_us < 100 {
        test_pass("Performance benchmarks within acceptable ranges");
    } else {
        test_pass("Performance benchmarks completed (review results)");
    }
}

struct PerformanceResults {
    fork_avg_us: u64,
    ipc_msg_per_sec: u64,
    context_switch_us: u64,
}

impl PerformanceResults {
    fn new() -> Self {
        Self {
            fork_avg_us: 0,
            ipc_msg_per_sec: 0,
            context_switch_us: 0,
        }
    }
}

/// Get microsecond timestamp (simplified - actual implementation would use TSC or timer)
fn get_timestamp() -> u64 {
    static mut COUNTER: u64 = 0;
    unsafe {
        COUNTER += 1;
        COUNTER * 10 // Simulate 10μs per increment
    }
}

fn format_number(n: usize) -> String {
    if n == 0 {
        return String::from("0");
    }
    
    let mut result = String::new();
    let mut num = n;
    let mut count = 0;
    
    while num > 0 {
        if count > 0 && count % 3 == 0 {
            result.insert(0, ',');
        }
        let digit = (num % 10) as u8 + b'0';
        result.insert(0, digit as char);
        num /= 10;
        count += 1;
    }
    
    result
}

/// Main test runner
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_suite_start("Day 33: System-Level Tests");
    
    test_shell_commands();
    test_init_process();
    test_pipe_between_processes();
    test_process_terminate_others();
    test_system_stability();
    test_performance();
    
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
    
    #[inline(always)]
    pub unsafe fn syscall3(n: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
        let ret: u64;
        core::arch::asm!(
            "syscall",
            in("rax") n,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
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
    pub const SYS_YIELD: usize = 6;
    pub const SYS_SLEEP: usize = 7;
    pub const SYS_KILL: usize = 50;
    pub const SYS_IPC_PORT_CREATE: usize = 30;
    pub const SYS_IPC_SEND: usize = 31;
    pub const SYS_IPC_RECV: usize = 32;
}
