// userland/shell/src/exec.rs
// Command execution
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::builtins::Builtin;
use crate::io::*;
use crate::jobs::JobControl;
use crate::parser::Command;
use crate::spawn::ProcessSpawner;
use gbsd::*;

pub fn execute(
    cmd: &Command,
    env: &mut crate::env::Environment,
    history: &[Option<[u8; 256]>; 100],
    history_count: usize,
    jobs: &mut JobControl,
) -> Result<()> {
    // Try built-in commands first
    if let Some(builtin) = Builtin::from_name(cmd.name) {
        return execute_builtin(&builtin, cmd, env, history, history_count, jobs);
    }

    // External command - spawn process
    let spawner = ProcessSpawner::new(env);
    match spawner.spawn(cmd, env) {
        Ok(pid) => {
            // Wait for child to complete
            let _ = spawner.wait(pid);
            Ok(())
        }
        Err(_) => {
            println(b"Command not found")?;
            Err(Error::NotFound)
        }
    }
}

fn execute_builtin(
    builtin: &Builtin,
    cmd: &Command,
    env: &mut crate::env::Environment,
    history: &[Option<[u8; 256]>; 100],
    history_count: usize,
    jobs: &mut JobControl,
) -> Result<()> {
    match builtin {
        Builtin::Exit => {
            exit(0);
        }
        Builtin::Help => {
            println(b"GuardBSD Shell (gsh) v1.0.0")?;
            println(b"Built-in commands:")?;
            println(b"  exit   - Exit shell")?;
            println(b"  help   - Show this help")?;
            println(b"  echo   - Print arguments")?;
            println(b"  cd     - Change directory")?;
            println(b"  pwd    - Print working directory")?;
            println(b"  export - Set environment variables")?;
            println(b"  set    - Set shell variables")?;
            println(b"  unset  - Unset variables")?;
            println(b"  env    - Show environment variables")?;
            println(b"  history- Show command history")?;
            println(b"  fg     - Bring job to foreground")?;
            println(b"  bg     - Send job to background")?;
            println(b"  jobs   - List background jobs")?;
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
                // Get the directory argument
                let dir_arg = cmd.args[0].unwrap_or(b"");
                let dir_str = core::str::from_utf8(dir_arg).unwrap_or("/");

                // Use chdir syscall
                match gbsd::fs::chdir(dir_arg) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        println(&format!("cd: {}", dir_str).as_bytes())?;
                        println(b": directory not found")?;
                        Err(e)
                    }
                }
            }
        }
        Builtin::Pwd => {
            // Use getcwd syscall
            let mut cwd_buf = [0u8; 256];
            match gbsd::fs::getcwd(&mut cwd_buf) {
                Ok(len) => {
                    // Print the current directory
                    let cwd_slice = &cwd_buf[..len.min(255)];
                    println(cwd_slice)?;
                    Ok(())
                }
                Err(e) => {
                    println(b"pwd: failed to get current directory")?;
                    Err(e)
                }
            }
        }
        Builtin::Export => {
            if cmd.arg_count == 0 {
                // Show all exported variables
                for i in 0..env.count {
                    if let Some(ref var) = env.vars[i] {
                        print(&var.name[..var.name_len])?;
                        print(b"=")?;
                        println(&var.value[..var.value_len])?;
                    }
                }
                Ok(())
            } else {
                // Export variable: export NAME=value
                for i in 0..cmd.arg_count {
                    if let Some(arg) = cmd.args[i] {
                        if let Some(pos) = find_char(arg, b'=') {
                            let name = &arg[..pos];
                            let value = &arg[pos + 1..];

                            if let (Ok(name_str), Ok(value_str)) =
                                (core::str::from_utf8(name), core::str::from_utf8(value))
                            {
                                env.set(name_str, value_str);
                            }
                        }
                    }
                }
                Ok(())
            }
        }
        Builtin::Set => {
            if cmd.arg_count == 0 {
                // Show all variables
                for i in 0..env.count {
                    if let Some(ref var) = env.vars[i] {
                        print(&var.name[..var.name_len])?;
                        print(b"=")?;
                        println(&var.value[..var.value_len])?;
                    }
                }
                Ok(())
            } else if cmd.arg_count == 1 {
                // Set variable without value: set NAME
                if let Some(arg) = cmd.args[0] {
                    if let Ok(name_str) = core::str::from_utf8(arg) {
                        env.set(name_str, "");
                    }
                }
                Ok(())
            } else {
                // Set variable with value: set NAME value
                if let (Some(name_arg), Some(value_arg)) = (cmd.args[0], cmd.args[1]) {
                    if let (Ok(name_str), Ok(value_str)) = (
                        core::str::from_utf8(name_arg),
                        core::str::from_utf8(value_arg),
                    ) {
                        env.set(name_str, value_str);
                    }
                }
                Ok(())
            }
        }
        Builtin::Unset => {
            if cmd.arg_count > 0 {
                if let Some(arg) = cmd.args[0] {
                    if let Ok(name_str) = core::str::from_utf8(arg) {
                        // For now, just set to empty (we don't have delete)
                        env.set(name_str, "");
                    }
                }
            }
            Ok(())
        }
        Builtin::Env => {
            // Show all environment variables
            for i in 0..env.count {
                if let Some(ref var) = env.vars[i] {
                    print(&var.name[..var.name_len])?;
                    print(b"=")?;
                    println(&var.value[..var.value_len])?;
                }
            }
            Ok(())
        }
        Builtin::History => {
            // Show command history
            for i in 0..history_count {
                let idx = if history_count <= history.len() {
                    i
                } else {
                    (history.len() + i - (history_count - history.len())) % history.len()
                };

                if let Some(ref entry) = history[idx] {
                    // Find actual length
                    let mut len = 0;
                    while len < entry.len() && entry[len] != 0 {
                        len += 1;
                    }

                    if len > 0 {
                        // Print history number
                        print_number(i + 1)?;
                        print(b"  ")?;
                        println(&entry[..len])?;
                    }
                }
            }
            Ok(())
        }
        Builtin::Fg => {
            // Bring job to foreground
            if cmd.arg_count > 0 {
                if let Some(arg) = cmd.args[0] {
                    // Parse job ID
                    let job_id = parse_number(arg).unwrap_or(1);
                    jobs.foreground(job_id)
                } else {
                    println(b"fg: missing job ID")?;
                    Err(Error::Invalid)
                }
            } else {
                println(b"fg: missing job ID")?;
                Err(Error::Invalid)
            }
        }
        Builtin::Bg => {
            // Send job to background
            if cmd.arg_count > 0 {
                if let Some(arg) = cmd.args[0] {
                    let job_id = parse_number(arg).unwrap_or(1);
                    jobs.background(job_id)
                } else {
                    println(b"bg: missing job ID")?;
                    Err(Error::Invalid)
                }
            } else {
                println(b"bg: missing job ID")?;
                Err(Error::Invalid)
            }
        }
        Builtin::Jobs => {
            // List jobs
            jobs.list_jobs()
        }
    }
}

fn find_char(slice: &[u8], ch: u8) -> Option<usize> {
    for (i, &c) in slice.iter().enumerate() {
        if c == ch {
            return Some(i);
        }
    }
    None
}

fn print_number(mut num: usize) -> Result<()> {
    if num == 0 {
        return print(b"0");
    }

    let mut buf = [0u8; 10];
    let mut pos = 0;

    while num > 0 {
        buf[pos] = b'0' + (num % 10) as u8;
        num /= 10;
        pos += 1;
    }

    // Print in reverse
    while pos > 0 {
        pos -= 1;
        print(&[buf[pos]])?;
    }

    Ok(())
}

fn parse_number(s: &[u8]) -> Option<usize> {
    let mut num = 0usize;
    for &ch in s {
        if ch >= b'0' && ch <= b'9' {
            num = num * 10 + (ch - b'0') as usize;
        } else if ch == 0 {
            break;
        } else {
            return None;
        }
    }
    Some(num)
}
