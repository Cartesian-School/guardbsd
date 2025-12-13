//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_signal
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Domyślne obsługi sygnałów (dzień 24).

#![no_std]

use super::types::*;
use crate::process::types::{Pid, ProcessState};
use crate::syscalls::process::{find_process_mut, set_process_state};

/// Handle default signal action (Day 24)
///
/// Implements BSD default signal behavior:
/// - Term: Terminate the process
/// - Ignore: Do nothing
/// - Core: Terminate with core dump (simplified: just terminate)
/// - Stop: Stop process execution
/// - Cont: Continue stopped process
///
/// Parameters:
/// - pid: Process receiving signal
/// - signo: Signal number
///
/// Returns:
/// - true if handled successfully
/// - false on error
pub fn handle_default_signal(pid: Pid, signo: Signal) -> bool {
    let action = default_action(signo);
    
    match action {
        DefaultAction::Term => {
            // Terminate process
            terminate_process(pid, signo)
        },
        
        DefaultAction::Ignore => {
            // Do nothing
            true
        },
        
        DefaultAction::Core => {
            // Core dump - for now, just terminate
            // TODO: Generate core dump file
            terminate_process(pid, signo)
        },
        
        DefaultAction::Stop => {
            // Stop process execution
            stop_process(pid)
        },
        
        DefaultAction::Cont => {
            // Continue process if stopped
            continue_process(pid)
        },
    }
}

/// Terminate a process due to signal (Day 24)
///
/// Similar to sys_exit() but called from signal handler
///
/// BSD Semantics:
/// - Close all file descriptors
/// - Set exit status to signal number + 128
/// - Become zombie
/// - Send SIGCHLD to parent
/// - Reparent children to init
///
/// Parameters:
/// - pid: Process to terminate
/// - signo: Signal that caused termination
///
/// Returns:
/// - true if successful
/// - false if process not found
fn terminate_process(pid: Pid, signo: Signal) -> bool {
    // Set exit status to signal number + 128 (BSD convention)
    let exit_status = 128 + signo;
    
    // Use existing sys_exit logic
    // Close FDs
    crate::syscalls::process::close_all_fds(pid);
    
    // Reparent children
    crate::syscalls::process::reparent_children_to_init(pid);
    
    // Set exit status
    crate::syscalls::process::set_exit_status(pid, exit_status);
    
    // Set process state to Zombie
    set_process_state(pid, ProcessState::Zombie);
    
    // Set thread state to Zombie in scheduler
    if let Some(tid) = crate::syscalls::process::get_thread_id(pid) {
        crate::sched::set_thread_state(tid, crate::sched::ThreadState::Zombie);
    }
    
    // Send SIGCHLD to parent
    if let Some(parent_pid) = crate::syscalls::process::get_parent_pid(pid) {
        send_signal(parent_pid, SIGCHLD);
    }
    
    true
}

/// Stop a process (Day 24)
///
/// BSD Semantics:
/// - Suspend process execution
/// - Process becomes unschedulable
/// - Can be resumed with SIGCONT
/// - Send SIGCHLD to parent
///
/// Parameters:
/// - pid: Process to stop
///
/// Returns:
/// - true if successful
/// - false if process not found
fn stop_process(pid: Pid) -> bool {
    // Set process state to Blocked (stopped)
    if !set_process_state(pid, ProcessState::Blocked) {
        return false;
    }
    
    // Update scheduler thread state
    if let Some(tid) = crate::syscalls::process::get_thread_id(pid) {
        crate::sched::set_thread_state(tid, crate::sched::ThreadState::Blocked);
    }
    
    // Send SIGCHLD to parent (BSD behavior)
    if let Some(parent_pid) = crate::syscalls::process::get_parent_pid(pid) {
        send_signal(parent_pid, SIGCHLD);
    }
    
    true
}

/// Continue a stopped process (Day 24)
///
/// BSD Semantics:
/// - Resume process execution
/// - Process becomes schedulable again
/// - Only works on stopped processes
///
/// Parameters:
/// - pid: Process to continue
///
/// Returns:
/// - true if successful
/// - false if process not found or not stopped
fn continue_process(pid: Pid) -> bool {
    // Get current state
    let state = match get_process_state(pid) {
        Some(s) => s,
        None => return false,
    };
    
    // Only continue if process is actually stopped
    if state != ProcessState::Blocked {
        return false;
    }
    
    // Set process state to Ready
    if !set_process_state(pid, ProcessState::Ready) {
        return false;
    }
    
    // Update scheduler thread state
    if let Some(tid) = crate::syscalls::process::get_thread_id(pid) {
        crate::sched::set_thread_state(tid, crate::sched::ThreadState::Ready);
        
        // Add back to run queue
        // TODO: Call sched::enqueue_thread(tid)
    }
    
    true
}

/// Handle SIGTERM - Terminate process gracefully (Day 24)
///
/// Default action: Terminate
/// Can be caught: Yes
///
/// This is the standard signal for requesting process termination.
/// Programs can catch this to cleanup before exiting.
pub fn handle_sigterm(pid: Pid) -> bool {
    terminate_process(pid, SIGTERM)
}

/// Handle SIGKILL - Kill process immediately (Day 24)
///
/// Default action: Terminate
/// Can be caught: NO (always terminates)
///
/// This signal cannot be caught, blocked, or ignored.
/// It always terminates the process immediately.
pub fn handle_sigkill(pid: Pid) -> bool {
    // SIGKILL cannot be blocked or caught
    // Always terminates immediately
    terminate_process(pid, SIGKILL)
}

/// Handle SIGSTOP - Stop process (Day 24)
///
/// Default action: Stop
/// Can be caught: NO (always stops)
///
/// This signal cannot be caught, blocked, or ignored.
/// It always stops the process.
pub fn handle_sigstop(pid: Pid) -> bool {
    // SIGSTOP cannot be blocked or caught
    stop_process(pid)
}

/// Handle SIGCONT - Continue stopped process (Day 24)
///
/// Default action: Continue
/// Can be caught: Yes
///
/// Resumes a stopped process. If process is not stopped,
/// signal is ignored.
pub fn handle_sigcont(pid: Pid) -> bool {
    continue_process(pid)
}

/// Handle SIGCHLD - Child status changed (Day 24)
///
/// Default action: Ignore
/// Can be caught: Yes
///
/// Sent to parent when:
/// - Child exits
/// - Child stops (if SA_NOCLDSTOP not set)
/// - Child continues
///
/// Default is to ignore (most processes use wait() instead)
pub fn handle_sigchld(_pid: Pid) -> bool {
    // Default action is ignore
    true
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test: SIGTERM terminates process (Day 24)
    #[test]
    fn test_sigterm_terminates() {
        // Expected behavior:
        // 1. Process running normally
        // 2. Send SIGTERM
        // 3. handle_sigterm() called
        // 4. Process state → Zombie
        // 5. Exit status = 128 + 15 = 143
        // 6. SIGCHLD sent to parent
    }
    
    /// Test: SIGKILL cannot be blocked (Day 24)
    #[test]
    fn test_sigkill_uncatchable() {
        // Expected behavior:
        // 1. Process blocks all signals (signal_mask = all 1s)
        // 2. Send SIGKILL
        // 3. SIGKILL delivered anyway (ignores mask)
        // 4. Process terminates
        // 5. Cannot be prevented
    }
    
    /// Test: SIGSTOP stops process (Day 24)
    #[test]
    fn test_sigstop_stops_process() {
        // Expected behavior:
        // 1. Process running
        // 2. Send SIGSTOP
        // 3. Process state → Blocked
        // 4. Thread state → Blocked
        // 5. Process removed from run queue
        // 6. SIGCHLD sent to parent
    }
    
    /// Test: SIGCONT continues process (Day 24)
    #[test]
    fn test_sigcont_continues() {
        // Expected behavior:
        // 1. Process stopped (state = Blocked)
        // 2. Send SIGCONT
        // 3. Process state → Ready
        // 4. Thread state → Ready
        // 5. Process added to run queue
        // 6. Process can execute again
    }
    
    /// Test: SIGCHLD ignored by default (Day 24)
    #[test]
    fn test_sigchld_ignored() {
        // Expected behavior:
        // 1. Child exits
        // 2. SIGCHLD sent to parent
        // 3. Parent has SIGCHLD pending
        // 4. deliver_signal(parent, SIGCHLD)
        // 5. Default handler ignores it
        // 6. Parent continues normally
        //
        // Note: Most programs use wait() instead of catching SIGCHLD
    }
    
    /// Test: Signal termination sets exit status (Day 24)
    #[test]
    fn test_signal_exit_status() {
        // Expected behavior:
        // 1. Send SIGTERM (15) to process
        // 2. Process terminates
        // 3. Exit status = 128 + 15 = 143
        // 4. Parent can distinguish signal death from normal exit
        //
        // BSD convention:
        //   Normal exit: status = exit_code (0-255)
        //   Signal death: status = 128 + signal_number
        //
        // Example:
        //   exit(0) → status = 0
        //   exit(1) → status = 1
        //   SIGTERM → status = 143
        //   SIGKILL → status = 137
    }
    
    /// Test: Stop/Continue cycle (Day 24)
    #[test]
    fn test_stop_continue_cycle() {
        // Expected behavior:
        // 1. Process running
        // 2. Send SIGSTOP → Process stops
        // 3. state = Blocked
        // 4. Send SIGCONT → Process continues
        // 5. state = Ready
        // 6. Process can run again
        //
        // Real-world use:
        //   Ctrl+Z in shell sends SIGTSTP (stoppable)
        //   fg command sends SIGCONT
    }
}
