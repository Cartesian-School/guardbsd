// Signal Types and Constants
// BSD 3-Clause License
// Day 22: Signal Infrastructure

#![no_std]

/// Signal number type
pub type Signal = i32;

/// Signal handler function pointer type
pub type SignalHandlerFn = fn();

/// User-mode signal frame placed on user stack when delivering a signal.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SignalFrame {
    pub saved_rip: u64,
    pub saved_rsp: u64,
    pub saved_rflags: u64,
    pub signo: u64,
}

// ==================== BSD Signal Numbers ====================
// Based on FreeBSD signal numbers

/// Hangup
pub const SIGHUP: Signal = 1;

/// Interrupt (Ctrl+C)
pub const SIGINT: Signal = 2;

/// Quit (Ctrl+\)
pub const SIGQUIT: Signal = 3;

/// Illegal instruction
pub const SIGILL: Signal = 4;

/// Trace/breakpoint trap
pub const SIGTRAP: Signal = 5;

/// Abort
pub const SIGABRT: Signal = 6;

/// Bus error
pub const SIGBUS: Signal = 7;

/// Floating point exception
pub const SIGFPE: Signal = 8;

/// Kill (cannot be caught or ignored)
pub const SIGKILL: Signal = 9;

/// User-defined signal 1
pub const SIGUSR1: Signal = 10;

/// Segmentation violation
pub const SIGSEGV: Signal = 11;

/// User-defined signal 2
pub const SIGUSR2: Signal = 12;

/// Broken pipe
pub const SIGPIPE: Signal = 13;

/// Alarm clock
pub const SIGALRM: Signal = 14;

/// Termination
pub const SIGTERM: Signal = 15;

/// Stack fault
pub const SIGSTKFLT: Signal = 16;

/// Child stopped or terminated
pub const SIGCHLD: Signal = 17;

/// Continue if stopped
pub const SIGCONT: Signal = 18;

/// Stop process (cannot be caught or ignored)
pub const SIGSTOP: Signal = 19;

/// Stop typed at terminal
pub const SIGTSTP: Signal = 20;

/// Background read from tty
pub const SIGTTIN: Signal = 21;

/// Background write to tty
pub const SIGTTOU: Signal = 22;

/// Urgent condition on socket
pub const SIGURG: Signal = 23;

/// CPU time limit exceeded
pub const SIGXCPU: Signal = 24;

/// File size limit exceeded
pub const SIGXFSZ: Signal = 25;

/// Virtual timer expired
pub const SIGVTALRM: Signal = 26;

/// Profiling timer expired
pub const SIGPROF: Signal = 27;

/// Window size change
pub const SIGWINCH: Signal = 28;

/// I/O now possible
pub const SIGIO: Signal = 29;

/// Power failure
pub const SIGPWR: Signal = 30;

/// Bad system call
pub const SIGSYS: Signal = 31;

/// Maximum signal number
pub const SIGMAX: Signal = 31;

// ==================== Special Handler Values ====================

/// Default signal handler
pub const SIG_DFL: u64 = 0;

/// Ignore signal
pub const SIG_IGN: u64 = 1;

/// Error return value
pub const SIG_ERR: u64 = !0; // -1 as u64

// ==================== Signal Action Structure ====================

/// Signal action structure (simplified sigaction)
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SignalAction {
    /// Handler function address:
    /// - SIG_DFL (0): Use default handler
    /// - SIG_IGN (1): Ignore signal
    /// - Other: User-defined handler address
    pub sa_handler: u64,
    
    /// Signals to block while handler runs
    pub sa_mask: u64,
    
    /// Signal action flags
    pub sa_flags: u32,
}

impl SignalAction {
    /// Create a default signal action
    pub const fn default() -> Self {
        Self {
            sa_handler: SIG_DFL,
            sa_mask: 0,
            sa_flags: 0,
        }
    }
    
    /// Create an ignore signal action
    pub const fn ignore() -> Self {
        Self {
            sa_handler: SIG_IGN,
            sa_mask: 0,
            sa_flags: 0,
        }
    }
    
    /// Create a custom signal action
    pub const fn handler(handler: u64) -> Self {
        Self {
            sa_handler: handler,
            sa_mask: 0,
            sa_flags: 0,
        }
    }
}

// ==================== Signal Action Flags ====================

/// SA_NOCLDSTOP: Don't notify parent when child stops
pub const SA_NOCLDSTOP: u32 = 0x0001;

/// SA_NOCLDWAIT: Don't create zombie on child death
pub const SA_NOCLDWAIT: u32 = 0x0002;

/// SA_SIGINFO: Use sa_sigaction instead of sa_handler
pub const SA_SIGINFO: u32 = 0x0004;

/// SA_RESTART: Restart interrupted system calls
pub const SA_RESTART: u32 = 0x0008;

/// SA_NODEFER: Don't mask signal while handler runs
pub const SA_NODEFER: u32 = 0x0010;

/// SA_RESETHAND: Reset handler to SIG_DFL after signal
pub const SA_RESETHAND: u32 = 0x0020;

// ==================== Signal Masks ====================

/// Create a signal mask from a signal number
pub const fn sigmask(signo: Signal) -> u64 {
    if signo > 0 && signo <= 64 {
        1u64 << (signo - 1)
    } else {
        0
    }
}

/// Check if a signal is in a mask
pub const fn sigismember(mask: u64, signo: Signal) -> bool {
    if signo > 0 && signo <= 64 {
        (mask & (1u64 << (signo - 1))) != 0
    } else {
        false
    }
}

/// Add a signal to a mask
pub const fn sigaddset(mask: u64, signo: Signal) -> u64 {
    if signo > 0 && signo <= 64 {
        mask | (1u64 << (signo - 1))
    } else {
        mask
    }
}

/// Remove a signal from a mask
pub const fn sigdelset(mask: u64, signo: Signal) -> u64 {
    if signo > 0 && signo <= 64 {
        mask & !(1u64 << (signo - 1))
    } else {
        mask
    }
}

// ==================== Default Signal Actions ====================

/// Default action for a signal
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DefaultAction {
    /// Terminate the process
    Term,
    
    /// Ignore the signal
    Ignore,
    
    /// Terminate with core dump
    Core,
    
    /// Stop the process
    Stop,
    
    /// Continue if stopped
    Cont,
}

/// Get the default action for a signal
pub const fn default_action(signo: Signal) -> DefaultAction {
    match signo {
        SIGHUP => DefaultAction::Term,
        SIGINT => DefaultAction::Term,
        SIGQUIT => DefaultAction::Core,
        SIGILL => DefaultAction::Core,
        SIGTRAP => DefaultAction::Core,
        SIGABRT => DefaultAction::Core,
        SIGBUS => DefaultAction::Core,
        SIGFPE => DefaultAction::Core,
        SIGKILL => DefaultAction::Term,  // Cannot be caught
        SIGUSR1 => DefaultAction::Term,
        SIGSEGV => DefaultAction::Core,
        SIGUSR2 => DefaultAction::Term,
        SIGPIPE => DefaultAction::Term,
        SIGALRM => DefaultAction::Term,
        SIGTERM => DefaultAction::Term,
        SIGSTKFLT => DefaultAction::Term,
        SIGCHLD => DefaultAction::Ignore,  // Special: ignored by default
        SIGCONT => DefaultAction::Cont,    // Continue if stopped
        SIGSTOP => DefaultAction::Stop,    // Cannot be caught
        SIGTSTP => DefaultAction::Stop,
        SIGTTIN => DefaultAction::Stop,
        SIGTTOU => DefaultAction::Stop,
        SIGURG => DefaultAction::Ignore,
        SIGXCPU => DefaultAction::Core,
        SIGXFSZ => DefaultAction::Core,
        SIGVTALRM => DefaultAction::Term,
        SIGPROF => DefaultAction::Term,
        SIGWINCH => DefaultAction::Ignore,
        SIGIO => DefaultAction::Ignore,
        SIGPWR => DefaultAction::Term,
        SIGSYS => DefaultAction::Core,
        _ => DefaultAction::Term,  // Unknown signals terminate
    }
}

/// Check if a signal cannot be caught or ignored
pub const fn is_uncatchable(signo: Signal) -> bool {
    signo == SIGKILL || signo == SIGSTOP
}

/// Check if a signal causes core dump by default
pub const fn causes_core_dump(signo: Signal) -> bool {
    matches!(default_action(signo), DefaultAction::Core)
}

/// Check if a signal stops the process by default
pub const fn causes_stop(signo: Signal) -> bool {
    matches!(default_action(signo), DefaultAction::Stop)
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_signal_constants() {
        assert_eq!(SIGHUP, 1);
        assert_eq!(SIGINT, 2);
        assert_eq!(SIGKILL, 9);
        assert_eq!(SIGTERM, 15);
        assert_eq!(SIGCHLD, 17);
        assert_eq!(SIGSTOP, 19);
    }
    
    #[test]
    fn test_sigmask() {
        assert_eq!(sigmask(1), 0x0000_0000_0000_0001);
        assert_eq!(sigmask(2), 0x0000_0000_0000_0002);
        assert_eq!(sigmask(9), 0x0000_0000_0000_0100);
    }
    
    #[test]
    fn test_sigismember() {
        let mask = 0x0000_0000_0000_0101; // Signals 1 and 9
        assert!(sigismember(mask, 1));
        assert!(!sigismember(mask, 2));
        assert!(sigismember(mask, 9));
    }
    
    #[test]
    fn test_default_actions() {
        assert_eq!(default_action(SIGTERM), DefaultAction::Term);
        assert_eq!(default_action(SIGKILL), DefaultAction::Term);
        assert_eq!(default_action(SIGCHLD), DefaultAction::Ignore);
        assert_eq!(default_action(SIGSTOP), DefaultAction::Stop);
        assert_eq!(default_action(SIGCONT), DefaultAction::Cont);
        assert_eq!(default_action(SIGSEGV), DefaultAction::Core);
    }
    
    #[test]
    fn test_uncatchable() {
        assert!(is_uncatchable(SIGKILL));
        assert!(is_uncatchable(SIGSTOP));
        assert!(!is_uncatchable(SIGTERM));
        assert!(!is_uncatchable(SIGCHLD));
    }
}
