//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: shell
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Zarządzanie zmiennymi środowiskowymi w powłoce gsh.

use gbsd::*;

const MAX_VARS: usize = 64;
const MAX_VAR_NAME: usize = 64;
const MAX_VAR_VALUE: usize = 256;

pub struct Environment {
    pub vars: [Option<EnvVar>; MAX_VARS],
    pub count: usize,
}

#[derive(Clone, Copy)]
pub struct EnvVar {
    pub name: [u8; MAX_VAR_NAME],
    pub name_len: usize,
    pub value: [u8; MAX_VAR_VALUE],
    pub value_len: usize,
}

impl Environment {
    pub const fn new() -> Self {
        Environment {
            vars: [None; MAX_VARS],
            count: 0,
        }
    }

    pub fn init_defaults(&mut self) {
        // Set default PATH
        self.set("PATH", "/bin:/usr/bin");

        // Set default PWD
        self.set("PWD", "/");

        // Set shell name
        self.set("SHELL", "gsh");

        // Set user (placeholder)
        self.set("USER", "root");
    }

    pub fn set(&mut self, name: &str, value: &str) {
        let name_bytes = name.as_bytes();
        let value_bytes = value.as_bytes();

        if name_bytes.len() >= MAX_VAR_NAME || value_bytes.len() >= MAX_VAR_VALUE {
            return;
        }

        // Check if variable already exists
        for i in 0..self.count {
            if let Some(ref mut var) = self.vars[i] {
                if var.name_len == name_bytes.len() && &var.name[..var.name_len] == name_bytes {
                    // Update existing
                    var.value[..value_bytes.len()].copy_from_slice(value_bytes);
                    var.value_len = value_bytes.len();
                    return;
                }
            }
        }

        // Add new variable
        if self.count < MAX_VARS {
            let mut var = EnvVar {
                name: [0; MAX_VAR_NAME],
                name_len: name_bytes.len(),
                value: [0; MAX_VAR_VALUE],
                value_len: value_bytes.len(),
            };

            var.name[..name_bytes.len()].copy_from_slice(name_bytes);
            var.value[..value_bytes.len()].copy_from_slice(value_bytes);

            self.vars[self.count] = Some(var);
            self.count += 1;
        }
    }

    pub fn get(&self, name: &str) -> Option<&[u8]> {
        let name_bytes = name.as_bytes();

        for i in 0..self.count {
            if let Some(ref var) = self.vars[i] {
                if var.name_len == name_bytes.len() && &var.name[..var.name_len] == name_bytes {
                    return Some(&var.value[..var.value_len]);
                }
            }
        }
        None
    }

    pub fn expand_variables(&self, input: &[u8], output: &mut [u8]) -> usize {
        let mut out_pos = 0;
        let mut in_pos = 0;

        while in_pos < input.len() && out_pos < output.len() - 1 {
            if input[in_pos] == b'$' && in_pos + 1 < input.len() {
                // Variable expansion
                in_pos += 1;

                if input[in_pos] == b'{' {
                    // ${VAR} syntax
                    in_pos += 1;
                    let start = in_pos;

                    while in_pos < input.len() && input[in_pos] != b'}' {
                        in_pos += 1;
                    }

                    if in_pos < input.len() {
                        let var_name = &input[start..in_pos];
                        in_pos += 1; // Skip closing }

                        if let Ok(name_str) = core::str::from_utf8(var_name) {
                            if let Some(value) = self.get(name_str) {
                                let copy_len = core::cmp::min(value.len(), output.len() - out_pos);
                                output[out_pos..out_pos + copy_len]
                                    .copy_from_slice(&value[..copy_len]);
                                out_pos += copy_len;
                                continue;
                            }
                        }
                    }
                } else {
                    // $VAR syntax
                    let start = in_pos;

                    while in_pos < input.len()
                        && (input[in_pos].is_ascii_alphanumeric() || input[in_pos] == b'_')
                    {
                        in_pos += 1;
                    }

                    let var_name = &input[start..in_pos];
                    if let Ok(name_str) = core::str::from_utf8(var_name) {
                        if let Some(value) = self.get(name_str) {
                            let copy_len = core::cmp::min(value.len(), output.len() - out_pos);
                            output[out_pos..out_pos + copy_len].copy_from_slice(&value[..copy_len]);
                            out_pos += copy_len;
                            continue;
                        }
                    }
                }
            } else {
                output[out_pos] = input[in_pos];
                out_pos += 1;
                in_pos += 1;
            }
        }

        output[out_pos] = 0; // Null terminate
        out_pos
    }
}

impl EnvVar {
    pub const fn new() -> Self {
        EnvVar {
            name: [0; MAX_VAR_NAME],
            name_len: 0,
            value: [0; MAX_VAR_VALUE],
            value_len: 0,
        }
    }
}
