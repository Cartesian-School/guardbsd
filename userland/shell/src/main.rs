// userland/shell/src/main.rs
// GuardBSD Shell (gsh) - zsh-inspired
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

mod builtins;
mod exec;
mod io;
mod parser;
mod env;

use gbsd::*;
use crate::env::*;

const MAX_LINE: usize = 256;
const MAX_HISTORY: usize = 100;

struct Shell {
    history: [Option<[u8; MAX_LINE]>; MAX_HISTORY],
    history_count: usize,
    history_index: usize, // Current position in history (-1 means not navigating)
    current_line: [u8; MAX_LINE],
    line_pos: usize,      // Current cursor position in line
    line_len: usize,      // Total length of current line
    prompt: &'static [u8],
    env: Environment,
}

impl Shell {
    fn new() -> Self {
        let mut env = Environment::new();
        env.init_defaults();

        Shell {
            history: [None; MAX_HISTORY],
            history_count: 0,
            history_index: usize::MAX, // Not navigating history
            current_line: [0; MAX_LINE],
            line_pos: 0,
            line_len: 0,
            prompt: b"gsh> ",
            env,
        }
    }

    fn run(&mut self) -> ! {
        loop {
            self.display_prompt();
            if self.read_line() {
                self.process_line();
            }
        }
    }

    fn display_prompt(&self) {
        let _ = print(self.prompt);
    }

    fn read_line(&mut self) -> bool {
        self.line_pos = 0;
        self.line_len = 0;
        self.current_line = [0; MAX_LINE];
        self.history_index = usize::MAX; // Reset history navigation

        loop {
            let mut byte = [0u8; 1];
            match gbsd::read(0, &mut byte) {
                Ok(0) => return false, // EOF
                Ok(_) => {
                    let c = byte[0];

                    match c {
                        b'\n' | b'\r' => {
                            let _ = println(b"");
                            self.line_len = self.line_pos;
                            return true;
                        }
                        127 | 8 => { // Backspace
                            if self.line_pos > 0 {
                                self.line_pos -= 1;
                                // Shift characters left
                                for i in self.line_pos..self.line_len {
                                    if i + 1 < MAX_LINE {
                                        self.current_line[i] = self.current_line[i + 1];
                                    }
                                }
                                self.current_line[self.line_len - 1] = 0;
                                self.line_len -= 1;

                                // Redisplay line
                                self.redisplay_line();
                            }
                        }
                        27 => { // ESC - start of escape sequence
                            if self.handle_escape_sequence() {
                                return true;
                            }
                        }
                        1 => { // Ctrl+A - beginning of line
                            self.line_pos = 0;
                            self.move_cursor();
                        }
                        5 => { // Ctrl+E - end of line
                            self.line_pos = self.line_len;
                            self.move_cursor();
                        }
                        3 => { // Ctrl+C
                            let _ = println(b"^C");
                            self.line_pos = 0;
                            self.line_len = 0;
                            self.current_line = [0; MAX_LINE];
                            self.history_index = usize::MAX;
                            break;
                        }
                        4 => { // Ctrl+D (EOF)
                            let _ = println(b"");
                            return false;
                        }
                        _ => {
                            if self.line_pos < MAX_LINE - 1 && self.line_len < MAX_LINE - 1 && c >= 32 && c <= 126 {
                                // Insert character at cursor position
                                for i in (self.line_pos..self.line_len + 1).rev() {
                                    if i + 1 < MAX_LINE {
                                        self.current_line[i + 1] = self.current_line[i];
                                    }
                                }
                                self.current_line[self.line_pos] = c;
                                self.line_pos += 1;
                                self.line_len += 1;

                                // Redisplay line from cursor
                                self.redisplay_line();
                            }
                        }
                    }
                }
                Err(_) => {
                    return false;
                }
            }
        }
        false
    }

    fn redisplay_line(&self) {
        // Clear from cursor to end of line
        let _ = gbsd::write(1, b"\x1b[K");

        // Redisplay the rest of the line
        if self.line_pos < self.line_len {
            let _ = gbsd::write(1, &self.current_line[self.line_pos..self.line_len]);
        }

        // Move cursor back to correct position
        if self.line_len > self.line_pos {
            let move_left = self.line_len - self.line_pos;
            for _ in 0..move_left {
                let _ = gbsd::write(1, b"\x1b[D");
            }
        }
    }

    fn move_cursor(&self) {
        // For now, just redisplay the entire line
        // In a full implementation, we'd use ANSI cursor positioning
        self.redisplay_line();
    }

    fn handle_escape_sequence(&mut self) -> bool {
        // Read the next two characters for arrow keys: ESC [ A/B/C/D
        let mut seq = [0u8; 3];
        for i in 0..3 {
            let mut byte = [0u8; 1];
            if gbsd::read(0, &mut byte).is_ok() {
                seq[i] = byte[0];
            } else {
                return false;
            }
        }

        // Check for arrow keys: ESC [ A (up), ESC [ B (down)
        if seq[0] == b'[' {
            match seq[1] {
                b'A' => { // Up arrow - previous history
                    self.navigate_history(true);
                    true
                }
                b'B' => { // Down arrow - next history
                    self.navigate_history(false);
                    true
                }
                _ => false
            }
        } else {
            false
        }
    }

    fn navigate_history(&mut self, up: bool) {
        if self.history_count == 0 {
            return;
        }

        if self.history_index == usize::MAX {
            // First time navigating - save current line
            if up {
                self.history_index = self.history_count - 1;
            } else {
                return; // Can't go down from current
            }
        } else if up {
            if self.history_index > 0 {
                self.history_index -= 1;
            } else {
                return; // Already at oldest
            }
        } else {
            if self.history_index < self.history_count - 1 {
                self.history_index += 1;
            } else {
                // Back to current (empty) line
                self.history_index = usize::MAX;
                self.clear_line();
                self.line_pos = 0;
                return;
            }
        }

        // Load history entry
        if self.history_index != usize::MAX {
            if let Some(ref entry) = self.history[self.history_index] {
                // Find actual length
                let mut len = 0;
                while len < MAX_LINE && entry[len] != 0 {
                    len += 1;
                }

                self.clear_line();
                self.current_line[..len].copy_from_slice(&entry[..len]);
                self.line_pos = len;

                // Redisplay line
                let _ = gbsd::write(1, &self.current_line[..len]);
            }
        }
    }

    fn clear_line(&self) {
        // Clear from cursor to end of line
        let _ = gbsd::write(1, b"\x1b[K");
    }


    fn add_to_history(&mut self) {
        if self.line_len == 0 {
            return;
        }

        let mut entry = [0u8; MAX_LINE];
        entry[..self.line_len].copy_from_slice(&self.current_line[..self.line_len]);

        self.history[self.history_pos] = Some(entry);
        self.history_pos = (self.history_pos + 1) % MAX_HISTORY;

        if self.history_count < MAX_HISTORY {
            self.history_count += 1;
        }
    }

    fn process_line(&mut self) {
        if self.line_len == 0 {
            return; // Empty line
        }

        // Add to history
        self.add_to_history();

        // Expand variables
        let mut expanded_line = [0u8; MAX_LINE];
        let expanded_len = self.env.expand_variables(&self.current_line[..self.line_len], &mut expanded_line);

        // Parse and execute
        if let Some(cmd) = parser::Command::parse(&expanded_line[..expanded_len]) {
            let _ = exec::execute(&cmd, &mut self.env, &self.history, self.history_count);
        }
    }
}

fn shell_main() -> ! {
    let pid = getpid().unwrap_or(0);
    print_pid(b"[GSH] pid=", pid);
    let _ = println(b" interactive shell started");
    let _ = println(b"[GSH] interactive shell ready");

    let mut shell = Shell::new();
    shell.run();
}

fn cpu_relax() {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("pause", options(nomem, nostack));
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("yield", options(nomem, nostack));
    }
}

fn print_pid(prefix: &[u8], pid: u64) {
    let mut buf = [0u8; 64];
    let mut pos = 0;
    for &b in prefix {
        if pos < buf.len() {
            buf[pos] = b;
            pos += 1;
        }
    }
    let pos_after = write_num(&mut buf, pos, pid);
    let _ = println(&buf[..core::cmp::min(pos_after, buf.len())]);
}

fn write_num(out: &mut [u8], mut pos: usize, mut val: u64) -> usize {
    let mut tmp = [0u8; 20];
    let mut i = 0;
    if val == 0 {
        tmp[0] = b'0';
        i = 1;
    } else {
        while val > 0 && i < tmp.len() {
            tmp[i] = b'0' + (val % 10) as u8;
            val /= 10;
            i += 1;
        }
    }
    while i > 0 {
        i -= 1;
        if pos < out.len() {
            out[pos] = tmp[i];
            pos += 1;
        }
    }
    pos
}

fn println(buf: &[u8]) {
    let _ = gbsd::write(1, buf);
    let _ = gbsd::write(1, b"\n");
}
