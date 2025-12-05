// Signal Module
// BSD 3-Clause License
// Days 22-24: Signal Infrastructure

#![no_std]

pub mod types;
pub mod delivery;
pub mod handlers;

// Re-export commonly used types and constants
pub use types::{
    Signal, SignalAction, DefaultAction,
    
    // Signal numbers
    SIGHUP, SIGINT, SIGQUIT, SIGILL, SIGTRAP, SIGABRT,
    SIGBUS, SIGFPE, SIGKILL, SIGUSR1, SIGSEGV, SIGUSR2,
    SIGPIPE, SIGALRM, SIGTERM, SIGSTKFLT, SIGCHLD, SIGCONT,
    SIGSTOP, SIGTSTP, SIGTTIN, SIGTTOU, SIGURG, SIGXCPU,
    SIGXFSZ, SIGVTALRM, SIGPROF, SIGWINCH, SIGIO, SIGPWR,
    SIGSYS, SIGMAX,
    
    // Special handlers
    SIG_DFL, SIG_IGN, SIG_ERR,
    
    // Signal action flags
    SA_NOCLDSTOP, SA_NOCLDWAIT, SA_SIGINFO, SA_RESTART,
    SA_NODEFER, SA_RESETHAND,
    
    // Helper functions
    sigmask, sigismember, sigaddset, sigdelset,
    default_action, is_uncatchable, causes_core_dump, causes_stop,
};

// Re-export delivery functions
pub use delivery::{
    send_signal, check_pending_signals, deliver_signal,
    queue_signal, process_pending_signals,
    is_signal_blocked, block_signal, unblock_signal,
};

// Re-export handler functions
pub use handlers::{
    handle_default_signal,
    handle_sigterm, handle_sigkill, handle_sigstop,
    handle_sigcont, handle_sigchld,
};

