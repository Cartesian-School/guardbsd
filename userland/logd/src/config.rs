// userland/logd/src/config.rs
// Configuration file parser for logd
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use kernel_log::LogLevel;

pub const DEFAULT_LOG_FILE: &[u8] = b"/var/log/kernel.log";
pub const DEFAULT_MAX_SIZE: usize = 512_000; // 512KB
pub const DEFAULT_ROTATE_KEEP: u8 = 3;
pub const DEFAULT_FLUSH_INTERVAL_MS: u32 = 100;
pub const DEFAULT_LEVEL: LogLevel = LogLevel::Info;

#[derive(Clone, Copy)]
pub struct LogdConfig {
    pub log_file: [u8; 128],
    pub log_file_len: usize,
    pub max_size: usize,
    pub rotate_keep: u8,
    pub flush_interval_ms: u32,
    pub level: LogLevel,
}

impl LogdConfig {
    pub const fn default() -> Self {
        let mut log_file = [0u8; 128];
        let mut i = 0;
        while i < DEFAULT_LOG_FILE.len() && i < log_file.len() {
            log_file[i] = DEFAULT_LOG_FILE[i];
            i += 1;
        }

        Self {
            log_file,
            log_file_len: DEFAULT_LOG_FILE.len(),
            max_size: DEFAULT_MAX_SIZE,
            rotate_keep: DEFAULT_ROTATE_KEEP,
            flush_interval_ms: DEFAULT_FLUSH_INTERVAL_MS,
            level: DEFAULT_LEVEL,
        }
    }

    pub fn load_from_file(&mut self, path: &[u8]) -> Result<(), ()> {
        match crate::open(path, crate::O_RDONLY) {
            Ok(fd) => {
                let mut buf = [0u8; 2048];
                let len = match crate::read(fd, &mut buf) {
                    Ok(len) => len,
                    Err(_) => {
                        let _ = crate::close(fd);
                        return Err(());
                    }
                };
                let _ = crate::close(fd);
                if len == 0 {
                    return Err(());
                }
                self.parse_config(&buf[..len])
            }
            Err(crate::error::Error::NoSys) => Err(()), // FS not available
            Err(_) => Err(()),
        }
    }

    fn parse_config(&mut self, data: &[u8]) -> Result<(), ()> {
        let mut line_start = 0;

        for i in 0..data.len() {
            if data[i] == b'\n' {
                self.parse_line(&data[line_start..i]);
                line_start = i + 1;
            }
        }

        // Handle last line if no trailing newline
        if line_start < data.len() {
            self.parse_line(&data[line_start..]);
        }

        Ok(())
    }

    fn parse_line(&mut self, line: &[u8]) {
        // Skip empty lines and comments
        if line.is_empty() || line[0] == b'#' || line[0] == b'\r' {
            return;
        }

        // Find '=' separator
        if let Some(eq_pos) = line.iter().position(|&b| b == b'=') {
            let key = trim_spaces(&line[..eq_pos]);
            let value = trim_spaces(&line[eq_pos + 1..]);

            self.parse_key_value(key, value);
        }
    }

    fn parse_key_value(&mut self, key: &[u8], value: &[u8]) {
        match key {
            b"log_file" => {
                let len = core::cmp::min(value.len(), self.log_file.len());
                self.log_file[..len].copy_from_slice(&value[..len]);
                self.log_file_len = len;
            }
            b"max_size" => {
                if let Some(size) = parse_usize(value) {
                    self.max_size = size;
                }
            }
            b"rotate_keep" => {
                if let Some(keep) = parse_u8(value) {
                    self.rotate_keep = keep;
                }
            }
            b"flush_interval_ms" => {
                if let Some(interval) = parse_u32(value) {
                    self.flush_interval_ms = interval;
                }
            }
            b"level" => {
                self.level = match value {
                    b"trace" => LogLevel::Trace,
                    b"debug" => LogLevel::Debug,
                    b"info" => LogLevel::Info,
                    b"warn" => LogLevel::Warn,
                    b"error" => LogLevel::Error,
                    _ => DEFAULT_LEVEL,
                };
            }
            _ => {} // Unknown key, ignore
        }
    }

    pub fn should_log(&self, level: LogLevel) -> bool {
        match self.level {
            LogLevel::Trace => true,
            LogLevel::Debug => matches!(
                level,
                LogLevel::Debug | LogLevel::Info | LogLevel::Warn | LogLevel::Error
            ),
            LogLevel::Info => matches!(level, LogLevel::Info | LogLevel::Warn | LogLevel::Error),
            LogLevel::Warn => matches!(level, LogLevel::Warn | LogLevel::Error),
            LogLevel::Error => matches!(level, LogLevel::Error),
        }
    }
}

fn trim_spaces(s: &[u8]) -> &[u8] {
    let start = s.iter().position(|&b| b != b' ' && b != b'\t').unwrap_or(0);
    let end = s
        .iter()
        .rposition(|&b| b != b' ' && b != b'\t')
        .map(|i| i + 1)
        .unwrap_or(s.len());
    &s[start..end]
}

fn parse_usize(s: &[u8]) -> Option<usize> {
    let mut result = 0usize;
    for &b in s {
        if b >= b'0' && b <= b'9' {
            result = result.checked_mul(10)?;
            result = result.checked_add((b - b'0') as usize)?;
        } else {
            return None;
        }
    }
    Some(result)
}

fn parse_u8(s: &[u8]) -> Option<u8> {
    let mut result = 0u8;
    for &b in s {
        if b >= b'0' && b <= b'9' {
            result = result.checked_mul(10)?;
            result = result.checked_add(b - b'0')?;
        } else {
            return None;
        }
    }
    Some(result)
}

fn parse_u32(s: &[u8]) -> Option<u32> {
    let mut result = 0u32;
    for &b in s {
        if b >= b'0' && b <= b'9' {
            result = result.checked_mul(10)?;
            result = result.checked_add((b - b'0') as u32)?;
        } else {
            return None;
        }
    }
    Some(result)
}
