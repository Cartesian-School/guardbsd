// Process Job Control Syscalls
// BSD 3-Clause License
// Implements: setpgid, getpgid, waitpid, kill (signal sending)

#![no_std]

use crate::process::types::{Process, Pid, ProcessState, SIGINT, SIGTSTP, SIGCHLD, SIGCONT};

// Process table from kernel/process/process.rs
extern "C" {
    static mut PROCESS_TABLE: [Option<Process>; 64];
    static mut CURRENT_PROCESS: Option<Pid>;
}

/// Get reference to process table
unsafe fn get_process_table() -> &'static mut [Option<Process>; 64] {
    &mut PROCESS_TABLE
}

/// Get current process ID
unsafe fn get_current_pid() -> Option<Pid> {
    CURRENT_PROCESS
}

/// Find process by PID
fn find_process_mut(pid: Pid) -> Option<&'static mut Process> {
    unsafe {
        let table = get_process_table();
        for slot in table.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    return Some(proc);
                }
            }
        }
    }
    None
}

// Error codes
const ESRCH: isize = -3;   // No such process
const EINVAL: isize = -22; // Invalid argument
const EPERM: isize = -1;   // Operation not permitted

/// Set process group ID
/// setpgid(pid, pgid)
/// - If pid == 0, use calling process
/// - If pgid == 0, set pgid = pid
pub fn sys_setpgid(pid: Pid, pgid: Pid) -> isize {
    unsafe {
        let current_pid = match get_current_pid() {
            Some(p) => p,
            None => return EINVAL,
        };
        
        // Resolve target pid
        let target_pid = if pid == 0 { current_pid } else { pid };
        
        // Resolve target pgid
        let target_pgid = if pgid == 0 { target_pid } else { pgid };
        
        // Find target process
        let proc = match find_process_mut(target_pid) {
            Some(p) => p,
            None => return ESRCH,
        };
        
        // Set process group
        proc.pgid = target_pgid;
        
        0
    }
}

/// Get process group ID
pub fn sys_getpgid(pid: Pid) -> isize {
    unsafe {
        let target_pid = if pid == 0 {
            match get_current_pid() {
                Some(p) => p,
                None => return EINVAL,
            }
        } else {
            pid
        };
        
        let proc = match find_process_mut(target_pid) {
            Some(p) => p,
            None => return ESRCH,
        };
        
        proc.pgid as isize
    }
}

/// Send signal to process or process group
/// kill(pid, sig)
/// - If pid > 0: send to process pid
/// - If pid == 0: send to all processes in caller's process group
/// - If pid == -1: send to all processes (except init)
/// - If pid < -1: send to all processes in process group -pid
pub fn sys_kill(pid: isize, sig: i32) -> isize {
    unsafe {
        let current_pid = match get_current_pid() {
            Some(p) => p,
            None => return EINVAL,
        };
        
        if pid > 0 {
            // Send to specific process
            send_signal_to_pid(pid as Pid, sig)
        } else if pid == 0 {
            // Send to all processes in caller's group
            let current_proc = match find_process_mut(current_pid) {
                Some(p) => p,
                None => return ESRCH,
            };
            let pgid = current_proc.pgid;
            send_signal_to_pgid(pgid, sig)
        } else if pid == -1 {
            // Send to all processes (except init)
            send_signal_to_all(sig)
        } else {
            // Send to process group -pid
            let pgid = (-pid) as Pid;
            send_signal_to_pgid(pgid, sig)
        }
    }
}

/// Send signal to a specific process
fn send_signal_to_pid(pid: Pid, sig: i32) -> isize {
    if sig < 0 || sig >= 32 {
        return EINVAL;
    }
    
    let proc = match find_process_mut(pid) {
        Some(p) => p,
        None => return ESRCH,
    };
    
    // Set pending signal bit
    proc.pending_signals |= 1u64 << sig;
    
    0
}

/// Send signal to all processes in a process group
fn send_signal_to_pgid(pgid: Pid, sig: i32) -> isize {
    if sig < 0 || sig >= 32 {
        return EINVAL;
    }
    
    unsafe {
        let table = get_process_table();
        let mut count = 0;
        
        for slot in table.iter_mut() {
            if let Some(proc) = slot {
                if proc.pgid == pgid {
                    proc.pending_signals |= 1u64 << sig;
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            0
        } else {
            ESRCH
        }
    }
}

/// Send signal to all processes (except init)
fn send_signal_to_all(sig: i32) -> isize {
    if sig < 0 || sig >= 32 {
        return EINVAL;
    }
    
    unsafe {
        let table = get_process_table();
        
        for slot in table.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid != 1 {  // Don't send to init
                    proc.pending_signals |= 1u64 << sig;
                }
            }
        }
    }
    
    0
}

/// Wait for any child process to change state
/// waitpid(pid, status, options)
/// - If pid > 0: wait for specific child
/// - If pid == 0: wait for any child in same process group
/// - If pid == -1: wait for any child
/// - If pid < -1: wait for any child in process group -pid
/// 
/// options:
/// - WNOHANG (1): return immediately if no child has exited
/// Returns: pid of child, or 0 if WNOHANG and no child, or negative error
pub fn sys_waitpid(pid: isize, status_ptr: *mut i32, options: u32) -> isize {
    const WNOHANG: u32 = 1;
    
    unsafe {
        let current_pid = match get_current_pid() {
            Some(p) => p,
            None => return EINVAL,
        };
        
        let current_proc = match find_process_mut(current_pid) {
            Some(p) => p,
            None => return EINVAL,
        };
        
        // Look for matching zombie children
        let table = get_process_table();
        
        for slot in table.iter_mut() {
            if let Some(proc) = slot {
                // Check if this is a matching child
                let matches = if pid > 0 {
                    proc.pid == pid as Pid && proc.parent == Some(current_pid)
                } else if pid == 0 {
                    proc.parent == Some(current_pid) && proc.pgid == current_proc.pgid
                } else if pid == -1 {
                    proc.parent == Some(current_pid)
                } else {
                    proc.parent == Some(current_pid) && proc.pgid == (-pid) as Pid
                };
                
                if !matches {
                    continue;
                }
                
                // Check if child is zombie or stopped
                if proc.state == ProcessState::Zombie {
                    let child_pid = proc.pid;
                    let exit_status = proc.exit_status.unwrap_or(0);
                    
                    // Write status if pointer provided
                    if !status_ptr.is_null() {
                        *status_ptr = exit_status;
                    }
                    
                    // Reap the process
                    *slot = None;
                    
                    // Remove from parent's children list
                    current_proc.remove_child(child_pid);
                    
                    return child_pid as isize;
                } else if proc.stopped {
                    // Child is stopped - report it
                    let child_pid = proc.pid;
                    
                    if !status_ptr.is_null() {
                        // Status format: stopped bit | signal number
                        *status_ptr = 0x7F | (SIGTSTP << 8);
                    }
                    
                    return child_pid as isize;
                }
            }
        }
        
        // No matching child found
        if (options & WNOHANG) != 0 {
            // Non-blocking: return 0
            return 0;
        }
        
        // Would block - for now, return 0 (no child ready)
        // In a real implementation, this would sleep until SIGCHLD
        0
    }
}

/// Check and deliver pending signals for current process
/// Called before returning to userspace
pub fn check_pending_signals() {
    unsafe {
        let current_pid = match get_current_pid() {
            Some(p) => p,
            None => return,
        };
        
        let proc = match find_process_mut(current_pid) {
            Some(p) => p,
            None => return,
        };
        
        let pending = proc.pending_signals;
        
        // Check signals in priority order
        
        // SIGINT: terminate
        if (pending & (1 << SIGINT)) != 0 {
            proc.pending_signals &= !(1 << SIGINT);
            proc.state = ProcessState::Zombie;
            proc.exit_status = Some(130);  // 128 + SIGINT
            
            // Send SIGCHLD to parent
            if let Some(parent_pid) = proc.parent {
                if let Some(parent) = find_process_mut(parent_pid) {
                    parent.pending_signals |= 1 << SIGCHLD;
                }
            }
            return;
        }
        
        // SIGTSTP: stop
        if (pending & (1 << SIGTSTP)) != 0 {
            proc.pending_signals &= !(1 << SIGTSTP);
            proc.state = ProcessState::Stopped;
            proc.stopped = true;
            
            // Send SIGCHLD to parent
            if let Some(parent_pid) = proc.parent {
                if let Some(parent) = find_process_mut(parent_pid) {
                    parent.pending_signals |= 1 << SIGCHLD;
                }
            }
            return;
        }
        
        // SIGCONT: resume
        if (pending & (1 << SIGCONT)) != 0 {
            proc.pending_signals &= !(1 << SIGCONT);
            if proc.stopped {
                proc.stopped = false;
                proc.state = ProcessState::Ready;
            }
            return;
        }
        
        // SIGCHLD: just clear (default is ignore)
        if (pending & (1 << SIGCHLD)) != 0 {
            proc.pending_signals &= !(1 << SIGCHLD);
        }
    }
}

