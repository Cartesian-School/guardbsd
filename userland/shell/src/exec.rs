// userland/shell/src/exec.rs
// Command execution
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use gbsd::*;
use crate::parser::Command;
use crate::builtins::Builtin;
use crate::io::*;

pub fn execute(cmd: &Command) -> Result<()> {
    // Try built-in commands first
    if let Some(builtin) = Builtin::from_name(cmd.name) {
        return execute_builtin(&builtin, cmd);
    }
    
    // External command (future: spawn process)
    println(b"External commands not yet supported")?;
    Err(Error::Invalid)
}

fn execute_builtin(builtin: &Builtin, cmd: &Command) -> Result<()> {
    match builtin {
        Builtin::Exit => {
            exit(0);
        }
        Builtin::Help => {
            println(b"GuardBSD Shell (gsh) v1.0.0")?;
            println(b"Built-in commands:")?;
            println(b"  exit  - Exit shell")?;
            println(b"  help  - Show this help")?;
            println(b"  echo  - Print arguments")?;
            println(b"  cd    - Change directory")?;
            println(b"  pwd   - Print working directory")?;
            Ok(())
        }
        Builtin::Echo => {
            for i in 0..cmd.arg_count {
                if let Some(arg) = cmd.args[i] {
                    print(arg)?;
                    if i < cmd.arg_count - 1 {
                        print(b" ")?;
                    }
                }
            }
            println(b"")?;
            Ok(())
        }
        Builtin::Cd => {
            if cmd.arg_count == 0 {
                println(b"cd: missing directory")?;
                Err(Error::Invalid)
            } else {
                // Future: actual chdir via VFS
                Ok(())
            }
        }
        Builtin::Pwd => {
            // Future: actual getcwd via VFS
            println(b"/")?;
            Ok(())
        }
    }
}
