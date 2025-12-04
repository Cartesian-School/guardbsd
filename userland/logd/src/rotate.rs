// userland/logd/src/rotate.rs
// Log rotation system for logd
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::LogdConfig;
use gbsd::{Fd, stat, rename, unlink, sync};

#[derive(Clone, Copy)]
pub struct FileStat {
    pub size: usize,
    pub exists: bool,
}

pub struct LogRotator {
    config: LogdConfig,
}

impl LogRotator {
    pub fn new(config: LogdConfig) -> Self {
        Self { config }
    }

    /// Check if rotation is needed and perform it
    pub fn check_and_rotate(&self, log_path: &[u8]) -> Result<(), ()> {
        // Get current file size
        let stat = self.stat_file(log_path)?;

        if !stat.exists || stat.size < self.config.max_size {
            return Ok(()); // No rotation needed
        }

        // Perform rotation
        self.rotate_files(log_path)
    }

    /// Get file statistics
    fn stat_file(&self, path: &[u8]) -> Result<FileStat, ()> {
        match fs_stat(path) {
            Ok(stat) => Ok(FileStat {
                size: stat.size as usize,
                exists: true,
            }),
            Err(_) => Ok(FileStat {
                size: 0,
                exists: false,
            }),
        }
    }

    /// Perform the actual rotation
    fn rotate_files(&self, log_path: &[u8]) -> Result<(), ()> {
        // Rotate from highest to lowest to avoid overwriting
        for i in (1..=self.config.rotate_keep).rev() {
            let src_path = self.build_rotated_path(log_path, i - 1);
            let dst_path = self.build_rotated_path(log_path, i);

            // Try to rename, ignore if source doesn't exist
            let _ = fs_rename(&src_path, &dst_path);
        }

        // Remove the oldest file if it exists
        if self.config.rotate_keep > 0 {
            let oldest_path = self.build_rotated_path(log_path, self.config.rotate_keep);
            let _ = fs_unlink(&oldest_path);
        }

        Ok(())
    }

    /// Build a rotated filename (e.g., kernel.log.1, kernel.log.2, etc.)
    fn build_rotated_path(&self, base_path: &[u8], rotation_num: u8) -> [u8; 256] {
        let mut result = [0u8; 256];
        let mut pos = 0;

        // Copy base path
        for &b in base_path {
            if b == 0 {
                break;
            }
            if pos < result.len() - 4 { // Leave room for ".N\0"
                result[pos] = b;
                pos += 1;
            }
        }

        // Add dot
        result[pos] = b'.';
        pos += 1;

        // Add rotation number
        let num_str = self.u8_to_str(rotation_num);
        for &b in &num_str {
            if b == 0 {
                break;
            }
            if pos < result.len() - 1 {
                result[pos] = b;
                pos += 1;
            }
        }

        // Null terminate
        result[pos] = 0;
        result
    }

    /// Convert u8 to string
    fn u8_to_str(&self, num: u8) -> [u8; 4] {
        let mut buf = [0u8; 4];
        if num == 0 {
            buf[0] = b'0';
            return buf;
        }

        let mut n = num;
        let mut i = 0;
        while n > 0 && i < 3 {
            buf[i] = b'0' + (n % 10);
            n /= 10;
            i += 1;
        }

        // Reverse the digits
        let mut result = [0u8; 4];
        for j in 0..i {
            result[j] = buf[i - 1 - j];
        }
        result
    }
}

/// File system operations (wrappers around VFS)
pub fn fs_stat(path: &[u8]) -> Result<gbsd::Stat, gbsd::Error> {
    stat(path)
}

pub fn fs_rename(old_path: &[u8], new_path: &[u8]) -> Result<(), gbsd::Error> {
    rename(old_path, new_path)
}

pub fn fs_unlink(path: &[u8]) -> Result<(), gbsd::Error> {
    unlink(path)
}

pub fn fs_sync(fd: Fd) -> Result<(), gbsd::Error> {
    sync(fd)
}
