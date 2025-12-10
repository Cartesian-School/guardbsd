// Minimal GuardBSD init (PID 1)
// Prints its PID via syscalls and exits.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod syscalls {
    include!("../../../shared/syscall_numbers.rs");

    #[inline(always)]
    pub fn getpid() -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") SYS_GETPID as u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        ret
    }

    #[inline(always)]
    pub fn write(fd: u64, buf: &[u8]) -> i64 {
        let ret: i64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") SYS_WRITE as u64,
                in("rdi") fd,
                in("rsi") buf.as_ptr() as u64,
                in("rdx") buf.len() as u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        ret
    }

    #[inline(always)]
    pub fn exit(code: u64) -> ! {
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") SYS_EXIT as u64,
                in("rdi") code,
                options(noreturn)
            );
        }
    }

    #[inline(always)]
    pub fn sigreturn() -> i64 {
        let ret: i64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") SYS_SIGRETURN as u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        ret
    }

    #[inline(always)]
    pub fn kill(pid: i32, sig: i32) -> i64 {
        let ret: i64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") SYS_KILL as u64,
                in("rdi") pid as u64,
                in("rsi") sig as u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        ret
    }

    #[inline(always)]
    pub fn signal(sig: i32, handler: u64) -> i64 {
        let ret: i64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") SYS_SIGNAL_REGISTER as u64,
                in("rdi") sig as u64,
                in("rsi") handler,
                lateout("rax") ret,
                options(nostack)
            );
        }
        ret
    }

    #[inline(always)]
    pub fn service_register(name: &[u8], pid: u64) -> i64 {
        let ret: i64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") SYS_SERVICE_REGISTER as u64,
                in("rdi") name.as_ptr() as u64,
                in("rsi") pid,
                lateout("rax") ret,
                options(nostack)
            );
        }
        ret
    }

    #[inline(always)]
    pub fn fork() -> i64 {
        let ret: i64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") SYS_FORK as u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        ret
    }

    #[inline(always)]
    pub fn waitpid(pid: i32, status_ptr: *mut i32, options: i32) -> i64 {
        let ret: i64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") SYS_WAITPID as u64,
                in("rdi") pid as u64,
                in("rsi") status_ptr as u64,
                in("rdx") options as u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        ret
    }
}

#[no_mangle]
pub extern "C" fn init_signal_handler(_sig: i32) {
    let msg = b"init: signal handler executed\n";
    let _ = syscalls::write(1, msg);
    let _ = syscalls::sigreturn();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let pid = syscalls::getpid();
    let parent_pid = pid as i32;
    let mut buf = [0u8; 96];
    let msg = b"init: pid = ";
    let mut idx = 0;
    for &b in msg {
        buf[idx] = b;
        idx += 1;
    }
    // append pid decimal
    append_decimal(&mut buf, &mut idx, pid);
    buf[idx] = b'\n';
    idx += 1;

    let _ = syscalls::write(1, &buf[..idx]);
    let _ = syscalls::service_register(b"init\0", pid);
    // Install real handler for SIGUSR1 (10)
    let handler_addr = init_signal_handler as usize as u64;
    let _ = syscalls::signal(10, handler_addr);
    // Self-signal
    let _ = syscalls::kill(pid as i32, 10);
    // After handler returns via sigreturn, continue
    let _ = syscalls::write(1, b"init: after signal\n");

    // Fork/wait test
    let child = syscalls::fork() as i32;
    if child == 0 {
        // Child path
        let mypid = syscalls::getpid() as i32;
        let mut buf = [0u8; 96];
        let mut idx = 0;
        let prefix = b"child: pid = ";
        for &b in prefix {
            buf[idx] = b;
            idx += 1;
        }
        append_decimal(&mut buf, &mut idx, mypid as u64);
        let mid = b", parent = ";
        for &b in mid {
            buf[idx] = b;
            idx += 1;
        }
        append_decimal(&mut buf, &mut idx, parent_pid as u64);
        buf[idx] = b'\n';
        idx += 1;
        let _ = syscalls::write(1, &buf[..idx]);
        syscalls::exit(42);
    } else if child > 0 {
        // Parent path
        let mut status: i32 = 0;
        let ret = syscalls::waitpid(child, &mut status as *mut i32, 0) as i32;
        if ret <= 0 {
            let msg = b"init: waitpid failed\n";
            let _ = syscalls::write(1, msg);
            syscalls::exit(1);
        }
        let mut buf = [0u8; 96];
        let mut idx = 0;
        let prefix = b"init: child PID ";
        for &b in prefix {
            buf[idx] = b;
            idx += 1;
        }
        append_decimal(&mut buf, &mut idx, child as u64);
        let mid = b" exited with status ";
        for &b in mid {
            buf[idx] = b;
            idx += 1;
        }
        append_decimal(&mut buf, &mut idx, status as u64);
        buf[idx] = b'\n';
        idx += 1;
        let _ = syscalls::write(1, &buf[..idx]);
        syscalls::exit(0);
    } else {
        let msg = b"init: fork failed\n";
        let _ = syscalls::write(1, msg);
        syscalls::exit(1);
    }
}

fn append_decimal(buf: &mut [u8], idx: &mut usize, mut val: u64) {
    if val == 0 {
        buf[*idx] = b'0';
        *idx += 1;
        return;
    }
    let mut digits = [0u8; 20];
    let mut d = 0;
    while val > 0 {
        digits[d] = (val % 10) as u8 + b'0';
        val /= 10;
        d += 1;
    }
    while d > 0 {
        d -= 1;
        buf[*idx] = digits[d];
        *idx += 1;
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    syscalls::exit(1);
}
