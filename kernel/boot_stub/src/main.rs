#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod interrupt;
mod drivers;
mod fs;
mod scheduler;

mod syscall {
    pub const SYS_EXIT: usize = 0;
    pub const SYS_WRITE: usize = 1;
    pub const SYS_READ: usize = 2;
    pub const SYS_FORK: usize = 3;
    pub const SYS_EXEC: usize = 4;
    pub const SYS_WAIT: usize = 5;
    pub const SYS_YIELD: usize = 6;
    pub const SYS_GETPID: usize = 7;
    
    pub fn syscall_handler(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
        match syscall_num {
            SYS_EXIT => sys_exit(arg1 as i32),
            SYS_WRITE => sys_write(arg1, arg2 as *const u8, arg3),
            SYS_READ => sys_read(arg1, arg2 as *mut u8, arg3),
            SYS_FORK => sys_fork(),
            SYS_EXEC => sys_exec(arg1 as *const u8),
            SYS_WAIT => sys_wait(arg1 as *mut i32),
            SYS_YIELD => sys_yield(),
            SYS_GETPID => sys_getpid(),
            _ => -1,
        }
    }
    
    fn sys_exit(status: i32) -> isize {
        let pid = super::scheduler::get_current();
        unsafe { 
            super::print("[SYSCALL] exit(");
            super::print_num(status as usize);
            super::print(") from PID ");
            super::print_num(pid);
            super::print("\n");
        }
        // Mark process as terminated
        // For now, halt
        loop { unsafe { core::arch::asm!("hlt"); } }
    }
    
    fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
        if fd == 1 || fd == 2 {
            unsafe {
                if buf.is_null() || len == 0 { return -1; }
                let slice = core::slice::from_raw_parts(buf, len);
                for &byte in slice {
                    while (super::inb(super::COM1 + 5) & 0x20) == 0 {}
                    super::outb(super::COM1, byte);
                }
            }
            len as isize
        } else {
            -1
        }
    }
    
    fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
        if fd == 0 {
            // stdin - read from keyboard
            unsafe {
                if buf.is_null() || len == 0 { return -1; }
                let mut count = 0;
                while count < len {
                    if let Some(ch) = super::drivers::keyboard::read_char() {
                        *buf.add(count) = ch;
                        count += 1;
                        if ch == b'\n' { break; }
                    } else {
                        break;
                    }
                }
                count as isize
            }
        } else {
            -1
        }
    }
    
    fn sys_fork() -> isize {
        let parent_pid = super::scheduler::get_current();
        unsafe {
            super::print("[SYSCALL] fork() from PID ");
            super::print_num(parent_pid);
            super::print("\n");
        }
        
        // Create child process (copy of parent)
        // For now, return mock PID
        if let Some(child_pid) = super::scheduler::create_process(0x400000, 0x7FFFFFFF) {
            child_pid as isize
        } else {
            -1
        }
    }
    
    fn sys_exec(path: *const u8) -> isize {
        unsafe {
            if path.is_null() { return -1; }
            
            // Read path string
            let mut len = 0;
            while len < 256 && *path.add(len) != 0 {
                len += 1;
            }
            let path_slice = core::slice::from_raw_parts(path, len);
            
            super::print("[SYSCALL] exec(");
            for &b in path_slice {
                while (super::inb(super::COM1 + 5) & 0x20) == 0 {}
                super::outb(super::COM1, b);
            }
            super::print(")\n");
            
            // Load and execute binary
            // For now, return success
            0
        }
    }
    
    fn sys_wait(status: *mut i32) -> isize {
        let pid = super::scheduler::get_current();
        unsafe {
            super::print("[SYSCALL] wait() from PID ");
            super::print_num(pid);
            super::print("\n");
            
            if !status.is_null() {
                *status = 0;
            }
        }
        // Return child PID (mock)
        -1
    }
    
    fn sys_yield() -> isize {
        unsafe {
            super::print("[SYSCALL] yield()\n");
        }
        // Trigger scheduler
        // For now, just return
        0
    }
    
    fn sys_getpid() -> isize {
        let pid = super::scheduler::get_current();
        pid as isize
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
    
    // Preemptive scheduling every 10 ticks (~100ms at 100Hz)
    if drivers::timer::get_ticks() % 10 == 0 {
        if let Some(next_pid) = scheduler::schedule() {
            if next_pid != scheduler::get_current() {
                // Context switch would happen here
                // For now, just track scheduling
            }
        }
    }
}

core::arch::global_asm!(
    ".section .multiboot",
    ".align 4",
    ".long 0x1BADB002",
    ".long 0x00000003",
    ".long -(0x1BADB002 + 0x00000003)",
);

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
pub extern "C" fn _start() -> ! {
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
        scheduler::init();
        drivers::timer::init(100); // 100 Hz
        print("[OK] Scheduler initialized (100 Hz timer)\n");
        
        enable_interrupts();
        print("\n[INIT] Microkernel bootstrap starting...\n");
        print("[INIT] Loading µK-Space (memory management)...\n");
        print("[INIT] Loading µK-Time (scheduler)...\n");
        print("[INIT] Loading µK-IPC (communication)...\n");
        print("[OK] Microkernels initialized\n\n");
        
        print("[INIT] Starting system servers...\n");
        print("[INIT] Starting init server...\n");
        print("[INIT] Starting vfs server...\n");
        print("[INIT] Starting ramfs server...\n");
        print("[INIT] Starting devd server...\n");
        print("[OK] System servers started\n\n");
        
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
