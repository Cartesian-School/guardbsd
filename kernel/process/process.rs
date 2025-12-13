//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_process
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Implementacja zarządzania procesami (tablica procesów, PID, stany).

#![no_std]

use crate::mm::AddressSpace;
use crate::process::types::{Process, Pid, ProcessState};
use crate::sched::ArchContext;

static mut NEXT_PID: Pid = 1;
static mut CURRENT_PROCESS: Option<Pid> = None;
static mut PROCESS_TABLE: [Option<Process>; 64] = [None; 64];

pub fn allocate_pid() -> Pid {
    unsafe {
        let pid = NEXT_PID;
        NEXT_PID += 1;
        pid
    }
}

pub fn create_process(entry: u64, stack_top: u64, page_table: usize) -> Option<Pid> {
    let pid = allocate_pid();
    
    // Allocate kernel stack
    let kernel_stack = crate::mm::alloc_page()? as u64 + 4096;
    
    let mut proc = Process::empty();
    proc.pid = pid;
    proc.entry = entry;
    proc.stack_top = stack_top;
    proc.stack_bottom = stack_top.saturating_sub(0x10000); // 64KB user stack
    proc.state = ProcessState::Ready;
    proc.kernel_stack = kernel_stack;
    proc.page_table = page_table;
    proc.parent = unsafe { CURRENT_PROCESS };
    proc.heap_base = 0x400000; // Default heap start at 4MB
    proc.heap_limit = 0x400000;
    proc.memory_limit = 16 * 1024 * 1024; // 16MB default limit
    
    // Add to parent's children list if we have a parent
    if let Some(parent_pid) = proc.parent {
        unsafe {
            for slot in PROCESS_TABLE.iter_mut() {
                if let Some(parent) = slot {
                    if parent.pid == parent_pid {
                        parent.add_child(pid);
                        break;
                    }
                }
            }
        }
    }
    
    unsafe {
        for slot in PROCESS_TABLE.iter_mut() {
            if slot.is_none() {
                *slot = Some(proc);
                return Some(pid);
            }
        }
    }
    None
}

pub fn exec(pid: Pid, _path: &str) -> bool {
    unsafe {
        for slot in PROCESS_TABLE.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    // TODO: Implement actual ELF loading
                    // For now, just set state to ready
                    // The actual implementation will require:
                    // 1. Load ELF file from VFS
                    // 2. Create new address space
                    // 3. Map segments
                    // 4. Update process entry point
                    // 5. Update thread context in scheduler
                    
                    proc.state = ProcessState::Ready;
                    return true;
                }
            }
        }
    }
    false
}

pub fn schedule() -> Option<Pid> {
    unsafe {
        for slot in PROCESS_TABLE.iter() {
            if let Some(proc) = slot {
                if proc.state == ProcessState::Ready {
                    return Some(proc.pid);
                }
            }
        }
    }
    None
}

pub fn switch_to(pid: Pid) {
    unsafe {
        CURRENT_PROCESS = Some(pid);
        for slot in PROCESS_TABLE.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    proc.state = ProcessState::Running;
                    // Activate page table
                    core::arch::asm!(
                        "mov cr3, {}",
                        in(reg) proc.page_table,
                        options(nostack, preserves_flags)
                    );
                    return;
                }
            }
        }
    }
}

pub fn get_current() -> Option<Pid> {
    unsafe { CURRENT_PROCESS }
}

/// Try to reserve additional memory_usage for a process. Returns true on success.
pub fn try_add_memory_usage(pid: Pid, bytes: u64) -> bool {
    unsafe {
        for slot in PROCESS_TABLE.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    let new_usage = proc.memory_usage.saturating_add(bytes);
                    if new_usage > proc.memory_limit {
                        return false;
                    }
                    proc.memory_usage = new_usage;
                    return true;
                }
            }
        }
    }
    false
}

/// Mark a process as killed (used by limit violations).
pub fn mark_killed(pid: Pid) {
    unsafe {
        for slot in PROCESS_TABLE.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    proc.killed = true;
                    return;
                }
            }
        }
    }
}

/// Get mutable reference to process by PID.
pub fn get_process_mut(pid: Pid) -> Option<&'static mut Process> {
    unsafe {
        for slot in PROCESS_TABLE.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    return Some(proc);
                }
            }
        }
    }
    None
}

/// Add to a process's accounted memory usage (saturating add).
pub fn add_memory_usage(pid: Pid, bytes: u64) {
    unsafe {
        for slot in PROCESS_TABLE.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    proc.memory_usage = proc.memory_usage.saturating_add(bytes);
                    return;
                }
            }
        }
    }
}
