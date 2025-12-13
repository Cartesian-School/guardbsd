//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: simple_shell
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Minimalna przykładowa powłoka (syscall write).

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

const SYS_WRITE: usize = 1;

fn syscall(num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    let ret: isize;
    unsafe {
        core::arch::asm!(
            "int 0x80",
            in("rax") num,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            lateout("rax") ret,
        );
    }
    ret
}

fn print(s: &str) {
    syscall(SYS_WRITE, 1, s.as_ptr() as usize, s.len());
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print("GuardBSD Shell v1.0\n");
    print("Type 'help' for commands\n\n");
    
    loop {
        print("GuardBSD# ");
        for _ in 0..50000000 {
            unsafe { core::arch::asm!("nop"); }
        }
    }
}
