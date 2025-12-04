// userland/logd/src/main.rs
// Kernel log daemon: pulls ring buffer records and emits to serial/FS.

#![no_std]
#![no_main]

use gbsd::*;
use kernel_log::{UserLogRecord, LOG_MSG_MAX, LOG_SUBSYS_MAX};

mod config;
mod rotate;

use config::LogdConfig;
use rotate::LogRotator;

const STDOUT: Fd = 1;
const LOG_BATCH: usize = 16;
const LINE_CAP: usize = LOG_MSG_MAX + LOG_SUBSYS_MAX + 64;
const CONFIG_PATH: &[u8] = b"/etc/logd.conf\0";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    logd_main()
}

fn logd_main() -> ! {
    // FIRST: Load configuration
    let mut config = LogdConfig::default();
    let config_loaded = config.load_from_file(CONFIG_PATH).is_ok();

    if config_loaded {
        // Log config loading success (but don't use klog_* to avoid recursion)
        let mut msg = [0u8; 64];
        let len = format_config_msg(&mut msg, "configuration loaded");
        let _ = write(STDOUT, &msg[..len]);
    } else {
        // Log config fallback (but don't use klog_* to avoid recursion)
        let mut msg = [0u8; 64];
        let len = format_config_msg(&mut msg, "FS unavailable, using defaults");
        let _ = write(STDOUT, &msg[..len]);
    }

    // SECOND: Create log rotator
    let mut rotator = LogRotator::new(config);

    // THIRD: Register ourselves as the logging daemon
    let pid = getpid().unwrap_or(0);
    let log_syscalls = register_kernel_log_daemon(pid).is_ok();

    // FOURTH: Log file output (placeholder - filesystem not implemented)
    let mut log_fd: Option<Fd> = None;
    // TODO: Open log file when filesystem syscalls are implemented

    // Log successful startup (but don't use klog_* to avoid recursion)
    let mut msg = [0u8; 64];
    let len = if log_syscalls {
        format_config_msg(&mut msg, "online (stdout + pending FS/log syscalls)")
    } else {
        format_config_msg(&mut msg, "stdout-only (log syscalls ENOSYS)")
    };
    let _ = write(STDOUT, &msg[..len]);

    if !log_syscalls {
        loop {
            cpu_relax();
        }
    }

    let mut records = [UserLogRecord::empty(); LOG_BATCH];
    let mut line = [0u8; LINE_CAP];
    let mut last_flush = 0u64;

    loop {
        let count = match read_kernel_logs(&mut records) {
            Ok(c) => c,
            Err(_) => 0,
        };

        if count > 0 {
            for rec in records.iter().take(count) {
                // Check if we should log this level
                if !config.should_log(rec.level) {
                    continue;
                }

                let len = format_record(rec, &mut line);

                // Write to stdout
                let _ = write(STDOUT, &line[..len]);

                // Write to log file if available
                if let Some(fd) = log_fd {
                    // Check if rotation is needed before writing
                    let log_path = &config.log_file[..config.log_file_len];
                    let _ = rotator.check_and_rotate(log_path, Some(fd));

                    let _ = write(fd, &line[..len]);
                    rotator.add_bytes(len);
                }
            }
            let _ = ack_kernel_logs(count);

            // Periodic flush (placeholder - no sync syscall yet)
            let now = last_flush.wrapping_add(1); // Simple counter for now
            if now.wrapping_sub(last_flush) >= config.flush_interval_ms as u64 {
                // TODO: Implement flush when sync syscall is available
                // For now, writes are unbuffered
                last_flush = now;
            }
        } else {
            cpu_relax();
        }
    }
}

fn format_config_msg(out: &mut [u8], msg: &str) -> usize {
    let mut pos = 0;
    pos = push(out, pos, b"[INIT] logd ");
    pos = push(out, pos, msg.as_bytes());
    pos = push(out, pos, b"\n");
    pos
}

fn format_record(rec: &UserLogRecord, out: &mut [u8]) -> usize {
    let mut pos = 0;

    pos = push(out, pos, b"[");
    pos = push_decimal(out, pos, rec.ts);
    pos = push(out, pos, b"]");

    pos = push(out, pos, b"[CPU");
    pos = push_decimal(out, pos, rec.cpu_id as u64);
    pos = push(out, pos, b"]");

    pos = push(out, pos, b"[TID ");
    pos = push_decimal(out, pos, rec.tid);
    pos = push(out, pos, b"]");

    pos = push(out, pos, b"[");
    pos = push(out, pos, rec.level.as_str().as_bytes());
    pos = push(out, pos, b"]");

    pos = push(out, pos, b"[");
    let subsys_len = core::cmp::min(rec.subsystem_len as usize, LOG_SUBSYS_MAX);
    pos = push(out, pos, &rec.subsystem[..subsys_len]);
    pos = push(out, pos, b"] ");

    let msg_len = core::cmp::min(rec.msg_len as usize, LOG_MSG_MAX);
    pos = push(out, pos, &rec.msg[..msg_len]);

    pos = push(out, pos, b"\n");
    pos
}

fn push(out: &mut [u8], mut pos: usize, data: &[u8]) -> usize {
    for b in data {
        if pos < out.len() {
            out[pos] = *b;
            pos += 1;
        }
    }
    pos
}

fn push_decimal(out: &mut [u8], mut pos: usize, mut val: u64) -> usize {
    let mut buf = [0u8; 20];
    let mut i = 0;
    if val == 0 {
        buf[0] = b'0';
        i = 1;
    } else {
        while val > 0 && i < buf.len() {
            buf[i] = b'0' + (val % 10) as u8;
            val /= 10;
            i += 1;
        }
    }
    while i > 0 {
        i -= 1;
        if pos < out.len() {
            out[pos] = buf[i];
            pos += 1;
        }
    }
    pos
}

#[inline]
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
