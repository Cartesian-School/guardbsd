// userland/shell/src/builtins.rs
// Built-in shell commands
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

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
            _ => None,
        }
    }
}
