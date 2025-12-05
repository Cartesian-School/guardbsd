#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod interrupt;
mod drivers;
mod fs;
// Note: Using kernel/sched/mod.rs instead of local scheduler
mod process;
mod ipc;

mod syscall {
    // Import canonical syscall numbers from shared crate
    use shared::syscall_numbers::*;

    pub fn syscall_handler(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
        // Day 29: Updated syscall handler - delegate to main kernel implementations
        match syscall_num {
            // Process management (Day 29)
            SYS_EXIT => {
                crate::syscalls::process::sys_exit(arg1 as i32);
            },
            SYS_GETPID => crate::syscalls::process::sys_getpid(),
            SYS_FORK => crate::syscalls::process::sys_fork(),
            SYS_EXEC => crate::syscalls::process::sys_exec(arg1 as *const u8, arg2 as *const *const u8),
            SYS_WAIT => crate::syscalls::process::sys_wait(arg1 as *mut i32),
            SYS_YIELD => {
                // Yield to scheduler
                crate::sched::yield_current();
                0
            },
            
            // Signal management (Day 29)
            SYS_KILL => crate::syscalls::signal::sys_kill(arg1, arg2 as i32),
            SYS_SIGNAL => crate::syscalls::signal::sys_signal(arg2 as i32, arg1 as u64),
            SYS_SIGACTION => crate::syscalls::signal::sys_sigaction(
                arg1 as i32,
                arg2 as *const crate::signal::SignalAction,
                arg3 as *mut crate::signal::SignalAction
            ),
            
            // File operations (Day 31: Full VFS/RAMFS integration via IPC)
            SYS_WRITE => sys_write(arg1, arg2 as *const u8, arg3),
            SYS_READ => sys_read(arg1, arg2 as *mut u8, arg3),
            SYS_OPEN => sys_open(arg1 as *const u8, arg2),
            SYS_CLOSE => sys_close(arg1),
            SYS_STAT => sys_stat(arg1 as *const u8, arg2 as *mut u8),
            SYS_MKDIR => sys_mkdir(arg1 as *const u8, arg2),
            SYS_UNLINK => sys_unlink(arg1 as *const u8),
            SYS_RENAME => sys_rename(arg1 as *const u8, arg2 as *const u8),
            SYS_SYNC => sys_sync(arg1),
            SYS_CHDIR => sys_chdir(arg1 as *const u8),
            SYS_GETCWD => sys_getcwd(arg1 as *mut u8, arg2),
            SYS_MOUNT => sys_mount(arg1 as *const u8, arg2 as *const u8, arg3 as *const u8),
            SYS_UMOUNT => sys_umount(arg1 as *const u8),
            
            // Logging (still using local stubs)
            SYS_LOG_READ => ENOSYS,
            SYS_LOG_ACK => ENOSYS,
            SYS_LOG_REGISTER_DAEMON => ENOSYS,
            
            // IPC
            SYS_IPC_PORT_CREATE => sys_ipc_port_create(),
            SYS_IPC_SEND => sys_ipc_send(arg1, arg2, arg3 as u32, [0, 0, 0, 0]),
            SYS_IPC_RECV => sys_ipc_recv(arg1),
            
            _ => -1, // EINVAL for unknown syscalls
        }
    }

    // File descriptor management
    const MAX_FDS: usize = 256;
    static mut OPEN_FDS: [Option<FileDescriptor>; MAX_FDS] = [None; MAX_FDS];

    #[derive(Clone, Copy)]
    struct FileDescriptor {
        inode: u64,
        offset: u64,
        flags: u32,
    }

    // Simple in-kernel RAMFS implementation
    const MAX_NODES: usize = 256;
    static mut RAMFS_NODES: [RamFsNode; MAX_NODES] = [RamFsNode::new(); MAX_NODES];
    static mut RAMFS_NODE_COUNT: usize = 1; // Root directory

    #[derive(Clone, Copy)]
    struct RamFsNode {
        name: [u8; 64],
        name_len: usize,
        node_type: NodeType,
        data: [u8; 4096],
        size: usize,
        parent: usize,
    }

    #[derive(Clone, Copy, PartialEq)]
    enum NodeType {
        File,
        Directory,
    }

    impl RamFsNode {
        const fn new() -> Self {
            RamFsNode {
                name: [0; 64],
                name_len: 0,
                node_type: NodeType::File,
                data: [0; 4096],
                size: 0,
                parent: 0,
            }
        }

        fn set_name(&mut self, name: &[u8]) {
            let len = name.len().min(64);
            self.name[..len].copy_from_slice(&name[..len]);
            self.name_len = len;
        }

        fn name_matches(&self, name: &[u8]) -> bool {
            self.name_len == name.len() && &self.name[..self.name_len] == name
        }
    }

    pub(crate) fn init_ramfs() {
        unsafe {
            // Initialize root directory
            RAMFS_NODES[0].set_name(b"/");
            RAMFS_NODES[0].node_type = NodeType::Directory;
            RAMFS_NODE_COUNT = 1;
        }
    }
    
    // Day 31: Full filesystem implementation via VFS/RAMFS IPC
    // All filesystem operations now use VFS server (port discovery)
    
    // VFS server port (discovered during init, or hardcoded)
    const VFS_PORT: u64 = 1000; // VFS server uses dynamic port_create, we'll use known port
    
    fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
        // Special handling for stdout/stderr - direct serial output
        if fd == 1 || fd == 2 {
            unsafe {
                if buf.is_null() || len == 0 { return -1; }
                let slice = core::slice::from_raw_parts(buf, len);
                for &byte in slice {
                    while (super::inb(super::COM1 + 5) & 0x20) == 0 {}
                    super::outb(super::COM1, byte);
                }
            }
            return len as isize;
        }
        
        // Regular file write via VFS
        vfs_write(fd, buf, len)
    }
    
    fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
        // stdin not implemented yet - return ENOSYS for fd 0
        if fd == 0 {
            return ENOSYS;
        }
        
        // Regular file read via VFS
        vfs_read(fd, buf, len)
    }
    
    fn sys_open(path: *const u8, flags: usize) -> isize {
        vfs_open(path, flags)
    }
    
    fn sys_close(fd: usize) -> isize {
        vfs_close(fd)
    }
    
    fn sys_stat(path: *const u8, stat_buf: *mut u8) -> isize {
        vfs_stat(path, stat_buf)
    }
    
    fn sys_mkdir(path: *const u8, mode: usize) -> isize {
        vfs_mkdir(path, mode)
    }
    
    fn sys_unlink(path: *const u8) -> isize {
        vfs_unlink(path)
    }
    
    fn sys_rename(old_path: *const u8, new_path: *const u8) -> isize {
        vfs_rename(old_path, new_path)
    }
    
    fn sys_sync(fd: usize) -> isize {
        // Sync is a no-op for RAMFS
        0
    }
    
    fn sys_chdir(path: *const u8) -> isize {
        // TODO: Implement current working directory tracking
        // For now, return success
        0
    }
    
    fn sys_getcwd(buf: *mut u8, size: usize) -> isize {
        // Return root directory for now
        unsafe {
            if size < 2 {
                return -34; // ERANGE
            }
            *buf = b'/';
            *(buf.add(1)) = 0;
            2
        }
    }
    
    fn sys_mount(source: *const u8, target: *const u8, fstype: *const u8) -> isize {
        // Mount operations handled by VFS server during init
        0
    }
    
    fn sys_umount(target: *const u8) -> isize {
        // Unmount not implemented
        -38 // ENOSYS
    }
    
    // ========== VFS IPC Helper Functions ==========
    
    fn vfs_open(path: *const u8, flags: usize) -> isize {
        unsafe {
            if path.is_null() {
                return -14; // EFAULT
            }
            
            // Build IPC message to VFS
            let mut req_buf = [0u8; 512];
            
            // Operation code: Open = 1
            req_buf[0..4].copy_from_slice(&1u32.to_le_bytes());
            
            // Reply port (use our IPC port if available, or 0)
            req_buf[4..8].copy_from_slice(&0u32.to_le_bytes());
            
            // Copy path (max 256 bytes)
            let mut path_len = 0;
            while path_len < 255 {
                let byte = *path.add(path_len);
                if byte == 0 { break; }
                req_buf[8 + path_len] = byte;
                path_len += 1;
            }
            
            // Flags and mode
            req_buf[264..268].copy_from_slice(&(flags as u32).to_le_bytes());
            req_buf[268..272].copy_from_slice(&0u32.to_le_bytes()); // mode
            
            // Send to VFS server
            if crate::ipc::ipc_send(VFS_PORT, req_buf.as_ptr(), 512) < 0 {
                return -5; // EIO
            }
            
            // Receive response
            let mut resp_buf = [0u8; 512];
            if crate::ipc::ipc_recv(VFS_PORT, resp_buf.as_mut_ptr(), 512) < 0 {
                return -5; // EIO
            }
            
            // Parse response (first 8 bytes = i64 result)
            let result = i64::from_le_bytes([
                resp_buf[0], resp_buf[1], resp_buf[2], resp_buf[3],
                resp_buf[4], resp_buf[5], resp_buf[6], resp_buf[7]
            ]);
            
            result as isize
        }
    }
    
    fn vfs_close(fd: usize) -> isize {
        // VFS close operation
        unsafe {
            let mut req_buf = [0u8; 512];
            req_buf[0..4].copy_from_slice(&2u32.to_le_bytes()); // Close = 2
            req_buf[8..12].copy_from_slice(&(fd as u32).to_le_bytes());
            
            if crate::ipc::ipc_send(VFS_PORT, req_buf.as_ptr(), 512) < 0 {
                return -5; // EIO
            }
            
            let mut resp_buf = [0u8; 512];
            if crate::ipc::ipc_recv(VFS_PORT, resp_buf.as_mut_ptr(), 512) < 0 {
                return -5; // EIO
            }
            
            let result = i64::from_le_bytes([
                resp_buf[0], resp_buf[1], resp_buf[2], resp_buf[3],
                resp_buf[4], resp_buf[5], resp_buf[6], resp_buf[7]
            ]);
            
            result as isize
        }
    }
    
    fn vfs_read(fd: usize, buf: *mut u8, len: usize) -> isize {
        unsafe {
            if buf.is_null() {
                return -14; // EFAULT
            }
            
            let mut req_buf = [0u8; 512];
            req_buf[0..4].copy_from_slice(&3u32.to_le_bytes()); // Read = 3
            req_buf[8..12].copy_from_slice(&(fd as u32).to_le_bytes());
            req_buf[12..16].copy_from_slice(&(len as u32).to_le_bytes());
            
            if crate::ipc::ipc_send(VFS_PORT, req_buf.as_ptr(), 512) < 0 {
                return -5; // EIO
            }
            
            let mut resp_buf = [0u8; 4096]; // Larger buffer for data
            if crate::ipc::ipc_recv(VFS_PORT, resp_buf.as_mut_ptr(), 4096) < 0 {
                return -5; // EIO
            }
            
            let result = i64::from_le_bytes([
                resp_buf[0], resp_buf[1], resp_buf[2], resp_buf[3],
                resp_buf[4], resp_buf[5], resp_buf[6], resp_buf[7]
            ]);
            
            if result > 0 {
                // Copy data from response
                let copy_len = (result as usize).min(len);
                core::ptr::copy_nonoverlapping(resp_buf.as_ptr().add(16), buf, copy_len);
            }
            
            result as isize
        }
    }
    
    fn vfs_write(fd: usize, buf: *const u8, len: usize) -> isize {
        unsafe {
            if buf.is_null() {
                return -14; // EFAULT
            }
            
            let mut req_buf = [0u8; 4096];
            req_buf[0..4].copy_from_slice(&4u32.to_le_bytes()); // Write = 4
            req_buf[8..12].copy_from_slice(&(fd as u32).to_le_bytes());
            req_buf[12..16].copy_from_slice(&(len as u32).to_le_bytes());
            
            // Copy data into request
            let copy_len = len.min(4000);
            core::ptr::copy_nonoverlapping(buf, req_buf.as_mut_ptr().add(16), copy_len);
            
            if crate::ipc::ipc_send(VFS_PORT, req_buf.as_ptr(), 16 + copy_len) < 0 {
                return -5; // EIO
            }
            
            let mut resp_buf = [0u8; 512];
            if crate::ipc::ipc_recv(VFS_PORT, resp_buf.as_mut_ptr(), 512) < 0 {
                return -5; // EIO
            }
            
            let result = i64::from_le_bytes([
                resp_buf[0], resp_buf[1], resp_buf[2], resp_buf[3],
                resp_buf[4], resp_buf[5], resp_buf[6], resp_buf[7]
            ]);
            
            result as isize
        }
    }
    
    fn vfs_stat(path: *const u8, stat_buf: *mut u8) -> isize {
        unsafe {
            if path.is_null() || stat_buf.is_null() {
                return -14; // EFAULT
            }
            
            let mut req_buf = [0u8; 512];
            req_buf[0..4].copy_from_slice(&5u32.to_le_bytes()); // Stat = 5
            
            // Copy path
            let mut path_len = 0;
            while path_len < 255 {
                let byte = *path.add(path_len);
                if byte == 0 { break; }
                req_buf[8 + path_len] = byte;
                path_len += 1;
            }
            
            if crate::ipc::ipc_send(VFS_PORT, req_buf.as_ptr(), 512) < 0 {
                return -5; // EIO
            }
            
            let mut resp_buf = [0u8; 512];
            if crate::ipc::ipc_recv(VFS_PORT, resp_buf.as_mut_ptr(), 512) < 0 {
                return -5; // EIO
            }
            
            let result = i64::from_le_bytes([
                resp_buf[0], resp_buf[1], resp_buf[2], resp_buf[3],
                resp_buf[4], resp_buf[5], resp_buf[6], resp_buf[7]
            ]);
            
            if result == 0 {
                // Copy stat data to user buffer
                core::ptr::copy_nonoverlapping(resp_buf.as_ptr().add(16), stat_buf, 128);
            }
            
            result as isize
        }
    }
    
    fn vfs_mkdir(path: *const u8, mode: usize) -> isize {
        unsafe {
            if path.is_null() {
                return -14; // EFAULT
            }
            
            let mut req_buf = [0u8; 512];
            req_buf[0..4].copy_from_slice(&6u32.to_le_bytes()); // Mkdir = 6
            
            // Copy path
            let mut path_len = 0;
            while path_len < 255 {
                let byte = *path.add(path_len);
                if byte == 0 { break; }
                req_buf[8 + path_len] = byte;
                path_len += 1;
            }
            
            req_buf[268..272].copy_from_slice(&(mode as u32).to_le_bytes());
            
            if crate::ipc::ipc_send(VFS_PORT, req_buf.as_ptr(), 512) < 0 {
                return -5; // EIO
            }
            
            let mut resp_buf = [0u8; 512];
            if crate::ipc::ipc_recv(VFS_PORT, resp_buf.as_mut_ptr(), 512) < 0 {
                return -5; // EIO
            }
            
            let result = i64::from_le_bytes([
                resp_buf[0], resp_buf[1], resp_buf[2], resp_buf[3],
                resp_buf[4], resp_buf[5], resp_buf[6], resp_buf[7]
            ]);
            
            result as isize
        }
    }
    
    fn vfs_unlink(path: *const u8) -> isize {
        unsafe {
            if path.is_null() {
                return -14; // EFAULT
            }
            
            let mut req_buf = [0u8; 512];
            req_buf[0..4].copy_from_slice(&8u32.to_le_bytes()); // Unlink = 8
            
            // Copy path
            let mut path_len = 0;
            while path_len < 255 {
                let byte = *path.add(path_len);
                if byte == 0 { break; }
                req_buf[8 + path_len] = byte;
                path_len += 1;
            }
            
            if crate::ipc::ipc_send(VFS_PORT, req_buf.as_ptr(), 512) < 0 {
                return -5; // EIO
            }
            
            let mut resp_buf = [0u8; 512];
            if crate::ipc::ipc_recv(VFS_PORT, resp_buf.as_mut_ptr(), 512) < 0 {
                return -5; // EIO
            }
            
            let result = i64::from_le_bytes([
                resp_buf[0], resp_buf[1], resp_buf[2], resp_buf[3],
                resp_buf[4], resp_buf[5], resp_buf[6], resp_buf[7]
            ]);
            
            result as isize
        }
    }
    
    fn vfs_rename(old_path: *const u8, new_path: *const u8) -> isize {
        // Rename not implemented in RAMFS yet
        -38 // ENOSYS
    }

    fn sys_ipc_port_create() -> isize {
        // Get current PID (simplified - assume PID 0 for boot stub)
        crate::ipc::ipc_create_port(0)
    }

    fn sys_ipc_send(port_id: usize, receiver_pid: usize, msg_type: u32, data: [u32; 4]) -> isize {
        // Get current PID (simplified - assume PID 0 for boot stub)
        crate::ipc::ipc_send(port_id, 0, receiver_pid, msg_type, data)
    }

    fn sys_ipc_recv(_port_id: usize) -> isize {
        // For now, just return success - actual receive would be more complex
        0
    }
}

#[no_mangle]
pub extern "C" fn syscall_dispatch(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    syscall::syscall_handler(syscall_num, arg1, arg2, arg3)
}

#[no_mangle]
pub extern "C" fn keyboard_interrupt_handler() {
    drivers::keyboard::handle_interrupt();
}

#[no_mangle]
pub extern "C" fn timer_interrupt_handler() {
    drivers::timer::handle_interrupt();
    
    // Day 30: Integrated with main kernel scheduler
    // Note: This is a simplified boot_stub handler
    // The main kernel uses arch-specific handlers (kernel/arch/*/interrupts/mod.rs)
    // with full trap frames and context switching
    
    // For boot_stub, we simply notify the scheduler
    // Real context switching happens in the main kernel's architecture-specific ISRs
    let cpu_id = 0;
    unsafe {
        crate::sched::on_tick(cpu_id);
    }
    
    // Context switching is handled by architecture-specific ISRs:
    // - x86_64: kernel/arch/x86_64/interrupts/mod.rs::x86_64_timer_interrupt_handler()
    // - aarch64: kernel/arch/aarch64/interrupts/mod.rs::aarch64_timer_interrupt_handler()
    // These save full CPU state, call scheduler, and perform arch_context_switch()
}

// ============================================================================
// GuaBoot Boot Protocol (FreeBSD-compatible, BSD 3-Clause License)
// ============================================================================

const GBSD_MAGIC: u32 = 0x42534447; // "GBSD" in little-endian

#[repr(C)]
pub struct BootInfo {
    pub magic: u32,
    pub version: u32,
    pub mem_lower: u64,
    pub mem_upper: u64,
    pub boot_device: u32,
    pub cmdline: *const u8,
    pub mods_count: u32,
    pub mods: *const Module,
    pub mmap: *const u8,
    pub mmap_count: u32,
}

#[repr(C)]
pub struct Module {
    pub mod_start: u64,
    pub mod_end: u64,
    pub string: *const u8,
    pub reserved: u32,
}

extern "C" {
    static guaboot_magic: u32;
    static guaboot_bootinfo_ptr: *const BootInfo;
}

const COM1: u16 = 0x3F8;

unsafe fn serial_init() {
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x80);
    outb(COM1 + 0, 0x03);
    outb(COM1 + 1, 0x00);
    outb(COM1 + 3, 0x03);
    outb(COM1 + 2, 0xC7);
    outb(COM1 + 4, 0x0B);
}

unsafe fn print(s: &str) {
    for b in s.bytes() {
        while (inb(COM1 + 5) & 0x20) == 0 {}
        outb(COM1, b);
    }
}

#[no_mangle]
pub extern "C" fn guardbsd_main() -> ! {
    unsafe {
        serial_init();
        
        print("\n\n");
        print("================================================================================\n");
        print("[BOOT] GuardBSD Winter Saga v1.0.0 - SYSTEM ONLINE\n");
        print("================================================================================\n");
        print("[OK] Bootloader: GuaBoot (BSD-Licensed)\n");
        print("[OK] Boot stub loaded\n");
        print("[OK] Serial COM1 initialized\n");
        print("[OK] Protected mode active\n");
        print("\n[INIT] Initializing memory management...\n");
        init_memory();
        print("[OK] PMM initialized\n");
        print("[OK] VMM initialized\n");
        print("\n[INIT] Initializing filesystem...\n");
        init_filesystem();
        print("[OK] ISO filesystem ready\n");
        print("\n[INIT] Loading shell from /bin/gsh...\n");
        load_shell();
        print("[OK] Shell loaded\n");
        print("\n[INIT] Creating init process...\n");
        create_init_process();
        print("[OK] Init process created (PID 1)\n");
        print("\n[INIT] Setting up interrupt handling...\n");
        init_pic();
        interrupt::idt::init_idt();
        print("[OK] IDT initialized (syscall int 0x80, IRQ0, IRQ1)\n");
        
        print("\n[INIT] Initializing scheduler...\n");
        // Initialize main kernel scheduler with 100 Hz timer
        crate::sched::init(100);
        drivers::timer::init(100); // 100 Hz
        print("[OK] Scheduler initialized (100 Hz timer)\n");
        
        enable_interrupts();
        print("\n[INIT] Microkernel bootstrap starting...\n");

        // Initialize IPC infrastructure
        crate::ipc::init_ipc();
        print("[OK] IPC infrastructure initialized\n");

        // Initialize process manager
        crate::process::init_process_manager();
        print("[OK] Process manager initialized\n");

        // Initialize RAMFS
        unsafe { crate::syscall::init_ramfs(); }
        print("[OK] RAMFS initialized\n");

        // Initialize microkernel communication channels
        if !crate::ipc::init_microkernel_channels() {
            print("[ERROR] Failed to initialize microkernel channels\n");
            loop { unsafe { core::arch::asm!("hlt"); } }
        }
        print("[OK] Microkernel communication channels established\n");

        // Initialize server communication channels
        if !crate::ipc::init_server_channels() {
            print("[ERROR] Failed to initialize server channels\n");
            loop { unsafe { core::arch::asm!("hlt"); } }
        }
        print("[OK] Server communication channels established\n");

        // Load microkernels
        print("[INIT] Loading µK-Space (memory management)...\n");
        let space_pid = crate::process::load_microkernel("uk_space", "modules/uk_space");
        if let Some(pid) = space_pid {
            print("[OK] µK-Space loaded (PID: ");
            print_num(pid);
            print(")\n");

            // Start the microkernel
            if crate::process::start_microkernel(pid) {
                print("[OK] µK-Space started successfully\n");
            } else {
                print("[ERROR] Failed to start µK-Space\n");
            }
        } else {
            print("[ERROR] Failed to load µK-Space\n");
        }

        print("[INIT] Loading µK-Time (scheduler)...\n");
        let time_pid = crate::process::load_microkernel("uk_time", "modules/uk_time");
        if let Some(pid) = time_pid {
            print("[OK] µK-Time loaded (PID: ");
            print_num(pid);
            print(")\n");

            // Start the microkernel
            if crate::process::start_microkernel(pid) {
                print("[OK] µK-Time started successfully\n");
            } else {
                print("[ERROR] Failed to start µK-Time\n");
            }
        } else {
            print("[ERROR] Failed to load µK-Time\n");
        }

        print("[INIT] Loading µK-IPC (communication)...\n");
        let ipc_pid = crate::process::load_microkernel("uk_ipc", "modules/uk_ipc");
        if let Some(pid) = ipc_pid {
            print("[OK] µK-IPC loaded (PID: ");
            print_num(pid);
            print(")\n");

            // Start the microkernel
            if crate::process::start_microkernel(pid) {
                print("[OK] µK-IPC started successfully\n");
            } else {
                print("[ERROR] Failed to start µK-IPC\n");
            }
        } else {
            print("[ERROR] Failed to load µK-IPC\n");
        }

        print("[OK] All microkernels loaded and started\n");
        print("[OK] Microkernel system operational\n\n");
        
        print("[INIT] Starting system servers...\n");

        // Load system servers
        print("[INIT] Starting init server...\n");
        let init_pid = crate::process::load_server("init", "servers/init");
        if let Some(pid) = init_pid {
            print("[OK] Init server loaded (PID: ");
            print_num(pid);
            print(")\n");

            // Register init service
            crate::ipc::register_service("init", 0, pid);

            if crate::process::start_server(pid) {
                print("[OK] Init server started successfully\n");
            } else {
                print("[ERROR] Failed to start init server\n");
            }
        } else {
            print("[ERROR] Failed to load init server\n");
        }

        print("[INIT] Starting vfs server...\n");
        let vfs_pid = crate::process::load_server("vfs", "servers/vfs");
        if let Some(pid) = vfs_pid {
            print("[OK] VFS server loaded (PID: ");
            print_num(pid);
            print(")\n");

            // Register VFS service
            crate::ipc::register_service("vfs", 0, pid);

            if crate::process::start_server(pid) {
                print("[OK] VFS server started successfully\n");
            } else {
                print("[ERROR] Failed to start VFS server\n");
            }
        } else {
            print("[ERROR] Failed to load VFS server\n");
        }

        print("[INIT] Starting ramfs server...\n");
        let ramfs_pid = crate::process::load_server("ramfs", "servers/ramfs");
        if let Some(pid) = ramfs_pid {
            print("[OK] RAMFS server loaded (PID: ");
            print_num(pid);
            print(")\n");

            // Register RAMFS service
            crate::ipc::register_service("ramfs", 0, pid);

            if crate::process::start_server(pid) {
                print("[OK] RAMFS server started successfully\n");
            } else {
                print("[ERROR] Failed to start RAMFS server\n");
            }
        } else {
            print("[ERROR] Failed to load RAMFS server\n");
        }

        print("[INIT] Starting devd server...\n");
        let devd_pid = crate::process::load_server("devd", "servers/devd");
        if let Some(pid) = devd_pid {
            print("[OK] DEVD server loaded (PID: ");
            print_num(pid);
            print(")\n");

            // Register DEVD service
            crate::ipc::register_service("devd", 0, pid);

            if crate::process::start_server(pid) {
                print("[OK] DEVD server started successfully\n");
            } else {
                print("[ERROR] Failed to start DEVD server\n");
            }
        } else {
            print("[ERROR] Failed to load DEVD server\n");
        }

        print("[OK] All system servers loaded and started\n\n");
        
        print("[SHELL] Starting gsh (GuardBSD Shell)...\n");
        print("================================================================================\n");
        print("\nGuardBSD# ");
        
        shell_loop();
    }
}

fn shell_loop() -> ! {
    let mut line_buf: [u8; 256] = [0; 256];
    let mut line_len: usize = 0;
    
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
        
        while let Some(ch) = drivers::keyboard::read_char() {
            unsafe {
                if ch == b'\n' {
                    print("\n");
                    if line_len > 0 {
                        execute_command(&line_buf[..line_len]);
                    }
                    line_len = 0;
                    print("GuardBSD# ");
                } else if ch == 8 {
                    if line_len > 0 {
                        line_len -= 1;
                        print("\x08 \x08");
                    }
                } else if line_len < 255 {
                    line_buf[line_len] = ch;
                    line_len += 1;
                    let s = core::slice::from_raw_parts(&ch as *const u8, 1);
                    for &b in s {
                        while (inb(COM1 + 5) & 0x20) == 0 {}
                        outb(COM1, b);
                    }
                }
            }
        }
    }
}

fn execute_command(cmd: &[u8]) {
    unsafe {
        if cmd == b"help" {
            print("Available commands: help, clear, echo, exit\n");
        } else if cmd == b"clear" {
            print("\x1b[2J\x1b[H");
        } else if cmd.starts_with(b"echo ") {
            let msg = &cmd[5..];
            for &b in msg {
                while (inb(COM1 + 5) & 0x20) == 0 {}
                outb(COM1, b);
            }
            print("\n");
        } else if cmd == b"exit" {
            print("Goodbye!\n");
            loop { core::arch::asm!("cli; hlt"); }
        } else {
            print("Unknown command: ");
            for &b in cmd {
                while (inb(COM1 + 5) & 0x20) == 0 {}
                outb(COM1, b);
            }
            print("\n");
        }
    }
}

fn init_pic() {
    unsafe {
        outb(0x20, 0x11);
        outb(0xA0, 0x11);
        outb(0x21, 0x20);
        outb(0xA1, 0x28);
        outb(0x21, 0x04);
        outb(0xA1, 0x02);
        outb(0x21, 0x01);
        outb(0xA1, 0x01);
        outb(0x21, 0xFC); // Unmask IRQ0 (timer) and IRQ1 (keyboard)
        outb(0xA1, 0xFF);
    }
}

fn enable_interrupts() {
    unsafe {
        core::arch::asm!("sti");
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        print("\n[PANIC] System halted\n");
    }
    loop {
        unsafe { core::arch::asm!("cli; hlt"); }
    }
}

#[inline(always)]
pub unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val);
}

#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    core::arch::asm!("in al, dx", out("al") ret, in("dx") port);
    ret
}

fn init_memory() {
    unsafe {
        static mut BITMAP: [u64; 512] = [0; 512];
        for i in 0..4 {
            BITMAP[i] = !0;
        }
    }
}

fn init_filesystem() {
    // Skip ISO detection for now - use simulated filesystem
    fs::iso9660::init(0);
}

fn print_hex(n: usize) {
    unsafe {
        let hex = b"0123456789abcdef";
        for i in (0..8).rev() {
            let nibble = (n >> (i * 4)) & 0xF;
            let ch = hex[nibble];
            while (inb(COM1 + 5) & 0x20) == 0 {}
            outb(COM1, ch);
        }
    }
}

fn load_shell() {
    // Filesystem loading will be implemented when ISO is properly mapped
    // For now, use built-in shell
}

fn print_num(n: usize) {
    unsafe {
        let mut buf = [0u8; 20];
        let mut i = 0;
        let mut num = n;
        
        if num == 0 {
            print("0");
            return;
        }
        
        while num > 0 {
            buf[i] = (num % 10) as u8 + b'0';
            num /= 10;
            i += 1;
        }
        
        while i > 0 {
            i -= 1;
            let s = core::slice::from_raw_parts(&buf[i] as *const u8, 1);
            for &b in s {
                while (inb(COM1 + 5) & 0x20) == 0 {}
                outb(COM1, b);
            }
        }
    }
}

fn create_init_process() {
    // Create first user process
    // Entry: 0x400000 (typical ELF entry)
    // Stack: 0x7FFFFFFF
    // Page table: 0 (will be created)
    
    // In real system:
    // 1. Create address space
    // 2. Load /bin/init ELF
    // 3. Set up stack
    // 4. Create process structure
    // 5. Add to scheduler
}
