//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel_process
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Kanoniczne definicje typów procesu.

#![no_std]

/// Process ID type
pub type Pid = usize;

/// Maximum number of children per process
pub const MAX_CHILDREN: usize = 32;

/// Maximum number of file descriptors per process
pub const MAX_FD_PER_PROCESS: usize = 64;

/// Maximum number of signals
pub const MAX_SIGNALS: usize = 32;

/// Process Control Block - Canonical structure for all process management
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Process {
    // Identity
    pub pid: Pid,
    pub parent: Option<Pid>,
    pub pgid: Pid,  // Process group ID (for job control)
    pub children: [Option<Pid>; MAX_CHILDREN],
    pub child_count: usize,
    
    // State
    pub state: ProcessState,
    pub stopped: bool,  // True if stopped by SIGTSTP/SIGSTOP
    pub exit_status: Option<i32>,
    
    // Memory layout
    pub page_table: usize,
    pub entry: u64,
    pub stack_top: u64,
    pub stack_bottom: u64,
    pub kernel_stack: u64,
    pub heap_base: u64,
    pub heap_limit: u64,
    
    // Resource tracking
    pub fd_table: [Option<FileDescriptor>; MAX_FD_PER_PROCESS],
    pub fd_count: usize,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub exit_code: i32,
    
    // Scheduling (link to TCB in scheduler)
    pub thread_id: Option<usize>,
    
    // Signals
    pub pending_signals: u64,
    pub killed: bool,
    pub signal_mask: u64,
    pub signal_handlers: [Option<u64>; MAX_SIGNALS],
}

impl Process {
    /// Create a new empty process structure
    pub const fn empty() -> Self {
        Self {
            pid: 0,
            parent: None,
            pgid: 0,
            children: [None; MAX_CHILDREN],
            child_count: 0,
            state: ProcessState::New,
            stopped: false,
            exit_status: None,
            page_table: 0,
            entry: 0,
            stack_top: 0,
            stack_bottom: 0,
            kernel_stack: 0,
            heap_base: 0,
            heap_limit: 0,
            fd_table: [None; MAX_FD_PER_PROCESS],
            fd_count: 0,
            memory_usage: 0,
            memory_limit: 0,
            thread_id: None,
            pending_signals: 0,
            killed: false,
            signal_mask: 0,
            signal_handlers: [None; MAX_SIGNALS],
            exit_code: 0,
        }
    }
    
    /// Add a child process to this process's children list
    pub fn add_child(&mut self, child_pid: Pid) -> bool {
        if self.child_count >= MAX_CHILDREN {
            return false;
        }
        
        for slot in self.children.iter_mut() {
            if slot.is_none() {
                *slot = Some(child_pid);
                self.child_count += 1;
                return true;
            }
        }
        false
    }
    
    /// Remove a child process from this process's children list
    pub fn remove_child(&mut self, child_pid: Pid) -> bool {
        for slot in self.children.iter_mut() {
            if *slot == Some(child_pid) {
                *slot = None;
                self.child_count = self.child_count.saturating_sub(1);
                return true;
            }
        }
        false
    }
    
    /// Allocate a file descriptor
    pub fn alloc_fd(&mut self, fd: FileDescriptor) -> Option<usize> {
        for (i, slot) in self.fd_table.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(fd);
                self.fd_count += 1;
                return Some(i);
            }
        }
        None
    }
    
    /// Free a file descriptor
    pub fn free_fd(&mut self, fd_num: usize) -> bool {
        if fd_num < MAX_FD_PER_PROCESS && self.fd_table[fd_num].is_some() {
            self.fd_table[fd_num] = None;
            self.fd_count = self.fd_count.saturating_sub(1);
            true
        } else {
            false
        }
    }
    
    /// Close all file descriptors
    pub fn close_all_fds(&mut self) {
        for fd in self.fd_table.iter_mut() {
            *fd = None;
        }
        self.fd_count = 0;
    }
}

/// Process state
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ProcessState {
    /// Just created, not yet runnable
    New = 0,
    /// Ready to be scheduled
    Ready = 1,
    /// Currently executing
    Running = 2,
    /// Waiting for I/O or event
    Blocked = 3,
    /// Sleeping (timer-based wait)
    Sleeping = 4,
    /// Stopped by signal (SIGTSTP/SIGSTOP)
    Stopped = 6,
    /// Exited, waiting to be reaped by parent
    Zombie = 5,
}

/// File descriptor
#[derive(Copy, Clone, Debug)]
pub struct FileDescriptor {
    pub inode: u64,
    pub offset: u64,
    pub flags: u32,
}

/// Signal handler
#[derive(Copy, Clone, Debug)]
pub struct SignalHandler {
    /// Handler address: 0 = SIG_DFL, 1 = SIG_IGN, other = user handler
    pub handler: u64,
}

impl SignalHandler {
    pub const fn default() -> Self {
        Self { handler: 0 } // SIG_DFL
    }
}

/// Special signal handler values
pub const SIG_DFL: u64 = 0;  // Default action
pub const SIG_IGN: u64 = 1;  // Ignore signal

/// Signal numbers (BSD-compatible)
pub const SIGHUP: i32 = 1;
pub const SIGINT: i32 = 2;
pub const SIGQUIT: i32 = 3;
pub const SIGILL: i32 = 4;
pub const SIGTRAP: i32 = 5;
pub const SIGABRT: i32 = 6;
pub const SIGBUS: i32 = 7;
pub const SIGFPE: i32 = 8;
pub const SIGKILL: i32 = 9;
pub const SIGUSR1: i32 = 10;
pub const SIGSEGV: i32 = 11;
pub const SIGUSR2: i32 = 12;
pub const SIGPIPE: i32 = 13;
pub const SIGALRM: i32 = 14;
pub const SIGTERM: i32 = 15;
pub const SIGCHLD: i32 = 17;
pub const SIGCONT: i32 = 18;
pub const SIGSTOP: i32 = 19;
pub const SIGTSTP: i32 = 20;
pub const SIGTTIN: i32 = 21;
pub const SIGTTOU: i32 = 22;
