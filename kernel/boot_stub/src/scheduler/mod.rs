// Preemptive Scheduler
// BSD 3-Clause License

#[derive(Copy, Clone, PartialEq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Process {
    pub pid: usize,
    pub state: ProcessState,
    pub regs: Registers,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Registers {
    pub eax: u32, pub ebx: u32, pub ecx: u32, pub edx: u32,
    pub esi: u32, pub edi: u32, pub ebp: u32, pub esp: u32,
    pub eip: u32, pub eflags: u32,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            eax: 0, ebx: 0, ecx: 0, edx: 0,
            esi: 0, edi: 0, ebp: 0, esp: 0,
            eip: 0, eflags: 0x202,
        }
    }
}

static mut PROCESS_TABLE: [Option<Process>; 8] = [None; 8];
static mut CURRENT_PID: usize = 0;
static mut NEXT_PID: usize = 1;

pub fn init() {
    // Create idle process (PID 0)
    unsafe {
        PROCESS_TABLE[0] = Some(Process {
            pid: 0,
            state: ProcessState::Running,
            regs: Registers::new(),
        });
        CURRENT_PID = 0;
    }
}

pub fn create_process(entry: u32, stack: u32) -> Option<usize> {
    unsafe {
        let pid = NEXT_PID;
        NEXT_PID += 1;
        
        let mut regs = Registers::new();
        regs.eip = entry;
        regs.esp = stack;
        
        let proc = Process {
            pid,
            state: ProcessState::Ready,
            regs,
        };
        
        for slot in PROCESS_TABLE.iter_mut() {
            if slot.is_none() {
                *slot = Some(proc);
                return Some(pid);
            }
        }
    }
    None
}

pub fn schedule() -> Option<usize> {
    unsafe {
        let start = (CURRENT_PID + 1) % PROCESS_TABLE.len();
        
        for i in 0..PROCESS_TABLE.len() {
            let idx = (start + i) % PROCESS_TABLE.len();
            if let Some(proc) = &PROCESS_TABLE[idx] {
                if proc.state == ProcessState::Ready {
                    return Some(idx);
                }
            }
        }
        
        // Return current if no ready process
        Some(CURRENT_PID)
    }
}

pub fn switch_context(old_regs: &mut Registers, new_pid: usize) {
    unsafe {
        // Save old process state
        if let Some(proc) = &mut PROCESS_TABLE[CURRENT_PID] {
            proc.regs = *old_regs;
            if proc.state == ProcessState::Running {
                proc.state = ProcessState::Ready;
            }
        }
        
        // Load new process state
        if let Some(proc) = &mut PROCESS_TABLE[new_pid] {
            *old_regs = proc.regs;
            proc.state = ProcessState::Running;
            CURRENT_PID = new_pid;
        }
    }
}

pub fn get_current() -> usize {
    unsafe { CURRENT_PID }
}
