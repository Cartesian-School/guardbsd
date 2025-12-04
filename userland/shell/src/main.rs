// userland/shell/src/main.rs
// GuardBSD Shell (gsh) - zsh-inspired
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use gbsd::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    shell_main();
}

fn shell_main() -> ! {
    let pid = getpid().unwrap_or(0);
    print_pid(b"gsh: pid=", pid);
    let _ = println(b" interactive shell started");

    loop {
        let _ = println(b"gsh> ");
        cpu_relax();
    }
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

fn print_pid(prefix: &[u8], pid: u64) {
    let mut buf = [0u8; 64];
    let mut pos = 0;
    for &b in prefix {
        if pos < buf.len() {
            buf[pos] = b;
            pos += 1;
        }
    }
    let pos_after = write_num(&mut buf, pos, pid);
    let _ = println(&buf[..core::cmp::min(pos_after, buf.len())]);
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

