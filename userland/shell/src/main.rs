// userland/shell/src/main.rs
// GuardBSD Shell (gsh) - zsh-inspired
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use gbsd::*;

mod builtins;
mod parser;
mod io;
mod exec;

use builtins::Builtin;
use parser::Command;
use io::*;

const PROMPT: &[u8] = b"gsh> ";
const MAX_CMD_LEN: usize = 256;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    shell_main();
}

fn shell_main() -> ! {
    let mut cmd_buf = [0u8; MAX_CMD_LEN];
    
    loop {
        // Display prompt
        let _ = print(PROMPT);
        
        // Read command
        let len = match read_line(io::STDIN, &mut cmd_buf) {
            Ok(n) => n,
            Err(_) => continue,
        };
        
        if len == 0 {
            // No input, yield CPU
            #[cfg(target_arch = "x86_64")]
            unsafe {
                core::arch::asm!("pause", options(nomem, nostack));
            }
            
            #[cfg(target_arch = "aarch64")]
            unsafe {
                core::arch::asm!("yield", options(nomem, nostack));
            }
            continue;
        }
        
        // Parse command
        let cmd = match Command::parse(&cmd_buf[..len]) {
            Some(c) => c,
            None => continue,
        };
        
        // Execute command
        if let Err(_) = exec::execute(&cmd) {
            let _ = println(b"Command failed");
        }
    }
}




