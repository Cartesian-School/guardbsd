//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Klej między backendem logowania a przyszłym sinkiem plikowym/VFS (stub).

use crate::fs::kfile::{self, KFile, KfError, KfOpenFlags};
use kernel_log::log_backend;

static mut KLOG_FILE: Option<KFile> = None;

/// Initialize kernel log file sink (no-op until VFS is implemented).
pub fn init_klog_file_sink() {
    let _ = kfile::kf_mkdir("/var");
    let _ = kfile::kf_mkdir("/var/log");
    let path = "/var/log/kern.log";
    let flags = KfOpenFlags::WRITE | KfOpenFlags::APPEND | KfOpenFlags::CREATE;

    match kfile::kf_open(path, flags, 0o644) {
        Ok(file) => unsafe {
            KLOG_FILE = Some(file);
            log_backend::set_external_sink(Some(klog_sink_write));
            log_backend::write_bytes(b"[KLOG-TEST] persistent sink marker\n");
        },
        Err(KfError::NotImplemented) => {
            // VFS not wired yet; leave external sink unset.
        }
        Err(_e) => {
            // Other errors: leave sink unset.
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
