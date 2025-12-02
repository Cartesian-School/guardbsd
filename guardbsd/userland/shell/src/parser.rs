// userland/shell/src/parser.rs
// Command parser (zsh-inspired)
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

pub struct Command<'a> {
    pub name: &'a [u8],
    pub args: [Option<&'a [u8]>; 8],
    pub arg_count: usize,
}

impl<'a> Command<'a> {
    pub fn new() -> Self {
        Self {
            name: &[],
            args: [None; 8],
            arg_count: 0,
        }
    }

    pub fn parse(input: &'a [u8]) -> Option<Self> {
        let mut cmd = Command::new();
        let _start = 0;
        let mut in_word = false;
        let mut word_start = 0;

        for (i, &byte) in input.iter().enumerate() {
            match byte {
                b' ' | b'\t' | b'\n' | 0 => {
                    if in_word {
                        let word = &input[word_start..i];
                        if cmd.name.is_empty() {
                            cmd.name = word;
                        } else if cmd.arg_count < 8 {
                            cmd.args[cmd.arg_count] = Some(word);
                            cmd.arg_count += 1;
                        }
                        in_word = false;
                    }
                    if byte == 0 || byte == b'\n' {
                        break;
                    }
                }
                _ => {
                    if !in_word {
                        word_start = i;
                        in_word = true;
                    }
                }
            }
        }

        if !cmd.name.is_empty() {
            Some(cmd)
        } else {
            None
        }
    }
}
