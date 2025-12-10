// kernel/boot_stub/src/log_sink.rs
// Glue between kernel log backend and future file/VFS sink (stubbed)

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
