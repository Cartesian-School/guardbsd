// Process Management - Complete Implementation
// BSD 3-Clause License

#![no_std]

use crate::mm::AddressSpace;

pub type Pid = usize;

#[repr(C)]
pub struct Process {
    pub pid: Pid,
    pub entry: u64,
    pub stack_top: u64,
    pub state: ProcessState,
    pub kernel_stack: u64,
    pub page_table: usize,
    pub parent: Option<Pid>,
    pub regs: Registers,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Registers {
    pub rax: u64, pub rbx: u64, pub rcx: u64, pub rdx: u64,
    pub rsi: u64, pub rdi: u64, pub rbp: u64, pub rsp: u64,
    pub r8: u64, pub r9: u64, pub r10: u64, pub r11: u64,
    pub r12: u64, pub r13: u64, pub r14: u64, pub r15: u64,
    pub rip: u64, pub rflags: u64,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            rax: 0, rbx: 0, rcx: 0, rdx: 0,
            rsi: 0, rdi: 0, rbp: 0, rsp: 0,
            r8: 0, r9: 0, r10: 0, r11: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rip: 0, rflags: 0x202,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

static mut NEXT_PID: Pid = 1;
static mut CURRENT_PROCESS: Option<Pid> = None;
static mut PROCESS_TABLE: [Option<Process>; 64] = [None; 64];

pub fn allocate_pid() -> Pid {
    unsafe {
        let pid = NEXT_PID;
        NEXT_PID += 1;
        pid
    }
}

pub fn create_process(entry: u64, stack_top: u64, page_table: usize) -> Option<Pid> {
    let pid = allocate_pid();
    
    // Allocate kernel stack
    let kernel_stack = crate::mm::alloc_page()? as u64 + 4096;
    
    let mut regs = Registers::new();
    regs.rip = entry;
    regs.rsp = stack_top;
    
    let proc = Process {
        pid,
        entry,
        stack_top,
        state: ProcessState::Ready,
        kernel_stack,
        page_table,
        parent: unsafe { CURRENT_PROCESS },
        regs,
    };
    
    unsafe {
        for slot in PROCESS_TABLE.iter_mut() {
            if slot.is_none() {
                *slot = Some(proc);
                return Some(pid);
            }
        }
    }
    None
}

pub fn exec(pid: Pid, path: &str) -> bool {
    unsafe {
        for slot in PROCESS_TABLE.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    // Create new address space
                    let mut addr_space = match AddressSpace::new() {
                        Some(a) => a,
                        None => return false,
                    };
                    
                    // Load ELF
                    let elf = match crate::process::elf_loader::load_elf_from_file(path, &mut addr_space) {
                        Ok(e) => e,
                        Err(_) => return false,
                    };
                    
                    // Update process
                    proc.entry = elf.entry;
                    proc.regs.rip = elf.entry;
                    proc.state = ProcessState::Ready;
                    
                    return true;
                }
            }
        }
    }
    false
}

pub fn schedule() -> Option<Pid> {
    unsafe {
        for slot in PROCESS_TABLE.iter() {
            if let Some(proc) = slot {
                if proc.state == ProcessState::Ready {
                    return Some(proc.pid);
                }
            }
        }
    }
    None
}

pub fn switch_to(pid: Pid) {
    unsafe {
        CURRENT_PROCESS = Some(pid);
        for slot in PROCESS_TABLE.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    proc.state = ProcessState::Running;
                    // Activate page table
                    core::arch::asm!(
                        "mov cr3, {}",
                        in(reg) proc.page_table,
                        options(nostack, preserves_flags)
                    );
                    return;
                }
            }
        }
    }
}

pub fn get_current() -> Option<Pid> {
    unsafe { CURRENT_PROCESS }
}
