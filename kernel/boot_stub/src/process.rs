//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Zarządzanie procesami i ładowanie ELF w boot stubie GuardBSD.

// ELF Structures (32-bit for compatibility)
#[repr(C)]
pub struct Elf32_Ehdr {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u32,
    pub e_phoff: u32,
    pub e_shoff: u32,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C)]
pub struct Elf32_Phdr {
    pub p_type: u32,
    pub p_offset: u32,
    pub p_vaddr: u32,
    pub p_paddr: u32,
    pub p_filesz: u32,
    pub p_memsz: u32,
    pub p_flags: u32,
    pub p_align: u32,
}

// ELF Constants
pub const ET_EXEC: u16 = 2;
pub const PT_LOAD: u32 = 1;
pub const PF_X: u32 = 1;
pub const PF_W: u32 = 2;
pub const PF_R: u32 = 4;

// CPU Context for process switching
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuContext {
    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp: u32,
    pub ebx: u32,
    pub edx: u32,
    pub ecx: u32,
    pub eax: u32,
    pub eip: u32,
    pub eflags: u32,
    pub cr3: u32,
}

pub mod process {
    pub fn switch_to(_pid: usize) {}
    pub fn get_current() -> Option<usize> {
        Some(1)
    }
    pub fn try_add_memory_usage(_pid: usize, _pages: usize) -> bool {
        true
    }
    pub fn mark_killed(_pid: usize) {}
    pub fn create_process(_entry: u64, _stack: u64, _pml4: u64) -> usize {
        1
    }
}

pub mod elf_loader {
    pub fn parse_and_load_elf<'a>(
        _bytes: &'a [u8],
        _aspace: &mut crate::kernel::mm::AddressSpace,
    ) -> Result<LoadedElf<'a>, ()> {
        Ok(LoadedElf {
            entry: 0,
            segments: &[],
        })
    }

    pub struct LoadedElf<'a> {
        pub entry: u64,
        pub segments: &'a [u8],
    }
}

impl CpuContext {
    pub const fn new() -> Self {
        CpuContext {
            edi: 0, esi: 0, ebp: 0, esp: 0,
            ebx: 0, edx: 0, ecx: 0, eax: 0,
            eip: 0, eflags: 0, cr3: 0,
        }
    }
}

// Note: The canonical Process structure is now defined in kernel/process/types.rs
// This boot stub uses a simplified internal structure for boot-time microkernel loading only.
// TODO: Eventually refactor to use the canonical Process structure throughout.

// Internal boot-time process tracking structure (32-bit, temporary)
#[derive(Debug, Clone, Copy)]
struct BootProcess {
    pub pid: usize,
    pub entry_point: u32,
    pub stack_top: u32,
    pub stack_bottom: u32,
    pub page_table: u32,
    pub state: BootProcessState,
    pub context: CpuContext,
    pub kernel_stack: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum BootProcessState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

// Process manager (boot stub internal only)
pub struct ProcessManager {
    processes: [Option<BootProcess>; 32],
    next_pid: usize,
}

impl ProcessManager {
    pub const fn new() -> Self {
        ProcessManager {
            processes: [None; 32],
            next_pid: 1, // PID 0 reserved for idle
        }
    }

    pub fn create_process(&mut self, entry: u32, stack_top: u32) -> Option<usize> {
        if self.next_pid >= 32 {
            return None;
        }

        let pid = self.next_pid;
        self.next_pid += 1;

        // Allocate kernel stack (4KB)
        let kernel_stack = allocate_kernel_stack();

        let mut context = CpuContext::new();
        context.eip = entry;
        context.esp = stack_top;
        context.cr3 = get_current_page_table();

        let process = BootProcess {
            pid,
            entry_point: entry,
            stack_top,
            stack_bottom: stack_top - 0x1000, // 4KB stack
            page_table: get_current_page_table(),
            state: BootProcessState::Ready,
            context,
            kernel_stack,
        };

        self.processes[pid] = Some(process);
        Some(pid)
    }

    pub fn get_process(&self, pid: usize) -> Option<&BootProcess> {
        if pid < 32 {
            self.processes[pid].as_ref()
        } else {
            None
        }
    }

    pub fn get_process_mut(&mut self, pid: usize) -> Option<&mut BootProcess> {
        if pid < 32 {
            self.processes[pid].as_mut()
        } else {
            None
        }
    }
}

// ELF Loader
pub struct ElfLoader;

impl ElfLoader {
    pub fn validate_header(ehdr: &Elf32_Ehdr) -> bool {
        // Check ELF magic
        if ehdr.e_ident[0] != 0x7F || ehdr.e_ident[1] != b'E' ||
           ehdr.e_ident[2] != b'L' || ehdr.e_ident[3] != b'F' {
            return false;
        }

        // Check 32-bit, little-endian, executable
        if ehdr.e_ident[4] != 1 || ehdr.e_ident[5] != 1 || ehdr.e_type != ET_EXEC {
            return false;
        }

        true
    }

    pub fn load_program_headers<'a>(
        ehdr: &Elf32_Ehdr,
        file_data: &'a [u8],
    ) -> Option<u32> {
        let phdr_start = ehdr.e_phoff as usize;
        let phdr_size = ehdr.e_phentsize as usize;
        let phdr_count = ehdr.e_phnum as usize;

        for i in 0..phdr_count {
            let phdr_offset = phdr_start + i * phdr_size;
            if phdr_offset + core::mem::size_of::<Elf32_Phdr>() > file_data.len() {
                return None;
            }

            // Safety: We've checked bounds
            let phdr = unsafe {
                &*(file_data.as_ptr().add(phdr_offset) as *const Elf32_Phdr)
            };

            if phdr.p_type == PT_LOAD {
                if !Self::load_segment(phdr, file_data) {
                    return None;
                }
            }
        }

        Some(ehdr.e_entry)
    }

    fn load_segment(phdr: &Elf32_Phdr, file_data: &[u8]) -> bool {
        let vaddr = phdr.p_vaddr as usize;
        let offset = phdr.p_offset as usize;
        let file_size = phdr.p_filesz as usize;
        let mem_size = phdr.p_memsz as usize;

        // Check bounds
        if offset + file_size > file_data.len() {
            return false;
        }

        // Copy file data to virtual address
        // In a real implementation, this would handle page allocation
        // For now, assume identity mapping
        unsafe {
            let src = file_data.as_ptr().add(offset);
            let dst = vaddr as *mut u8;

            // Copy file data
            for i in 0..file_size {
                *dst.add(i) = *src.add(i);
            }

            // Zero BSS
            for i in file_size..mem_size {
                *dst.add(i) = 0;
            }
        }

        true
    }
}

// Stack allocation (simplified)
static mut NEXT_STACK_ADDR: u32 = 0x800000; // Start allocating stacks from 8MB

fn allocate_kernel_stack() -> u32 {
    unsafe {
        let stack_addr = NEXT_STACK_ADDR;
        NEXT_STACK_ADDR += 0x1000; // 4KB stacks
        stack_addr + 0x1000 // Return top of stack
    }
}

fn get_current_page_table() -> u32 {
    // Get CR3 register value
    let cr3: u32;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) cr3);
    }
    cr3
}

// Global process manager
pub static mut PROCESS_MANAGER: ProcessManager = ProcessManager::new();

pub fn init_process_manager() {
    unsafe {
        PROCESS_MANAGER = ProcessManager::new();
    }
}

pub fn load_microkernel(_name: &str, path: &str) -> Option<usize> {
    // Load ELF file from filesystem
    if let Some(file_data) = crate::fs::iso9660::read_file(path) {
        // Parse ELF header
        if file_data.len() < core::mem::size_of::<Elf32_Ehdr>() {
            return None;
        }

        // Safety: We've checked the size
        let ehdr = unsafe {
            &*(file_data.as_ptr() as *const Elf32_Ehdr)
        };

        // Validate ELF header
        if !ElfLoader::validate_header(ehdr) {
            return None;
        }

        // Load program segments
        if let Some(entry_point) = ElfLoader::load_program_headers(ehdr, file_data) {
            // Create process with loaded entry point
            unsafe {
                PROCESS_MANAGER.create_process(entry_point, 0x7FFFFFFF)
            }
        } else {
            None
        }
    } else {
        None
    }
}

pub fn start_microkernel(pid: usize) -> bool {
    unsafe {
        if let Some(process) = PROCESS_MANAGER.get_process_mut(pid) {
            process.state = BootProcessState::Running;

            // For microkernels, we need to enter long mode first
            if is_64bit_microkernel(pid) {
                enter_long_mode_for_microkernel(&process.context);
            } else {
                // Switch to process context in protected mode
                switch_to_process(&process.context);
            }
            true
        } else {
            false
        }
    }
}

pub fn load_server(_name: &str, path: &str) -> Option<usize> {
    // Load server ELF file from filesystem (same as microkernels)
    if let Some(file_data) = crate::fs::iso9660::read_file(path) {
        // Parse ELF header
        if file_data.len() < core::mem::size_of::<Elf32_Ehdr>() {
            return None;
        }

        // Safety: We've checked the size
        let ehdr = unsafe {
            &*(file_data.as_ptr() as *const Elf32_Ehdr)
        };

        // Validate ELF header
        if !ElfLoader::validate_header(ehdr) {
            return None;
        }

        // Load program segments
        if let Some(entry_point) = ElfLoader::load_program_headers(ehdr, file_data) {
            // Create process with loaded entry point
            unsafe {
                PROCESS_MANAGER.create_process(entry_point, 0x7FFFFFFF)
            }
        } else {
            None
        }
    } else {
        None
    }
}

pub fn start_server(pid: usize) -> bool {
    unsafe {
        if let Some(process) = PROCESS_MANAGER.get_process_mut(pid) {
            process.state = BootProcessState::Running;

            // Servers run in 64-bit mode like microkernels
            if is_64bit_server(pid) {
                enter_long_mode_for_microkernel(&process.context);
            } else {
                // Switch to process context in protected mode
                switch_to_process(&process.context);
            }
            true
        } else {
            false
        }
    }
}

fn is_64bit_server(pid: usize) -> bool {
    // For now, assume all servers are 64-bit like microkernels
    // In practice, we'd check the ELF header
    pid >= 4 && pid <= 7 // Server PIDs after microkernels
}

fn is_64bit_microkernel(pid: usize) -> bool {
    // For now, assume all microkernels are 64-bit
    // In practice, we'd check the ELF header
    pid >= 1 && pid <= 3 // PIDs 1-3 are microkernels
}

fn enter_long_mode_for_microkernel(_context: &CpuContext) {
    // TODO: Implement proper long mode transition
    // For now, microkernels will run in compatibility mode or not at all
    // This is a complex implementation that requires proper 64-bit page tables,
    // GDT setup, and mode switching

    // For the current implementation, we'll skip long mode transition
    // and let microkernels fail to start (which is better than crashing)
}

pub fn switch_to_process(context: &CpuContext) {
    unsafe {
        // Load new page table
        core::arch::asm!("mov cr3, {}", in(reg) context.cr3);

        // Restore registers and jump to process
        // This is a simplified context switch - in reality we'd save current context first
        core::arch::asm!(
            "mov esp, {esp}",
            "push {eflags}",
            "push {cs}",
            "push {eip}",
            "mov edi, {edi}",
            "mov esi, {esi}",
            "mov ebp, {ebp}",
            "mov ebx, {ebx}",
            "mov edx, {edx}",
            "mov ecx, {ecx}",
            "mov eax, {eax}",
            "iret",
            esp = in(reg) context.esp,
            eflags = in(reg) context.eflags,
            cs = const 0x08, // Kernel code segment
            eip = in(reg) context.eip,
            edi = in(reg) context.edi,
            esi = in(reg) context.esi,
            ebp = in(reg) context.ebp,
            ebx = in(reg) context.ebx,
            edx = in(reg) context.edx,
            ecx = in(reg) context.ecx,
            eax = in(reg) context.eax,
        );
    }
}
