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
}

impl Builtin {
    pub fn from_name(name: &[u8]) -> Option<Self> {
        match name {
            b"exit" => Some(Builtin::Exit),
            b"help" => Some(Builtin::Help),
            b"echo" => Some(Builtin::Echo),
            b"cd" => Some(Builtin::Cd),
            b"pwd" => Some(Builtin::Pwd),
            _ => None,
        }
    }


}
