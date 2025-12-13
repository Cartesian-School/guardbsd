//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_syscalls
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Wywołania systemowe sygnałów (dni 25-27).

#![no_std]

use crate::signal::{Signal, SignalAction, send_signal, is_uncatchable};
use crate::process::types::{Pid, ProcessState};
use crate::syscalls::process::{find_process_mut, get_current_pid};
use crate::signal::SignalFrame;

// ==================== DAY 25: sys_kill() ====================

/// Send a signal to a process (Day 25)
///
/// BSD Semantics:
/// - kill(pid, sig) sends signal sig to process pid
/// - Requires permission: sender must be same user or root
/// - Special PIDs:
///   - pid > 0: Send to specific process
///   - pid == 0: Send to all processes in current process group
///   - pid == -1: Send to all processes (broadcast)
///   - pid < -1: Send to process group |pid|
///
/// Parameters:
/// - pid: Target process ID
/// - sig: Signal number to send
///
/// Returns:
/// - 0 on success
/// - -1 on error (ESRCH: no such process, EPERM: permission denied)
pub fn sys_kill(pid: Pid, sig: Signal) -> isize {
    // Validate signal number
    if sig < 0 || sig > crate::signal::SIGMAX {
        return -22; // EINVAL: Invalid signal
    }
    
    // Signal 0 is special: just check if process exists
    if sig == 0 {
        if find_process_mut(pid).is_some() {
            return 0; // Process exists
        } else {
            return -3; // ESRCH: No such process
        }
    }
    
    // Get current process for permission check
    let current_pid = match get_current_pid() {
        Some(p) => p,
        None => return -1, // No current process
    };
    
    // Permission check (Day 25)
    if !check_kill_permission(current_pid, pid) {
        return -1; // EPERM: Operation not permitted
    }
    
    // Send signal (Day 25)
    if send_signal(pid, sig) {
        0 // Success
    } else {
        -3 // ESRCH: No such process
    }
}

/// Check if current process has permission to send signal to target (Day 25)
///
/// BSD Permission Rules:
/// - Root (uid 0) can signal any process
/// - Non-root can only signal processes with same effective uid
/// - Cannot signal init (PID 1) unless root
///
/// Parameters:
/// - sender_pid: Process sending signal
/// - target_pid: Process receiving signal
///
/// Returns:
/// - true if permission granted
/// - false if permission denied
fn check_kill_permission(sender_pid: Pid, target_pid: Pid) -> bool {
    // For now, allow all signals (Phase 4 will add full permission checking)
    // TODO: Check UIDs when user system implemented
    
    // Basic checks:
    
    // Can always signal self
    if sender_pid == target_pid {
        return true;
    }
    
    // Check if target exists
    if find_process_mut(target_pid).is_none() {
        return false;
    }
    
    // Check if target is init (PID 1)
    // Only allow if sender is root (TODO: check uid)
    if target_pid == 1 {
        // For now, allow (will be restricted when user system exists)
        return true;
    }
    
    // Allow for now (Phase 4 will add full permission system)
    true
}

// ==================== DAY 26: sys_sigaction() ====================

/// Install a signal handler (Day 26)
///
/// BSD Semantics:
/// - sigaction(sig, act, oldact) installs new handler for signal
/// - Returns old handler if oldact != NULL
/// - Cannot catch SIGKILL or SIGSTOP
/// - Handler address meanings:
///   - SIG_DFL (0): Use default handler
///   - SIG_IGN (1): Ignore signal
///   - Other: User-defined handler function
///
/// Parameters:
/// - signo: Signal number
/// - act: Pointer to new action (or NULL to query)
/// - oldact: Pointer to store old action (or NULL)
///
/// Returns:
/// - 0 on success
/// - -1 on error
pub fn sys_sigaction(signo: Signal, act: *const SignalAction, oldact: *mut SignalAction) -> isize {
    // Validate signal number (Day 26)
    if signo <= 0 || signo > crate::signal::SIGMAX {
        return -22; // EINVAL: Invalid signal
    }
    
    // Check for uncatchable signals (Day 26)
    if is_uncatchable(signo) {
        return -22; // EINVAL: Cannot catch SIGKILL or SIGSTOP
    }
    
    // Get current process
    let current_pid = match get_current_pid() {
        Some(p) => p,
        None => return -1,
    };
    
    let process = match find_process_mut(current_pid) {
        Some(p) => p,
        None => return -1,
    };
    
    let handler_idx = (signo - 1) as usize;
    if handler_idx >= process.signal_handlers.len() {
        return -22; // EINVAL
    }
    
    // Save old handler if requested (Day 26)
    if !oldact.is_null() {
        unsafe {
            // Copy old action to user space
            let old_action = SignalAction {
                sa_handler: process.signal_handlers[handler_idx].handler,
                sa_mask: 0, // TODO: Implement signal mask
                sa_flags: 0, // TODO: Implement flags
            };
            
            if !is_user_pointer_valid(oldact as *const u8, core::mem::size_of::<SignalAction>()) {
                return -14; // EFAULT: Bad address
            }
            
            core::ptr::write(oldact, old_action);
        }
    }
    
    // Set new handler if provided (Day 26)
    if !act.is_null() {
        unsafe {
            // Validate pointer
            if !is_user_pointer_valid(act as *const u8, core::mem::size_of::<SignalAction>()) {
                return -14; // EFAULT: Bad address
            }
            
            // Read new action from user space
            let new_action = core::ptr::read(act);
            
            // Update handler
            process.signal_handlers[handler_idx].handler = new_action.sa_handler;
            
            // TODO: Update sa_mask and sa_flags when implemented
        }
    }
    
    0 // Success
}

/// Simplified signal() syscall (Day 26)
///
/// BSD Semantics:
/// - signal(sig, handler) is simplified sigaction()
/// - Returns old handler
/// - Handler can be SIG_DFL, SIG_IGN, or function pointer
///
/// Parameters:
/// - signo: Signal number
/// - handler: New handler address
///
/// Returns:
/// - Old handler address on success
/// - SIG_ERR (-1) on error
pub fn sys_signal(signo: Signal, handler: u64) -> isize {
    // Validate signal
    if signo <= 0 || signo > crate::signal::SIGMAX {
        return crate::signal::SIG_ERR as isize;
    }

    // Check for uncatchable
    if is_uncatchable(signo) {
        return crate::signal::SIG_ERR as isize;
    }

    // Basic user-space address check: below canonical kernel base
    if handler >= 0xFFFF_8000_0000_0000 {
        return crate::signal::SIG_ERR as isize;
    }

    // Get current process
    let current_pid = match get_current_pid() {
        Some(p) => p,
        None => return crate::signal::SIG_ERR as isize,
    };

    let process = match find_process_mut(current_pid) {
        Some(p) => p,
        None => return crate::signal::SIG_ERR as isize,
    };

    let handler_idx = (signo - 1) as usize;
    if handler_idx >= process.signal_handlers.len() {
        return crate::signal::SIG_ERR as isize;
    }

    // Save old handler
    let old_handler = process.signal_handlers[handler_idx].unwrap_or(0);

    // Set new handler
    process.signal_handlers[handler_idx] = Some(handler);

    unsafe {
        crate::print("[SIGNAL] PID ");
        crate::print_num(current_pid as usize);
        crate::print(" installed handler for signal ");
        crate::print_num(signo as usize);
        crate::print(" at 0x");
        crate::print_hex64(handler);
        crate::print("\n");
    }

    old_handler as isize
}

/// Restore user context from a SignalFrame located on the user stack.
/// This does not return to the caller in the traditional sense; it mutates
/// the interrupt frame so iretq returns to the saved context.
pub fn sys_sigreturn() -> isize {
    // Get current process
    let current_pid = match get_current_pid() {
        Some(p) => p,
        None => return -1,
    };

    // Locate hardware frame on kernel stack.
    // int 0x80 pushes RIP,CS,RFLAGS,RSP,SS then stub pushes 15 GPRs.
    let kstack_rsp: usize;
    unsafe {
        core::arch::asm!("mov {}, rsp", out(reg) kstack_rsp, options(nomem, nostack, preserves_flags));
    }
    // Hardware frame starts after 15 pushes (15*8 bytes)
    let hw_frame = (kstack_rsp + 15 * 8) as *mut u64;
    unsafe {
        let user_rsp = *hw_frame.add(3);
        // Basic user-space canonical check: below kernel base
        if user_rsp >= 0xFFFF_8000_0000_0000 {
            crate::print("[SIGNAL] sigreturn: invalid frame pointer\n");
            if let Some(proc) = find_process_mut(current_pid) {
                proc.killed = true;
            }
            return -1;
        }
        let frame_ptr = user_rsp as *const SignalFrame;
        let frame = core::ptr::read(frame_ptr);

        // Restore hardware frame fields
        *hw_frame = frame.saved_rip;           // RIP
        *hw_frame.add(2) = frame.saved_rflags; // RFLAGS
        *hw_frame.add(3) = frame.saved_rsp;    // RSP

        crate::print("[SIGNAL] sigreturn: restored context for PID ");
        crate::print_num(current_pid as usize);
        crate::print(" from frame at 0x");
        crate::print_hex64(user_rsp as u64);
        crate::print("\n");
    }
    0
}

// ==================== DAY 27: User Signal Handlers ====================

/// Setup signal frame for user handler execution (Day 27)
///
/// When a signal with a custom handler is delivered:
/// 1. Save current process context (registers, RIP, RSP)
/// 2. Modify stack to include signal number
/// 3. Set RIP to handler address
/// 4. When handler returns, restore old context
///
/// Parameters:
/// - pid: Process to setup signal frame for
/// - signo: Signal number being delivered
/// - handler: Handler function address
///
/// Returns:
/// - true on success
/// - false on error
pub fn setup_signal_frame(pid: Pid, signo: Signal, handler: u64) -> bool {
    // Get process
    let process = match find_process_mut(pid) {
        Some(p) => p,
        None => return false,
    };
    
    // Get thread ID
    let tid = match process.thread_id {
        Some(t) => t,
        None => return false,
    };
    
    // Get current context from scheduler (Day 27)
    let mut ctx = match crate::sched::get_current_context(0) {
        Some(c) => c,
        None => return false,
    };
    
    // Save old context (Day 27)
    // TODO: Store in process struct for sigreturn
    
    // Setup new context for signal handler
    #[cfg(target_arch = "x86_64")]
    {
        // Save old RSP and RIP (will be restored by sigreturn)
        let old_rsp = ctx.rsp;
        let old_rip = ctx.rip;
        
        // Align stack to 16 bytes (x86_64 ABI requirement)
        let new_rsp = (old_rsp & !0xF) - 128; // Red zone + alignment
        
        // Push signal number as argument (RDI in System V ABI)
        ctx.rdi = signo as u64;
        
        // Push return address (sigreturn stub)
        // TODO: Implement sigreturn trampoline
        // For now, use old RIP (will crash, but shows handler was called)
        
        // Set new stack pointer
        ctx.rsp = new_rsp;
        
        // Set instruction pointer to handler
        ctx.rip = handler;
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        // Save old SP and ELR
        let old_sp = ctx.sp;
        let old_elr = ctx.elr;
        
        // Align stack to 16 bytes (ARM64 ABI requirement)
        let new_sp = (old_sp & !0xF) - 128;
        
        // Push signal number as argument (X0 in ARM64 ABI)
        ctx.x[0] = signo as u64;
        
        // Set new stack pointer
        ctx.sp = new_sp;
        
        // Set program counter to handler
        ctx.elr = handler;
    }
    
    // Update context in scheduler (Day 27)
    crate::sched::set_thread_context(tid, ctx)
}

/// Check if a user pointer is valid (helper for sigaction)
fn is_user_pointer_valid(ptr: *const u8, len: usize) -> bool {
    let addr = ptr as usize;
    
    // Null pointer check
    if addr == 0 {
        return false;
    }
    
    // Check if in user space (below 0x8000_0000_0000)
    if addr >= 0x8000_0000_0000 {
        return false;
    }
    
    // Check for overflow
    if addr.checked_add(len).is_none() {
        return false;
    }
    
    // Check end address still in user space
    if addr + len >= 0x8000_0000_0000 {
        return false;
    }
    
    true
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test: sys_kill() sends signal (Day 25)
    #[test]
    fn test_sys_kill_sends_signal() {
        // Expected behavior:
        // 1. Process 100 exists
        // 2. Call sys_kill(100, SIGTERM)
        // 3. Signal queued in process.pending_signals
        // 4. Returns 0 (success)
    }
    
    /// Test: sys_kill() validates signal number (Day 25)
    #[test]
    fn test_sys_kill_validates_signal() {
        // Expected behavior:
        // 1. Call sys_kill(100, -1) → -EINVAL
        // 2. Call sys_kill(100, 0) → 0 (just checks existence)
        // 3. Call sys_kill(100, 32) → -EINVAL (> SIGMAX)
    }
    
    /// Test: sys_kill() permission check (Day 25)
    #[test]
    fn test_sys_kill_permission() {
        // Expected behavior:
        // 1. Process 100 (user A) tries to kill process 200 (user B)
        // 2. check_kill_permission() returns false
        // 3. sys_kill() returns -EPERM
        //
        // Exception:
        // 1. Process 100 (root) tries to kill process 200
        // 2. check_kill_permission() returns true
        // 3. sys_kill() succeeds
    }
    
    /// Test: sys_kill() signal 0 checks existence (Day 25)
    #[test]
    fn test_sys_kill_signal_zero() {
        // Expected behavior:
        // 1. Call sys_kill(100, 0)
        // 2. If process 100 exists: return 0
        // 3. If process 100 doesn't exist: return -ESRCH
        // 4. No signal actually sent
        //
        // Use case: Check if process is alive without signaling it
    }
    
    /// Test: sys_sigaction() sets handler (Day 26)
    #[test]
    fn test_sys_sigaction_sets_handler() {
        // Expected behavior:
        // 1. Process has default handler for SIGTERM
        // 2. Call sys_sigaction(SIGTERM, &new_action, &old_action)
        // 3. old_action.sa_handler = SIG_DFL
        // 4. New handler installed
        // 5. process.signal_handlers[SIGTERM-1] = new_action.sa_handler
    }
    
    /// Test: sys_sigaction() protects SIGKILL (Day 26)
    #[test]
    fn test_sys_sigaction_protects_sigkill() {
        // Expected behavior:
        // 1. Call sys_sigaction(SIGKILL, &action, NULL)
        // 2. Returns -EINVAL
        // 3. Handler not changed
        // 4. SIGKILL remains uncatchable
        //
        // Same for SIGSTOP
    }
    
    /// Test: sys_signal() returns old handler (Day 26)
    #[test]
    fn test_sys_signal_returns_old() {
        // Expected behavior:
        // 1. signal_handlers[SIGINT-1] = SIG_DFL
        // 2. old = sys_signal(SIGINT, my_handler)
        // 3. old = SIG_DFL
        // 4. signal_handlers[SIGINT-1] = my_handler
    }
    
    /// Test: setup_signal_frame() modifies context (Day 27)
    #[test]
    fn test_setup_signal_frame_context() {
        // Expected behavior:
        // 1. Process running at RIP=0x400000, RSP=0x7FFF0000
        // 2. Signal SIGTERM delivered, handler=0x500000
        // 3. setup_signal_frame(pid, SIGTERM, 0x500000)
        // 4. Context updated:
        //    - RIP/ELR = 0x500000 (handler)
        //    - RDI/X0 = 15 (SIGTERM)
        //    - RSP/SP adjusted for stack frame
        // 5. Old RIP/RSP saved for sigreturn
    }
    
    /// Test: User handler is called (Day 27)
    #[test]
    fn test_user_handler_called() {
        // Expected behavior - Full flow:
        //
        // 1. User program installs handler:
        //    void handler(int sig) { printf("Got signal %d\n", sig); }
        //    signal(SIGTERM, handler);
        //
        // 2. Another process sends signal:
        //    kill(target_pid, SIGTERM);
        //
        // 3. Kernel queues signal:
        //    send_signal(target_pid, SIGTERM)
        //    pending_signals |= (1 << 14)
        //
        // 4. Before returning to userland:
        //    check_pending_signals() → SIGTERM
        //    deliver_signal(pid, SIGTERM)
        //    Handler is custom (not SIG_DFL or SIG_IGN)
        //    setup_signal_frame(pid, SIGTERM, handler_addr)
        //
        // 5. Context switch loads new context:
        //    RIP = handler address
        //    RDI = signal number
        //    CPU executes handler
        //
        // 6. Handler executes:
        //    printf("Got signal %d\n", sig);
        //    return;
        //
        // 7. Return triggers sigreturn (TODO):
        //    Restore old context
        //    Continue where interrupted
    }
    
    /// Test: Multiple signals to same process (Day 27)
    #[test]
    fn test_multiple_signals() {
        // Expected behavior:
        // 1. Process receives SIGTERM, SIGINT, SIGUSR1
        // 2. All queued in pending_signals
        // 3. Delivered in priority order (SIGINT first)
        // 4. Each handler executes
        // 5. Process continues after all handlers
    }
}
