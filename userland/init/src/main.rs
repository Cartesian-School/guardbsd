// userland/init/src/main.rs
// GuardBSD init process: minimal bootstrap

#![no_std]
#![no_main]

use gbsd::{exit, exec, getpid, write};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_main()
}

fn init_main() -> ! {
    let pid = getpid().unwrap_or(0);
    println_pid("[INIT] pid=", pid);
    println("[INIT] GuardBSD init started");
    println("[INIT] exec /bin/gsh ...");

    let path = b"/bin/gsh\0";
    match exec(path) {
        Ok(()) => {
            println("[INIT] ERROR: exec(\"/bin/gsh\") returned unexpectedly");
            exit(1);
        }
        Err(err) => {
            println_errno("[INIT] exec(\"/bin/gsh\") failed: errno=", err);
            exit(1);
        }
    }
}

fn println(msg: &str) {
    let mut buf = [0u8; 256];
    let mut pos = 0;

    for &b in msg.as_bytes() {
        if pos < buf.len() - 1 {
            buf[pos] = b;
            pos += 1;
        }
    }
    buf[pos] = b'\n';
    pos += 1;

    let _ = write(1, &buf[..pos]); // stdout
}

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

fn println_pid(prefix: &str, pid: u64) {
    let mut buf = [0u8; 64];
    let mut pos = 0;
    for &b in prefix.as_bytes() {
        if pos < buf.len() {
            buf[pos] = b;
            pos += 1;
        }
    }
    let pos_after = write_num(&mut buf, pos, pid);
    if pos_after < buf.len() {
        buf[pos_after] = b'\n';
    }
    let _ = write(1, &buf[..core::cmp::min(pos_after + 1, buf.len())]);
}

fn write_num(out: &mut [u8], mut pos: usize, mut val: u64) -> usize {
    let mut tmp = [0u8; 20];
    let mut i = 0;
    if val == 0 {
        tmp[0] = b'0';
        i = 1;
    } else {
        while val > 0 && i < tmp.len() {
            tmp[i] = b'0' + (val % 10) as u8;
            val /= 10;
            i += 1;
        }
    }
    while i > 0 {
        i -= 1;
        if pos < out.len() {
            out[pos] = tmp[i];
            pos += 1;
        }
    }
    pos
}

fn println_errno(prefix: &str, err: gbsd::error::Error) {
    let mut buf = [0u8; 128];
    let mut pos = 0;
    for &b in prefix.as_bytes() {
        if pos < buf.len() {
            buf[pos] = b;
            pos += 1;
        }
    }
    let code = err as u64;
    let pos_after = write_num(&mut buf, pos, code);
    if pos_after < buf.len() {
        buf[pos_after] = b'\n';
    }
    let _ = write(1, &buf[..core::cmp::min(pos_after + 1, buf.len())]);
}
