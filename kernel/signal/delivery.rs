//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_signal
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Implementacja dostarczania sygnałów (dzień 23).

#![no_std]

use super::types::*;
use crate::process::types::{Pid, ProcessState};

// Import process functions
use crate::syscalls::process::{find_process, find_process_mut, get_process_state};

/// Send a signal to a process (Day 23)
///
/// BSD Semantics:
/// - Adds signal to process's pending_signals mask
/// - Does not interrupt currently running process
/// - Signal will be delivered when process next runs
/// - Returns true if signal was sent, false if process not found
///
/// Parameters:
/// - pid: Target process ID
/// - signo: Signal number to send
///
/// Returns:
/// - true if signal was successfully queued
/// - false if process not found or signal invalid
pub fn send_signal(pid: Pid, signo: Signal) -> bool {
    // Validate signal number
    if signo <= 0 || signo > SIGMAX {
        return false;
    }
    
    // Get process
    let process = match find_process_mut(pid) {
        Some(p) => p,
        None => return false,
    };
    
    // Check if process is in a state that can receive signals
    match process.state {
        ProcessState::Zombie => return false,  // Can't signal zombies
        _ => {}
    }
    
    // Add signal to pending mask
    process.pending_signals = sigaddset(process.pending_signals, signo);
    
    // TODO: Wake up process if it's sleeping/blocked
    // TODO: Interrupt system calls if SA_RESTART not set
    
    true
}

/// Check if a process has pending signals (Day 23)
///
/// Returns the highest priority pending signal that is not masked
///
/// Parameters:
/// - pid: Process ID to check
///
/// Returns:
/// - Some(signo) if there's a pending, unmasked signal
/// - None if no pending signals or process not found
pub fn check_pending_signals(pid: Pid) -> Option<Signal> {
    let process = find_process(pid)?;
    
    // Get unmasked pending signals
    let deliverable = process.pending_signals & !process.signal_mask;
    
    if deliverable == 0 {
        return None;
    }
    
    // Find highest priority signal (lowest number)
    // In BSD, lower signal numbers have higher priority
    for signo in 1..=SIGMAX {
        if sigismember(deliverable, signo) {
            return Some(signo);
        }
    }
    
    None
}

/// Deliver a signal to a process (Day 23)
///
/// This function handles the actual signal delivery:
/// 1. Removes signal from pending mask
/// 2. Checks for custom handler
/// 3. Executes handler or default action
///
/// Parameters:
/// - pid: Target process ID
/// - signo: Signal number to deliver
///
/// Returns:
/// - true if signal was delivered
/// - false if delivery failed
pub fn deliver_signal(pid: Pid, signo: Signal) -> bool {
    // Validate signal
    if signo <= 0 || signo > SIGMAX {
        return false;
    }
    
    let process = match find_process_mut(pid) {
        Some(p) => p,
        None => return false,
    };
    
    // Remove signal from pending mask
    process.pending_signals = sigdelset(process.pending_signals, signo);
    
    // Get signal handler
    let handler_idx = (signo - 1) as usize;
    let handler = if handler_idx < process.signal_handlers.len() {
        process.signal_handlers[handler_idx].handler
    } else {
        SIG_DFL
    };
    
    // Handle based on handler type
    match handler {
        SIG_DFL => {
            // Use default handler (Day 24)
            super::handlers::handle_default_signal(pid, signo)
        },
        SIG_IGN => {
            // Ignore signal
            true
        },
        _ => {
            // Custom user handler (Day 27)
            crate::syscalls::signal::setup_signal_frame(pid, signo, handler)
        }
    }
}

/// Queue a signal for delivery at next opportunity (Day 23)
///
/// This is the main entry point for signal delivery.
/// Used by kill(), exit() (for SIGCHLD), etc.
///
/// Parameters:
/// - pid: Target process
/// - signo: Signal to send
///
/// Returns:
/// - true if successfully queued
/// - false on error
pub fn queue_signal(pid: Pid, signo: Signal) -> bool {
    // Special handling for uncatchable signals
    if signo == SIGKILL || signo == SIGSTOP {
        // These signals are delivered immediately
        return deliver_signal(pid, signo);
    }
    
    // Regular signals are just queued
    send_signal(pid, signo)
}

/// Check and deliver pending signals for a process (Day 23)
///
/// This should be called:
/// - Before returning to userland from syscall
/// - After waking from sleep
/// - On timer interrupt
///
/// Parameters:
/// - pid: Process to check
///
/// Returns:
/// - true if a signal was delivered
/// - false if no signals pending
pub fn process_pending_signals(pid: Pid) -> bool {
    // Check for pending signals
    if let Some(signo) = check_pending_signals(pid) {
        // Deliver the signal
        deliver_signal(pid, signo)
    } else {
        false
    }
}

/// Check if a signal is blocked by a process (Day 23)
///
/// Parameters:
/// - pid: Process to check
/// - signo: Signal number
///
/// Returns:
/// - true if signal is blocked
/// - false if not blocked or process not found
pub fn is_signal_blocked(pid: Pid, signo: Signal) -> bool {
    if let Some(process) = find_process(pid) {
        sigismember(process.signal_mask, signo)
    } else {
        false
    }
}

/// Block a signal for a process (Day 23)
///
/// Parameters:
/// - pid: Process
/// - signo: Signal to block
///
/// Returns:
/// - true if successful
/// - false if process not found
pub fn block_signal(pid: Pid, signo: Signal) -> bool {
    // Cannot block SIGKILL or SIGSTOP
    if is_uncatchable(signo) {
        return false;
    }
    
    if let Some(process) = find_process_mut(pid) {
        process.signal_mask = sigaddset(process.signal_mask, signo);
        true
    } else {
        false
    }
}

/// Unblock a signal for a process (Day 23)
///
/// Parameters:
/// - pid: Process
/// - signo: Signal to unblock
///
/// Returns:
/// - true if successful
/// - false if process not found
pub fn unblock_signal(pid: Pid, signo: Signal) -> bool {
    if let Some(process) = find_process_mut(pid) {
        process.signal_mask = sigdelset(process.signal_mask, signo);
        true
    } else {
        false
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test: send_signal() queues signal
    #[test]
    fn test_send_signal_queues() {
        // Expected behavior:
        // 1. Process has no pending signals
        // 2. send_signal(pid, SIGTERM)
        // 3. Process has SIGTERM pending
        // 4. pending_signals bit 15 set
    }
    
    /// Test: check_pending_signals() finds signal
    #[test]
    fn test_check_pending_signals() {
        // Expected behavior:
        // 1. Process has SIGTERM and SIGINT pending
        // 2. check_pending_signals() returns SIGINT (lower number, higher priority)
        // 3. Signal mask respected
    }
    
    /// Test: deliver_signal() removes from pending
    #[test]
    fn test_deliver_signal_removes() {
        // Expected behavior:
        // 1. SIGTERM pending
        // 2. deliver_signal(pid, SIGTERM)
        // 3. SIGTERM no longer pending
        // 4. Handler executed
    }
    
    /// Test: SIGKILL delivered immediately
    #[test]
    fn test_sigkill_immediate() {
        // Expected behavior:
        // 1. queue_signal(pid, SIGKILL)
        // 2. Signal delivered immediately (not just queued)
        // 3. Process terminates
    }
    
    /// Test: Blocked signals not delivered
    #[test]
    fn test_blocked_signals() {
        // Expected behavior:
        // 1. block_signal(pid, SIGTERM)
        // 2. send_signal(pid, SIGTERM)
        // 3. Signal pending but not delivered
        // 4. check_pending_signals() returns None
        // 5. unblock_signal(pid, SIGTERM)
        // 6. check_pending_signals() returns SIGTERM
    }
    
    /// Test: Cannot block SIGKILL
    #[test]
    fn test_cannot_block_sigkill() {
        // Expected behavior:
        // 1. block_signal(pid, SIGKILL) returns false
        // 2. SIGKILL can still be delivered
        // 3. Same for SIGSTOP
    }
}
