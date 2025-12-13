//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: kernel
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Klej między backendem logowania jądra a sinkiem plikowym/VFS.

use crate::fs::kfile::{self, KFile, KfError, KfOpenFlags};
use kernel_log::log_backend;

static mut KLOG_FILE: Option<KFile> = None;

/// Initialize kernel log file sink (stubbed until VFS is ready).
pub fn init_klog_file_sink() {
    let path = "/var/log/kern.log";
    let flags = KfOpenFlags::WRITE | KfOpenFlags::APPEND | KfOpenFlags::CREATE;

    match kfile::kf_open(path, flags, 0o644) {
        Ok(file) => unsafe {
            KLOG_FILE = Some(file);
            log_backend::set_external_sink(Some(klog_sink_write));
        },
        Err(KfError::NotImplemented) => {
            // VFS not wired yet; skip sink registration.
        }
        Err(_e) => {
            // Other errors: also skip sink registration for now.
        }
    }
}

fn klog_sink_write(data: &[u8]) {
    unsafe {
        if let Some(file) = KLOG_FILE {
            let _ = kfile::kf_write_all(file, data);
        }
    }
}
