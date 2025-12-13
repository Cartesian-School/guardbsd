//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: shell
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Wbudowane polecenia powłoki GuardBSD.

use gbsd::*;

pub enum Builtin {
    Exit,
    Help,
    Echo,
    Cd,
    Pwd,
    Export,
    Set,
    Unset,
    Env,
    History,
    Fg,
    Bg,
    Jobs,
    Klog,
    KlogFile,
    KlogCheck,
}

impl Builtin {
    pub fn from_name(name: &[u8]) -> Option<Self> {
        match name {
            b"exit" => Some(Builtin::Exit),
            b"help" => Some(Builtin::Help),
            b"echo" => Some(Builtin::Echo),
            b"cd" => Some(Builtin::Cd),
            b"pwd" => Some(Builtin::Pwd),
            b"export" => Some(Builtin::Export),
            b"set" => Some(Builtin::Set),
            b"unset" => Some(Builtin::Unset),
            b"env" => Some(Builtin::Env),
            b"history" => Some(Builtin::History),
            b"fg" => Some(Builtin::Fg),
            b"bg" => Some(Builtin::Bg),
            b"jobs" => Some(Builtin::Jobs),
            b"klog" | b"dmesg" => Some(Builtin::Klog),
            b"klogfile" | b"klogfs" => Some(Builtin::KlogFile),
            b"klogcheck" => Some(Builtin::KlogCheck),
            _ => None,
        }
    }
}
