// servers/init/src/main.rs
// GuardBSD Init Process (PID 1)
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use core::panic::PanicInfo;

// GuardBSD syscall interface
mod syscalls {
    // Import canonical syscall numbers from shared module
    include!("../../../shared/syscall_numbers.rs");

    // Compatibility aliases for this module
    pub const GBSD_SYS_PORT_CREATE: u64 = SYS_IPC_PORT_CREATE as u64;
    pub const GBSD_SYS_PORT_SEND: u64 = SYS_IPC_SEND as u64;
    pub const GBSD_SYS_PORT_RECEIVE: u64 = SYS_IPC_RECV as u64;
    pub const GBSD_SYS_EXIT: u64 = SYS_EXIT as u64;
    pub const GBSD_SYS_EXEC: u64 = SYS_EXEC as u64;
    pub const GBSD_SYS_FORK: u64 = SYS_FORK as u64;
    pub const GBSD_SYS_OPEN: u64 = SYS_OPEN as u64;
    pub const GBSD_SYS_DUP2: u64 = SYS_DUP2 as u64;
    pub const GBSD_SYS_CLOSE: u64 = SYS_CLOSE as u64;
    pub const GBSD_SYS_SETPGID: u64 = SYS_SETPGID as u64;
    pub const GBSD_SYS_GETPGID: u64 = SYS_GETPGID as u64;
    pub const GBSD_SYS_TCSETPGRP: u64 = SYS_TCSETPGRP as u64;
    pub const GBSD_SYS_WAITPID: u64 = SYS_WAITPID as u64;

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn port_create() -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_PORT_CREATE,
                in("rdi") 0u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        ret
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn port_send(port: u64, buffer: *const u8, length: usize) -> Result<(), ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_PORT_SEND,
                in("rdi") port,
                in("rsi") buffer as u64,
                in("rdx") length as u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn port_receive(port: u64, buffer: *mut u8, length: usize) -> Result<u64, ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_PORT_RECEIVE,
                in("rdi") port,
                in("rsi") buffer as u64,
                in("rdx") length as u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(ret)
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn fork() -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_FORK,
                lateout("rax") ret,
                options(nostack)
            );
        }
        ret
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn exec(path: &[u8]) -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_EXEC,
                in("rdi") path.as_ptr() as u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        ret
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn fork() -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_FORK,
                lateout("x0") ret,
                options(nostack)
            );
        }
        ret
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn port_create() -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_PORT_CREATE,
                in("x0") 0u64,
                lateout("x0") ret,
                options(nostack)
            );
        }
        ret
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn port_send(port: u64, buffer: *const u8, length: usize) -> Result<(), ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_PORT_SEND,
                in("x0") port,
                in("x1") buffer as u64,
                in("x2") length as u64,
                lateout("x0") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn port_receive(port: u64, buffer: *mut u8, length: usize) -> Result<u64, ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_PORT_RECEIVE,
                in("x0") port,
                in("x1") buffer as u64,
                in("x2") length as u64,
                lateout("x0") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(ret)
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn exec(path: &[u8]) -> u64 {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_EXEC,
                in("x0") path.as_ptr() as u64,
                lateout("x0") ret,
                options(nostack)
            );
        }
        ret
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn open(path: &[u8], flags: u64) -> Result<u64, ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_OPEN,
                in("rdi") path.as_ptr() as u64,
                in("rsi") flags,
                lateout("rax") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(ret)
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn dup2(oldfd: u64, newfd: u64) -> Result<u64, ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_DUP2,
                in("rdi") oldfd,
                in("rsi") newfd,
                lateout("rax") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(ret)
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn close(fd: u64) -> Result<(), ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_CLOSE,
                in("rdi") fd,
                lateout("rax") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(())
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn setpgid(pid: usize, pgid: usize) -> Result<(), ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_SETPGID,
                in("rdi") pid as u64,
                in("rsi") pgid as u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(())
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn getpgid(pid: usize) -> Result<usize, ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_GETPGID,
                in("rdi") pid as u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(ret as usize)
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn tcsetpgrp(fd: usize, pgid: u64) -> Result<(), ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_TCSETPGRP,
                in("rdi") fd as u64,
                in("rsi") pgid,
                lateout("rax") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(())
        }
    }
    
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn waitpid(pid: isize, options: u32) -> Result<Option<(usize, i32)>, ()> {
        let ret: u64;
        let mut status: i32 = 0;
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_WAITPID,
                in("rdi") pid as i64 as u64,
                in("rsi") &mut status as *mut i32 as u64,
                in("rdx") options as u64,
                lateout("rax") ret,
                options(nostack)
            );
        }
        let ret_i = ret as i64;
        if ret_i < 0 {
            Err(())
        } else if ret_i == 0 {
            Ok(None)
        } else {
            Ok(Some((ret_i as usize, status)))
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub fn exit(code: u64) -> ! {
        unsafe {
            core::arch::asm!(
                "int 0x80",
                in("rax") GBSD_SYS_EXIT,
                in("rdi") code,
                options(noreturn)
            );
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn open(path: &[u8], flags: u64) -> Result<u64, ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_OPEN,
                in("x0") path.as_ptr() as u64,
                in("x1") flags,
                lateout("x0") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(ret)
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn dup2(oldfd: u64, newfd: u64) -> Result<u64, ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_DUP2,
                in("x0") oldfd,
                in("x1") newfd,
                lateout("x0") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(ret)
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn close(fd: u64) -> Result<(), ()> {
        let ret: u64;
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_CLOSE,
                in("x0") fd,
                lateout("x0") ret,
                options(nostack)
            );
        }
        if (ret as i64) < 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    pub fn exit(code: u64) -> ! {
        unsafe {
            core::arch::asm!(
                "svc #0",
                in("x8") GBSD_SYS_EXIT,
                in("x0") code,
                options(noreturn)
            );
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init_main();
    syscalls::exit(0);
}

fn init_main() {
    // Set init as its own process group leader
    let _ = syscalls::setpgid(0, 0);
    
    // Set init as foreground process group for console initially
    let init_pgid = syscalls::getpgid(0).unwrap_or(1);
    let _ = syscalls::tcsetpgrp(0, init_pgid as u64);
    
    // Create IPC port for init process
    let port = syscalls::port_create();
    if port == 0 {
        syscalls::exit(1);
    }

    // Startup sequence: microkernels → servers → logd → shell
    startup_sequence();

    // After startup, init becomes the parent of all processes
    // Reap zombie children and handle signals
    loop {
        // Reap any zombie children (non-blocking)
        while let Ok(Some((_pid, _status))) = syscalls::waitpid(-1, 1) {
            // Child exited - in production, restart critical services
        }
        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("pause", options(nomem, nostack));
        }

        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("yield", options(nomem, nostack));
        }
    }
}

fn startup_sequence() {
    // Phase 1: Microkernels are already loaded by kernel

    // Phase 2: Start system servers
    start_servers();

    // Phase 3: Start logd (logging daemon)
    start_logd();

    // Phase 4: Start shell
    start_shell();
}

fn start_servers() {
    // Start core system servers
    // Note: In microkernel architecture, these run as separate processes
    
    // Phase 1: Start RAMFS (must be first - provides storage backend)
    let ramfs_pid = start_ramfs();
    if ramfs_pid == 0 {
        // RAMFS failed to start - system cannot function
        // For now, continue anyway (servers may already be loaded)
        return;
    }
    
    // Wait for RAMFS to initialize (simple delay)
    wait_for_server_ready();
    
    // Phase 2: Start VFS (depends on RAMFS)
    let vfs_pid = start_vfs();
    if vfs_pid == 0 {
        // VFS failed to start - system can still work with direct kernel paths
        return;
    }
    
    // Wait for VFS to mount RAMFS
    wait_for_server_ready();
    
    // Phase 3: Start device server (for device registration and management)
    let devd_pid = start_devd();
    if devd_pid == 0 {
        // devd failed to start - system can work without it but devices won't be managed
        return;
    }
    
    // Wait for devd to initialize
    wait_for_server_ready();
    
    // Phase 4: Setup /dev directory and device nodes
    setup_dev_nodes();
    
    // Phase 5: Wire /dev/console to stdio (fd 0/1/2)
    setup_console_stdio();
}

fn start_logd() {
    // Start the logging daemon
    let path = b"/bin/logd\0";
    let ret = syscalls::exec(path);

    if ret != 0 {
        // Logd failed to start - log error but continue
        // System can still function without logd (logs go to serial)
        // In production, this would log to serial console
        return;
    }

    // Wait for logd to register itself (simple polling)
    // In practice, this would use IPC to wait for registration
    for _ in 0..1000 {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("pause", options(nomem, nostack));
        }

        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("yield", options(nomem, nostack));
        }
    }
}

fn start_shell() {
    // Fork and start the GuardBSD shell with job control
    let pid = syscalls::fork();
    
    if pid == 0 {
        // Child process - will become the shell
        
        // Put shell in its own process group
        let _ = syscalls::setpgid(0, 0);
        
        // Make shell the foreground process group
        let shell_pgid = syscalls::getpgid(0).unwrap_or(0);
        let _ = syscalls::tcsetpgrp(0, shell_pgid as u64);
        
        // Execute shell
        let path = b"/bin/gsh\0";
        let ret = syscalls::exec(path);
        
        // If exec fails, exit child
        if ret != 0 {
            syscalls::exit(1);
        }
    } else if pid > 0 {
        // Parent (init) - ensure shell's pgid is set
        let _ = syscalls::setpgid(pid as usize, pid as usize);
        
        // Give terminal to shell
        let _ = syscalls::tcsetpgrp(0, pid as u64);
        
        // Init doesn't wait for shell - shell runs as long-lived process
    }
    // If fork failed (pid < 0), just return
}

fn start_ramfs() -> u64 {
    // Fork and exec RAMFS server
    let pid = syscalls::fork();
    if pid == 0 {
        // Child process - exec ramfs
        let path = b"/servers/ramfs\0";
        let ret = syscalls::exec(path);
        if ret != 0 {
            // Exec failed - exit child
            syscalls::exit(1);
        }
    }
    // Parent returns PID of child (or 0 on fork failure)
    pid
}

fn start_vfs() -> u64 {
    // Fork and exec VFS server
    let pid = syscalls::fork();
    if pid == 0 {
        // Child process - exec vfs
        let path = b"/servers/vfs\0";
        let ret = syscalls::exec(path);
        if ret != 0 {
            // Exec failed - exit child
            syscalls::exit(1);
        }
    }
    // Parent returns PID of child (or 0 on fork failure)
    pid
}

fn start_devd() -> u64 {
    // Fork and exec device daemon
    let pid = syscalls::fork();
    if pid == 0 {
        // Child process - exec devd
        let path = b"/servers/devd\0";
        let ret = syscalls::exec(path);
        if ret != 0 {
            // Exec failed - exit child
            syscalls::exit(1);
        }
    }
    // Parent returns PID of child (or 0 on fork failure)
    pid
}

fn wait_for_server_ready() {
    // Simple delay to let servers initialize
    // In production, this would use IPC handshake
    for _ in 0..10000 {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("pause", options(nomem, nostack));
        }

        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("yield", options(nomem, nostack));
        }
    }
}

fn setup_dev_nodes() {
    // Create /dev directory and populate with device nodes
    // This function uses direct IPC to RAMFS and devd servers
    
    const RAMFS_PORT: u64 = 1001;
    const DEVD_PORT: u64 = 1100;
    
    // Step 1: Create /dev directory via RAMFS
    // Op 6 = mkdir
    let mut req_buf = [0u8; 512];
    req_buf[0..4].copy_from_slice(&6u32.to_le_bytes()); // mkdir op
    req_buf[4..8].copy_from_slice(&0u32.to_le_bytes()); // reply port (not used in simplified impl)
    
    // Path "/dev"
    let dev_path = b"/dev\0";
    req_buf[8..8 + dev_path.len()].copy_from_slice(dev_path);
    
    // Send mkdir request to RAMFS
    if syscalls::port_send(RAMFS_PORT, req_buf.as_ptr(), 512).is_ok() {
        let mut resp_buf = [0u8; 512];
        let _ = syscalls::port_receive(RAMFS_PORT, resp_buf.as_mut_ptr(), 512);
        // Ignore errors - /dev might already exist
    }
    
    // Step 2: Register devices with devd and create nodes
    
    // Device 1: /dev/null (character device 1,3)
    if let Ok(null_dev_id) = register_device_with_devd(DEVD_PORT, 0, 1, 3) {
        create_device_node_in_ramfs(RAMFS_PORT, b"/dev/null\0", null_dev_id);
    }
    
    // Device 2: /dev/console (character device 1,0 - already pre-registered as device 1)
    // Use the pre-registered console device (ID 1)
    create_device_node_in_ramfs(RAMFS_PORT, b"/dev/console\0", 1);
    
    // Device 3: /dev/tty0 (alias to console, character device 4,0)
    if let Ok(tty0_dev_id) = register_device_with_devd(DEVD_PORT, 0, 4, 0) {
        create_device_node_in_ramfs(RAMFS_PORT, b"/dev/tty0\0", tty0_dev_id);
    }
}

fn register_device_with_devd(devd_port: u64, dev_type: u32, major: u16, minor: u16) -> Result<u32, ()> {
    // Register device via IPC to devd
    // Format: DevRequest [op:u32][dev_id:u32][major:u16][minor:u16][flags:u32]
    let mut req_buf = [0u8; 16];
    req_buf[0..4].copy_from_slice(&1u32.to_le_bytes()); // op=1 (register)
    req_buf[4..8].copy_from_slice(&0u32.to_le_bytes()); // dev_id (unused for register)
    req_buf[8..10].copy_from_slice(&major.to_le_bytes());
    req_buf[10..12].copy_from_slice(&minor.to_le_bytes());
    req_buf[12..16].copy_from_slice(&dev_type.to_le_bytes()); // flags = device type
    
    if syscalls::port_send(devd_port, req_buf.as_ptr(), 16).is_ok() {
        let mut resp_buf = [0u8; 8];
        if syscalls::port_receive(devd_port, resp_buf.as_mut_ptr(), 8).is_ok() {
            let result = i64::from_le_bytes(resp_buf);
            if result >= 0 {
                return Ok(result as u32);
            }
        }
    }
    Err(())
}

fn create_device_node_in_ramfs(ramfs_port: u64, path: &[u8], dev_id: u32) {
    // Create device node via RAMFS mknod operation
    // Op 9 = mknod, Format: [op:u32][reply_port:u32][path:256][dev_id:u32]
    let mut req_buf = [0u8; 512];
    req_buf[0..4].copy_from_slice(&9u32.to_le_bytes()); // mknod op
    req_buf[4..8].copy_from_slice(&0u32.to_le_bytes()); // reply port
    
    let path_len = path.iter().position(|&c| c == 0).unwrap_or(path.len()).min(256);
    req_buf[8..8 + path_len].copy_from_slice(&path[..path_len]);
    req_buf[264..268].copy_from_slice(&dev_id.to_le_bytes());
    
    if syscalls::port_send(ramfs_port, req_buf.as_ptr(), 512).is_ok() {
        let mut resp_buf = [0u8; 512];
        let _ = syscalls::port_receive(ramfs_port, resp_buf.as_mut_ptr(), 512);
        // Ignore errors for now
    }
}

fn setup_console_stdio() {
    // Wire /dev/console to stdio (fd 0/1/2)
    // This makes console the standard input/output/error for init and all child processes
    
    const O_RDWR: u64 = 0x2;
    let console_path = b"/dev/console\0";
    
    // Open /dev/console
    match syscalls::open(console_path, O_RDWR) {
        Ok(console_fd) => {
            // Duplicate console_fd to stdin (0), stdout (1), stderr (2)
            // Note: Current sys_close doesn't allow closing 0/1/2, so we use dup2 directly
            
            let _ = syscalls::dup2(console_fd, 0); // stdin
            let _ = syscalls::dup2(console_fd, 1); // stdout
            let _ = syscalls::dup2(console_fd, 2); // stderr
            
            // If console_fd is > 2, close the original to avoid fd leak
            if console_fd > 2 {
                let _ = syscalls::close(console_fd);
            }
            
            // Now fd 0/1/2 point to /dev/console
            // All child processes (logd, gsh) will inherit these fds
        }
        Err(_) => {
            // /dev/console open failed
            // Fall back to existing kernel console behavior
            // In production, this would log to serial console directly
            // For now, just continue - stdio will use kernel defaults
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    syscalls::exit(1);
}
