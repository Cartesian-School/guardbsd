// userland/logd/src/main.rs
// Kernel log daemon: pulls ring buffer records and emits to serial/FS.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use gbsd::*;
use kernel_log::{LogLevel, UserLogRecord, LOG_MSG_MAX, LOG_SUBSYS_MAX};

const STDOUT: Fd = 1;
const LOG_BATCH: usize = 16;
const LINE_CAP: usize = LOG_MSG_MAX + LOG_SUBSYS_MAX + 64;
const LOG_PATH: &[u8] = b"/var/log/kernel.log\0";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    logd_main()
}

fn logd_main() -> ! {
    let mut records = [UserLogRecord::empty(); LOG_BATCH];
    let mut line = [0u8; LINE_CAP];
    let mut log_fd: Option<Fd> = None;

    loop {
        if log_fd.is_none() {
            if let Ok(fd) = open(LOG_PATH, O_CREAT | O_WRONLY) {
                log_fd = Some(fd);
            }
        }

        let count = match read_kernel_logs(&mut records) {
            Ok(c) => c,
            Err(_) => 0,
        };

        if count > 0 {
            for rec in records.iter().take(count) {
                let len = format_record(rec, &mut line);
                let _ = write(STDOUT, &line[..len]);
                if let Some(fd) = log_fd {
                    let _ = write(fd, &line[..len]);
                }
            }
            let _ = ack_kernel_logs(count);
        } else {
            cpu_relax();
        }
    }
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

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    exit(1);
}
