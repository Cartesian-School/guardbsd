// userland/shell/src/completion.rs
// Tab Completion (zsh-inspired)
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::builtins::Builtin;

const BUILTINS: &[&[u8]] = &[
    b"exit", b"help", b"echo", b"cd", b"pwd", b"export", b"set", b"unset", b"env", b"history",
    b"fg", b"bg", b"jobs",
];

pub struct Completer {
    matches: [[u8; 64]; 32],
    match_count: usize,
    current_match: usize,
}

impl Completer {
    pub const fn new() -> Self {
        Self {
            matches: [[0; 64]; 32],
            match_count: 0,
            current_match: 0,
        }
    }

    /// Find completions for the given prefix
    pub fn complete(&mut self, line: &[u8], cursor_pos: usize) -> Option<&[u8]> {
        // Find the word to complete (from last space to cursor)
        let word_start = self.find_word_start(line, cursor_pos);
        let prefix = &line[word_start..cursor_pos];

        // Clear previous matches
        self.match_count = 0;
        self.current_match = 0;

        // If at the beginning or after whitespace, complete command names
        let is_command = word_start == 0
            || (word_start > 0 && (line[word_start - 1] == b' ' || line[word_start - 1] == b'\t'));

        if is_command {
            self.find_command_matches(prefix);
        } else {
            self.find_path_matches(prefix);
        }

        if self.match_count > 0 {
            Some(&self.matches[0][..self.get_match_len(0)])
        } else {
            None
        }
    }

    /// Cycle to next completion match
    pub fn next_match(&mut self) -> Option<&[u8]> {
        if self.match_count == 0 {
            return None;
        }

        self.current_match = (self.current_match + 1) % self.match_count;
        Some(&self.matches[self.current_match][..self.get_match_len(self.current_match)])
    }

    fn find_word_start(&self, line: &[u8], cursor_pos: usize) -> usize {
        let mut start = cursor_pos;
        while start > 0 && line[start - 1] != b' ' && line[start - 1] != b'\t' {
            start -= 1;
        }
        start
    }

    fn find_command_matches(&mut self, prefix: &[u8]) {
        // Match builtin commands
        for &builtin in BUILTINS {
            if self.starts_with(builtin, prefix) {
                self.add_match(builtin);
            }
        }

        // TODO: Match executables in PATH directories
    }

    fn find_path_matches(&mut self, prefix: &[u8]) {
        // TODO: Match files/directories from current directory
        // For now, just return nothing
        let _ = prefix;
    }

    fn starts_with(&self, s: &[u8], prefix: &[u8]) -> bool {
        if prefix.len() > s.len() {
            return false;
        }
        &s[..prefix.len()] == prefix
    }

    fn add_match(&mut self, s: &[u8]) {
        if self.match_count >= 32 {
            return;
        }

        let len = s.len().min(64);
        self.matches[self.match_count][..len].copy_from_slice(&s[..len]);
        self.match_count += 1;
    }

    fn get_match_len(&self, idx: usize) -> usize {
        let mut len = 0;
        while len < 64 && self.matches[idx][len] != 0 {
            len += 1;
        }
        len
    }

    /// Get common prefix of all matches (for multiple matches)
    pub fn get_common_prefix(&self) -> usize {
        if self.match_count <= 1 {
            return 0;
        }

        let mut common_len = 0;
        'outer: for i in 0..64 {
            let ch = self.matches[0][i];
            if ch == 0 {
                break;
            }

            for j in 1..self.match_count {
                if self.matches[j][i] != ch {
                    break 'outer;
                }
            }

            common_len = i + 1;
        }

        common_len
    }

    /// List all matches (for multiple completions)
    pub fn list_matches(&self) -> &[[u8; 64]] {
        &self.matches[..self.match_count]
    }

    pub fn match_count(&self) -> usize {
        self.match_count
    }
}
