// Minimal proc stubs for ETAP 3.2 syscall wiring
#![no_std]

const ENOSYS: isize = -38;

pub enum TaskState {
    Ready,
    Running,
    Zombie,
}

#[repr(C)]
pub struct SavedRegs {
    pub rip: u64,
    pub rsp: u64,
    pub rflags: u64,
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

pub struct Task {
    pub pid: u32,
    pub state: TaskState,
    pub kstack_top: *mut u8,
    pub kstack_ptr: *mut u8,
    pub entry: u64,
    pub user_sp: u64,
    pub regs: SavedRegs,
}

const MAX_TASKS: usize = 16;
static mut TASKS: [Option<Task>; MAX_TASKS] = [None; MAX_TASKS];
static mut CURRENT_INDEX: usize = 0;
static mut NEXT_PID: u32 = 1;

impl SavedRegs {
    pub const fn zero() -> Self {
        SavedRegs {
            rip: 0,
            rsp: 0,
            rflags: 0,
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
        }
    }
}

pub fn alloc_pid() -> u32 {
    unsafe {
        let pid = if NEXT_PID == 0 { 1 } else { NEXT_PID };
        NEXT_PID = NEXT_PID.wrapping_add(1);
        pid
    }
}

pub fn find_task_by_pid(pid: u32) -> Option<&'static Task> {
    unsafe {
        for slot in TASKS.iter() {
            if let Some(t) = slot {
                if t.pid == pid {
                    return Some(t);
                }
            }
        }
        None
    }
}

pub fn find_task_by_pid_mut(pid: u32) -> Option<&'static mut Task> {
    unsafe {
        for slot in TASKS.iter_mut() {
            if let Some(t) = slot {
                if t.pid == pid {
                    return Some(t);
                }
            }
        }
        None
    }
}

pub fn current_pid() -> u32 {
    unsafe {
        if let Some(t) = TASKS.get(CURRENT_INDEX).and_then(|s| s.as_ref()) {
            t.pid
        } else {
            1
        }
    }
}

pub fn do_exec(_path: *const u8) -> isize {
    const MAX_PATH: usize = 128;
    let mut buf = [0u8; MAX_PATH];
    unsafe {
        if _path.is_null() {
            return -2; // ENOENT
        }
        let mut i = 0;
        while i < MAX_PATH {
            let c = core::ptr::read(_path.add(i));
            buf[i] = c;
            if c == 0 {
                break;
            }
            i += 1;
        }
        if i == MAX_PATH {
            return -2; // ENOENT
        }
    }

    if cstr_eq(&buf, "/bin/init") {
        return 0;
    }
    if cstr_eq(&buf, "/bin/gsh") {
        return 0;
    }

    -2 // ENOENT
}

pub fn do_getpid() -> isize {
    1
}

fn copy_cstr_from_user(ptr: *const u8, buf: &mut [u8]) -> usize {
    if ptr.is_null() || buf.is_empty() {
        buf[0] = 0;
        return 0;
    }
    let mut i = 0;
    let max = buf.len().saturating_sub(1);
    unsafe {
        while i < max {
            let b = core::ptr::read(ptr.add(i));
            if b == 0 {
                break;
            }
            buf[i] = b;
            i += 1;
        }
    }
    buf[i] = 0;
    i
}

fn cstr_eq(buf: &[u8], s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < buf.len() && buf[i] != 0 {
        if i >= bytes.len() || buf[i] != bytes[i] {
            return false;
        }
        i += 1;
    }
    i == bytes.len() && buf.get(i) == Some(&0)
}

/// Create (or update) a task from a kernel-owned path string.
/// This is a kernel-only helper, not a syscall entry.
pub fn create_task_from_kernel(path: &[u8]) -> isize {
    const INIT_PATH: &[u8] = b"/bin/init";
    const GSH_PATH: &[u8] = b"/bin/gsh";

    let known = (path == INIT_PATH) || (path == GSH_PATH);
    if !known {
        return -2; // ENOENT
    }

    unsafe {
        let pid = if let Some(t) = TASKS.get(CURRENT_INDEX).and_then(|s| s.as_ref()) {
            t.pid
        } else {
            alloc_pid()
        };

        let new_task = Task {
            pid,
            state: TaskState::Running,
            kstack_top: core::ptr::null_mut(),
            kstack_ptr: core::ptr::null_mut(),
            entry: 0,
            user_sp: 0,
            regs: SavedRegs::zero(),
        };

        if let Some(slot) = TASKS.get_mut(CURRENT_INDEX) {
            *slot = Some(new_task);
        }
        pid as isize
    }
}
