// userland/shell/src/parser.rs
// Command parser (zsh-inspired) with quotes and escaping
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

pub struct Command<'a> {
    pub name: &'a [u8],
    pub args: [Option<&'a [u8]>; 8],
    pub arg_count: usize,
    pub background: bool, // true if command ends with &
}

impl<'a> Command<'a> {
    pub fn new() -> Self {
        Self {
            name: &[],
            args: [None; 8],
            arg_count: 0,
            background: false,
        }
    }

    pub fn parse(input: &'a [u8]) -> Option<Self> {
        let mut cmd = Command::new();
        let mut in_word = false;
        let mut word_start = 0;
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escape_next = false;

        for (i, &byte) in input.iter().enumerate() {
            if escape_next {
                // Skip escaped character
                escape_next = false;
                continue;
            }

            match byte {
                b'\\' if !in_single_quote => {
                    escape_next = true;
                }
                b'\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                }
                b'"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                }
                b' ' | b'\t' | b'\n' | 0 if !in_single_quote && !in_double_quote => {
                    if in_word {
                        let word = &input[word_start..i];

                        // Check for & (background)
                        if word == b"&" {
                            cmd.background = true;
                        } else if cmd.name.is_empty() {
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

        // Handle last word
        if in_word && word_start < input.len() {
            let word = &input[word_start..];
            if word == b"&" {
                cmd.background = true;
            } else if cmd.name.is_empty() {
                cmd.name = word;
            } else if cmd.arg_count < 8 {
                cmd.args[cmd.arg_count] = Some(word);
                cmd.arg_count += 1;
            }
        }

        if !cmd.name.is_empty() {
            Some(cmd)
        } else {
            None
        }
    }

    /// Unescape and dequote a string
    pub fn process_string(input: &[u8]) -> ([u8; 256], usize) {
        let mut output = [0u8; 256];
        let mut out_pos = 0;
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escape_next = false;

        for &byte in input {
            if byte == 0 {
                break;
            }

            if escape_next {
                if out_pos < 255 {
                    // Escaped character
                    output[out_pos] = byte;
                    out_pos += 1;
                }
                escape_next = false;
                continue;
            }

            match byte {
                b'\\' if !in_single_quote => {
                    escape_next = true;
                }
                b'\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                }
                b'"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                }
                _ => {
                    if out_pos < 255 {
                        output[out_pos] = byte;
                        out_pos += 1;
                    }
                }
            }
        }

        (output, out_pos)
    }
}
