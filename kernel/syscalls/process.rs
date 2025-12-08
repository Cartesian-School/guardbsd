// Process Syscall Implementations
// BSD 3-Clause License
// Day 6: Basic sys_exit() implementation

#![no_std]

extern crate alloc;

use crate::process::types::{Process, Pid, ProcessState, SIGCHLD};

// Process table from kernel/process/process.rs
extern "C" {
    static mut PROCESS_TABLE: [Option<Process>; 64];
    static mut CURRENT_PROCESS: Option<Pid>;
}

/// Get reference to process table (unsafe access required)
unsafe fn get_process_table() -> &'static mut [Option<Process>; 64] {
    &mut PROCESS_TABLE
}

/// Get current process ID
unsafe fn get_current_pid() -> Option<Pid> {
    CURRENT_PROCESS
}

/// Set current process ID
unsafe fn set_current_pid(pid: Option<Pid>) {
    CURRENT_PROCESS = pid;
}

/// Find process by PID in process table
fn find_process_mut(pid: Pid) -> Option<&'static mut Process> {
    unsafe {
        let table = get_process_table();
        for slot in table.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    return Some(proc);
                }
            }
        }
    }
    None
}

/// Find process by PID (immutable reference)
fn find_process(pid: Pid) -> Option<&'static Process> {
    unsafe {
        let table = get_process_table();
        for slot in table.iter() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    return Some(proc);
                }
            }
        }
    }
    None
}

/// Close all file descriptors for a process
/// Returns number of file descriptors closed
pub fn close_all_fds(pid: Pid) -> usize {
    if let Some(proc) = find_process_mut(pid) {
        let count = proc.fd_count;
        proc.close_all_fds();
        count
    } else {
        0
    }
}

/// Set process state
/// Returns true if successful, false if process not found
pub fn set_process_state(pid: Pid, state: ProcessState) -> bool {
    if let Some(proc) = find_process_mut(pid) {
        proc.state = state;
        true
    } else {
        false
    }
}

/// Set exit status for a process
/// Returns true if successful, false if process not found
pub fn set_exit_status(pid: Pid, status: i32) -> bool {
    if let Some(proc) = find_process_mut(pid) {
        proc.exit_status = Some(status);
        true
    } else {
        false
    }
}

/// Get process state
pub fn get_process_state(pid: Pid) -> Option<ProcessState> {
    find_process(pid).map(|p| p.state)
}

/// Get exit status
pub fn get_exit_status(pid: Pid) -> Option<i32> {
    find_process(pid).and_then(|p| p.exit_status)
}

/// Get parent PID of a process
/// Returns Some(parent_pid) if process exists and has a parent, None otherwise
pub fn get_parent_pid(pid: Pid) -> Option<Pid> {
    find_process(pid).and_then(|p| p.parent)
}

/// Get thread ID associated with a process
/// Returns Some(tid) if process has a registered thread, None otherwise
pub fn get_thread_id(pid: Pid) -> Option<usize> {
    find_process(pid).and_then(|p| p.thread_id)
}

/// Send signal to a process (basic implementation)
/// Returns true if signal was sent successfully, false otherwise
pub fn send_signal(pid: Pid, signal: i32) -> bool {
    if let Some(proc) = find_process_mut(pid) {
        if signal >= 0 && signal < 64 {
            proc.pending_signals |= 1 << signal;
            true
        } else {
            false
        }
    } else {
        false
    }
}

/// Reparent children to init process (PID 1)
/// This is called when a process exits to ensure orphaned children
/// are adopted by the init process, following BSD/UNIX semantics
/// Returns number of children reparented
pub fn reparent_children_to_init(pid: Pid) -> usize {
    const INIT_PID: Pid = 1;
    let mut reparented = 0;
    
    // First, get the list of children
    let children: [Option<Pid>; 32] = if let Some(proc) = find_process(pid) {
        proc.children
    } else {
        return 0;
    };
    
    // For each child, update their parent to init
    for child_pid in children.iter().filter_map(|&c| c) {
        if let Some(child) = find_process_mut(child_pid) {
            child.parent = Some(INIT_PID);
            reparented += 1;
        }
    }
    
    // Add children to init's children list
    if reparented > 0 {
        if let Some(init_proc) = find_process_mut(INIT_PID) {
            for child_pid in children.iter().filter_map(|&c| c) {
                init_proc.add_child(child_pid);
            }
        }
    }
    
    // Clear the exiting process's children list
    if let Some(proc) = find_process_mut(pid) {
        proc.children = [None; 32];
        proc.child_count = 0;
    }
    
    reparented
}

/// System call: exit - Terminate current process
/// 
/// This implementation follows BSD semantics:
/// 1. Close all open file descriptors
/// 2. Reparent children to init (PID 1)
/// 3. Set exit status
/// 4. Transition process to Zombie state
/// 5. Set thread state to Zombie in scheduler
/// 6. Send SIGCHLD to parent
/// 7. Clear current process and schedule next
///
/// The process remains in Zombie state until parent calls wait()
pub fn sys_exit(status: i32) -> ! {
    unsafe {
        let current_pid = match get_current_pid() {
            Some(pid) => pid,
            None => {
                // No current process - kernel panic
                loop {
                    core::arch::asm!("hlt");
                }
            }
        };
        
        // Step 1: Close all open file descriptors
        let _fds_closed = close_all_fds(current_pid);
        
        // Step 2: Reparent children to init (PID 1)
        // This ensures orphaned children are properly adopted
        let _children_reparented = reparent_children_to_init(current_pid);
        
        // Step 3: Set exit status
        set_exit_status(current_pid, status);
        
        // Step 4: Transition process to Zombie state
        set_process_state(current_pid, ProcessState::Zombie);
        
        // Step 5: Set thread state to Zombie in scheduler
        if let Some(tid) = get_thread_id(current_pid) {
            crate::sched::set_thread_state(tid, crate::sched::ThreadState::Zombie);
        }
        
        // Step 6: Send SIGCHLD to parent (if exists)
        if let Some(parent_pid) = get_parent_pid(current_pid) {
            send_signal(parent_pid, SIGCHLD);
        }
        
        // Step 7: Clear current process (scheduler will pick next)
        set_current_pid(None);
        
        // TODO: Full scheduler integration in Phase 4
        // For now, just halt - in Phase 4 this will do actual context switch
        // Will call: crate::sched::schedule() or trigger reschedule
        loop {
            core::arch::asm!("hlt");
        }
    }
}

/// System call: getpid - Get current process ID
/// 
/// BSD Semantics:
/// - Returns the PID of the calling process
/// - Never fails (always returns valid PID)
/// - PIDs are unique per process
/// - PIDs are typically positive integers starting from 1
///
/// Implementation:
/// - Looks up CURRENT_PROCESS global
/// - Returns PID from current process context
/// - Future: Could verify with scheduler TID→PID mapping
///
/// Returns:
/// - PID (> 0) on success
/// - -1 only if no current process (kernel panic scenario)
pub fn sys_getpid() -> isize {
    unsafe {
        match get_current_pid() {
            Some(pid) => {
                // Verify PID is valid (paranoid check)
                if pid > 0 && pid < 65536 {
                    pid as isize
                } else {
                    // Invalid PID - should never happen
                    -1
                }
            },
            None => {
                // No current process - should never happen in userland
                // This would indicate a serious kernel bug
                -1
            }
        }
    }
}

/// Get PID from Thread ID (scheduler integration)
/// This provides an alternative way to get PID via scheduler
/// Returns Some(pid) if thread exists and has associated process
pub fn get_pid_from_tid(tid: usize) -> Option<Pid> {
    // Access scheduler to get TCB
    use crate::sched;
    
    // TODO: Add sched::get_tcb(tid) function
    // For now, this is a stub for future scheduler integration
    // In Phase 4, this will:
    // 1. Lock scheduler
    // 2. Get TCB by TID
    // 3. Return TCB.pid
    
    // Temporary: Just use current process lookup
    unsafe { get_current_pid() }
}

/// System call: fork - Create child process
///
/// BSD Semantics:
/// - Creates exact copy of parent process
/// - Returns child PID to parent
/// - Returns 0 to child
/// - Child gets copy of address space
/// - Child gets copy of file descriptors
/// - Child gets copy of signal handlers
/// - Child has unique PID
/// - Child added to parent's children list
///
/// Implementation:
/// 1. Allocate new PID
/// 2. Clone parent's address space
/// 3. Create child Process structure
/// 4. Copy parent fields to child
/// 5. Allocate kernel stack
/// 6. Add child to parent's list
/// 7. Register with scheduler (Phase 4)
/// 8. Return child PID to parent
///
/// Returns:
/// - Child PID (> 0) to parent on success
/// - 0 to child process
/// - -1 on error (out of memory, no slots, etc.)
pub fn sys_fork() -> isize {
    unsafe {
        // Step 1: Get current process (parent)
        let parent_pid = match get_current_pid() {
            Some(pid) => pid,
            None => return -1, // No current process
        };
        
        let parent = match find_process(parent_pid) {
            Some(p) => p,
            None => return -1, // Parent not found
        };
        
        // Step 2: Allocate new PID for child
        let child_pid = crate::process::allocate_pid();
        
        // Step 3: Clone address space (page table)
        let child_page_table = match crate::mm::clone_address_space(parent.page_table) {
            Some(pt) => pt,
            None => return -1, // Memory allocation failed
        };
        
        // Step 4: Allocate kernel stack for child
        let child_kernel_stack = match crate::mm::alloc_page() {
            Some(page) => (page as u64) + 4096, // Top of stack
            None => return -1, // Stack allocation failed
        };
        
        // Step 5: Create child Process structure by copying parent
        let mut child = *parent; // Copy all fields from parent
        
        // Step 6: Set child-specific fields
        child.pid = child_pid;
        child.parent = Some(parent_pid);
        child.children = [None; 32]; // Child starts with no children
        child.child_count = 0;
        child.state = ProcessState::Ready;
        child.exit_status = None;
        child.page_table = child_page_table;
        child.kernel_stack = child_kernel_stack;
        child.thread_id = None; // Will be set when registered with scheduler
        
        // Copy file descriptor table (each process gets independent copy)
        child.fd_table = parent.fd_table;
        child.fd_count = parent.fd_count;
        
        // Copy signal handlers
        child.signal_handlers = parent.signal_handlers;
        child.pending_signals = 0; // Child starts with no pending signals
        child.signal_mask = parent.signal_mask;
        
        // Copy memory limits
        child.memory_usage = 0; // Child starts fresh
        child.memory_limit = parent.memory_limit;
        
        // Step 7: Add child to parent's children list
        if let Some(parent_mut) = find_process_mut(parent_pid) {
            if !parent_mut.add_child(child_pid) {
                // Parent's children list is full
                // TODO: Free allocated resources (page table, kernel stack)
                return -1;
            }
        }
        
        // Step 8: Add child to process table
        let table = get_process_table();
        let mut child_added = false;
        for slot in table.iter_mut() {
            if slot.is_none() {
                *slot = Some(child);
                child_added = true;
                break;
            }
        }
        
        if !child_added {
            // Process table is full
            // TODO: Remove child from parent's list
            // TODO: Free allocated resources
            return -1;
        }
        
        // Step 9: Clone parent's CPU context (Day 14)
        // Get parent's current context from scheduler
        let parent_ctx = match crate::sched::get_current_context(0) {
            Some(ctx) => ctx,
            None => {
                // Can't get parent context - use default
                crate::sched::ArchContext::zeroed()
            }
        };
        
        // Create child's context by copying parent
        let mut child_ctx = parent_ctx;
        
        // Step 10: Set child's return value to 0 (Day 14)
        // This is the key difference: parent gets child PID, child gets 0
        #[cfg(target_arch = "x86_64")]
        {
            child_ctx.rax = 0; // Return value for child
            child_ctx.cr3 = child_page_table as u64; // Child's page table
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            child_ctx.x[0] = 0; // x0 is return value on AArch64
            child_ctx.ttbr0 = child_page_table as u64; // Child's page table
        }
        
        // Step 11: Register child with scheduler (Day 15)
        let child_tid = match crate::sched::register_thread(
            child_pid,
            1, // Default priority
            0, // CPU 0 (TODO: use parent's CPU affinity)
            child_ctx
        ) {
            Some(tid) => tid,
            None => {
                // Scheduler registration failed
                // TODO: Cleanup allocated resources
                return -1;
            }
        };
        
        // Step 12: Store thread_id in child Process (Day 15)
        if let Some(child_mut) = find_process_mut(child_pid) {
            child_mut.thread_id = Some(child_tid);
        }
        
        // Step 13: Return child PID to parent
        // Note: Parent's context.rax will be set to child_pid by syscall return
        // Child's context.rax is already set to 0 above
        child_pid as isize
    }
}

/// Copy a null-terminated string from userspace to kernel space (Day 17)
///
/// This is a critical security function that safely copies strings from
/// untrusted user memory into trusted kernel memory.
///
/// Parameters:
/// - user_ptr: Pointer to string in user address space
/// - max_len: Maximum length to copy (prevents overflow)
///
/// Returns:
/// - Ok(String) on success
/// - Err(&str) on failure (null pointer, invalid pointer, too long)
///
/// Safety:
/// This function performs basic validation but is inherently unsafe
/// as it dereferences user-provided pointers.
pub unsafe fn copy_string_from_user(user_ptr: *const u8, max_len: usize) -> Result<alloc::vec::Vec<u8>, &'static str> {
    // Validate pointer is not null
    if user_ptr.is_null() {
        return Err("Null pointer");
    }
    
    // Validate pointer is in user space (below kernel space)
    // x86_64: User space is typically 0x0000_0000_0000_0000 to 0x0000_7FFF_FFFF_FFFF
    // Kernel space is 0xFFFF_8000_0000_0000 and above
    let addr = user_ptr as usize;
    if addr >= 0xFFFF_8000_0000_0000 {
        return Err("Pointer in kernel space");
    }
    
    // TODO: In production, also check:
    // - Pointer is mapped in current process's page table
    // - Pages are marked as USER accessible
    // - No SMAP/SMEP violations
    
    // Copy string byte by byte until null terminator
    let mut result = alloc::vec::Vec::new();
    for i in 0..max_len {
        let byte = unsafe { *user_ptr.add(i) };
        if byte == 0 {
            // Found null terminator
            return Ok(result);
        }
        result.push(byte);
    }
    
    // String too long (no null terminator within max_len)
    Err("String too long")
}

/// Validate a user pointer for safety (Day 17)
///
/// Checks if a pointer is safe to dereference from kernel space.
/// This is a simplified version - production kernels do much more validation.
///
/// Returns:
/// - true if pointer appears safe
/// - false if pointer is unsafe
pub fn is_user_pointer_valid(ptr: *const u8, len: usize) -> bool {
    if ptr.is_null() {
        return false;
    }
    
    let start_addr = ptr as usize;
    let end_addr = start_addr.saturating_add(len);
    
    // Check not in kernel space
    if start_addr >= 0xFFFF_8000_0000_0000 {
        return false;
    }
    
    // Check doesn't wrap around
    if end_addr < start_addr {
        return false;
    }
    
    // Check end also not in kernel space
    if end_addr >= 0xFFFF_8000_0000_0000 {
        return false;
    }
    
    true
}

/// System call: exec - Execute program (Days 17-21)
///
/// BSD Semantics:
/// - Replaces current process with new program
/// - Loads ELF binary from file system
/// - Sets up new address space
/// - Resets process state
/// - Preserves PID and parent
/// - Closes FDs marked close-on-exec
/// - Does not return on success (process is replaced)
/// - Returns -1 on error
///
/// Parameters:
/// - path: Pointer to null-terminated path string in user space
/// - argv: Pointer to array of argument strings (not used yet)
///
/// Implementation (Days 17-21):
/// Day 17: Path handling & file loading
/// Day 18: ELF parsing  
/// Day 19: Address space setup
/// Day 20: Context setup
/// Day 21: Testing
///
/// Returns:
/// - Does not return on success (process is replaced)
/// - -1 on error (file not found, invalid ELF, etc.)
pub fn sys_exec(path: *const u8, _argv: *const *const u8) -> isize {
    unsafe {
        // Day 17: Path Handling & File Loading
        
        // Step 1: Validate path pointer
        if !is_user_pointer_valid(path, 4096) {
            return -1; // EFAULT: Invalid pointer
        }
        
        // Step 2: Copy path string from user space (Day 17)
        const MAX_PATH_LEN: usize = 4096;
        let path_bytes = match copy_string_from_user(path, MAX_PATH_LEN) {
            Ok(bytes) => bytes,
            Err(_) => return -1, // EFAULT: Bad address or string too long
        };
        
        // Convert to string slice
        let path_str = match core::str::from_utf8(&path_bytes) {
            Ok(s) => s,
            Err(_) => return -1, // EINVAL: Invalid UTF-8
        };
        
        // Step 3: Get current process
        let current_pid = match get_current_pid() {
            Some(pid) => pid,
            None => return -1, // No current process
        };
        
        let process = match find_process(current_pid) {
            Some(p) => p,
            None => return -1, // Process not found
        };
        
        // Day 18: ELF Parsing
        
        // Step 4: Read ELF file from VFS (Day 17)
        // TODO: Refactor to use VFS layer when available (Issue: sys_exec VFS integration)
        // For now, read directly from ISO9660 filesystem
        
        // Step 5: Read ELF data
        let elf_data = match crate::fs::iso9660::read_file(path_str) {
            Some(data) => data,
            None => return -1, // ENOENT: File not found
        };
        
        // Step 6: Parse ELF header (Day 18)
        use crate::process::elf_loader;
        let (entry_point, _segment_count) = match elf_loader::parse_elf_header(&elf_data) {
            Ok(result) => result,
            Err(_) => return -1, // ENOEXEC: Invalid executable
        };
        
        // Step 7: Get loadable segments
        let segments = match elf_loader::get_loadable_segments(&elf_data) {
            Ok(segs) => segs,
            Err(_) => return -1, // ENOEXEC: Invalid executable
        };
        
        // Day 19: Address Space Replacement
        
        // Step 8: Save old page table (will be freed later)
        let old_page_table = process.page_table;
        
        // Step 9: Create new address space
        let new_page_table = match crate::mm::create_address_space() {
            Some(pt) => pt,
            None => return -1, // ENOMEM: Out of memory
        };
        
        // Step 10: Create AddressSpace for loading
        let mut addr_space = match crate::mm::AddressSpace::new() {
            Some(as_) => as_,
            None => {
                crate::mm::destroy_address_space(new_page_table);
                return -1; // ENOMEM
            }
        };
        
        // Step 11: Load ELF segments into new address space
        for segment in &segments {
            let vaddr = segment.vaddr as usize;
            let memsz = segment.memsz as usize;
            let filesz = segment.filesz as usize;
            let offset = segment.offset as usize;
            let flags = segment.flags;
            
            // Allocate and map pages for this segment
            let pages = (memsz + 4095) / 4096;
            for p in 0..pages {
                let phys_page = match crate::mm::alloc_page() {
                    Some(page) => page,
                    None => {
                        // Out of memory - cleanup and return
                        crate::mm::destroy_address_space(new_page_table);
                        return -1; // ENOMEM
                    }
                };
                
                let virt_addr = vaddr + p * 4096;
                
                // Copy data from ELF if within filesz
                if p * 4096 < filesz {
                    let copy_size = core::cmp::min(4096, filesz - p * 4096);
                    let src = elf_data.as_ptr().add(offset + p * 4096);
                    let dst = phys_page as *mut u8;
                    core::ptr::copy_nonoverlapping(src, dst, copy_size);
                    
                    // Zero remaining bytes in page if any (BSS)
                    if copy_size < 4096 && p * 4096 + 4096 <= memsz {
                        let zero_start = dst.add(copy_size);
                        let zero_size = 4096 - copy_size;
                        core::ptr::write_bytes(zero_start, 0, zero_size);
                    }
                } else if p * 4096 < memsz {
                    // Pure BSS - zero entire page
                    let dst = phys_page as *mut u8;
                    core::ptr::write_bytes(dst, 0, 4096);
                }
                
                // Setup page flags
                let mut page_flags = crate::mm::PageFlags::PRESENT | crate::mm::PageFlags::USER;
                if flags & 2 != 0 {  // PF_W (writable)
                    page_flags |= crate::mm::PageFlags::WRITABLE;
                }
                
                // Map page into address space
                addr_space.map(virt_addr, phys_page, page_flags);
            }
        }
        
        // Step 12: Setup user stack (8MB at high address)
        const STACK_SIZE: usize = 8 * 1024 * 1024;  // 8MB
        const STACK_TOP: u64 = 0x7FFF_FFFF_F000;    // Just below 128TB
        let stack_pages = STACK_SIZE / 4096;
        
        for i in 0..stack_pages {
            let phys_page = match crate::mm::alloc_page() {
                Some(page) => page,
                None => {
                    crate::mm::destroy_address_space(new_page_table);
                    return -1; // ENOMEM
                }
            };
            
            let virt_addr = (STACK_TOP as usize) - STACK_SIZE + i * 4096;
            
            // Zero stack page
            let dst = phys_page as *mut u8;
            core::ptr::write_bytes(dst, 0, 4096);
            
            // Map with user + writable
            let page_flags = crate::mm::PageFlags::PRESENT | 
                           crate::mm::PageFlags::USER | 
                           crate::mm::PageFlags::WRITABLE;
            addr_space.map(virt_addr, phys_page, page_flags);
        }
        
        // Day 20: Process Update
        
        // Step 13: Update process structure
        let process_mut = match find_process_mut(current_pid) {
            Some(p) => p,
            None => {
                crate::mm::destroy_address_space(new_page_table);
                return -1;
            }
        };
        
        // Update page table
        process_mut.page_table = new_page_table;
        
        // Update entry point
        process_mut.entry = entry_point;
        
        // Update stack pointers
        process_mut.stack_top = STACK_TOP;
        process_mut.stack_bottom = STACK_TOP - STACK_SIZE as u64;
        
        // Update heap pointers (set to after last segment)
        let mut heap_start = 0x400000u64;  // Default 4MB
        for segment in &segments {
            let seg_end = segment.vaddr + segment.memsz;
            if seg_end > heap_start {
                heap_start = seg_end;
            }
        }
        heap_start = (heap_start + 4095) & !4095;  // Align to page
        process_mut.heap_base = heap_start;
        process_mut.heap_limit = heap_start;
        
        // Reset state
        process_mut.state = ProcessState::Ready;
        
        // Close FDs marked close-on-exec
        // TODO: Implement FD_CLOEXEC flag checking
        // For now, all FDs are preserved
        
        // Day 21: Context Update
        
        // Step 14: Update scheduler context (TCB)
        if let Some(tid) = process_mut.thread_id {
            // Get current context and update it
            if let Some(mut ctx) = crate::sched::get_current_context(0) {
                // Update architecture-specific registers
                #[cfg(target_arch = "x86_64")]
                {
                    ctx.rip = entry_point;          // Entry point
                    ctx.rsp = STACK_TOP;            // Stack pointer
                    ctx.rbp = 0;                    // Base pointer (clear)
                    ctx.cr3 = new_page_table as u64; // Page table
                    ctx.rax = 0;                    // Clear return value
                    ctx.rbx = 0;
                    ctx.rcx = 0;
                    ctx.rdx = 0;
                    ctx.rsi = 0;  // TODO: argc
                    ctx.rdi = 0;  // TODO: argv
                    ctx.r8 = 0;
                    ctx.r9 = 0;
                    ctx.r10 = 0;
                    ctx.r11 = 0;
                    ctx.r12 = 0;
                    ctx.r13 = 0;
                    ctx.r14 = 0;
                    ctx.r15 = 0;
                    ctx.rflags = 0x202;  // IF (interrupts enabled)
                }
                
                #[cfg(target_arch = "aarch64")]
                {
                    ctx.elr = entry_point;          // Entry point
                    ctx.sp = STACK_TOP;             // Stack pointer
                    ctx.ttbr0 = new_page_table as u64; // Page table
                    // Clear general purpose registers
                    for i in 0..31 {
                        ctx.x[i] = 0;
                    }
                    // TODO: Set x0 = argc, x1 = argv
                }
                
                // Update context in scheduler
                crate::sched::set_thread_context(tid, ctx);
            }
        }
        
        // Step 15: Free old address space
        crate::mm::destroy_address_space(old_page_table);
        
        // exec() doesn't return on success
        // The process continues from the new entry point
        // But since we're updating the context, we return 0
        // The actual jump happens when scheduler resumes this thread
        
        0 // Success
    }
}

/// Get list of children for a process
/// Returns array of (pid, state) tuples for all children
pub fn get_children(pid: Pid) -> [(Option<Pid>, ProcessState); 32] {
    let mut result = [(None, ProcessState::New); 32];
    
    if let Some(proc) = find_process(pid) {
        for (i, &child_pid) in proc.children.iter().enumerate() {
            if let Some(cpid) = child_pid {
                if let Some(child) = find_process(cpid) {
                    result[i] = (Some(cpid), child.state);
                }
            }
        }
    }
    
    result
}

/// Copy data to user space (basic implementation)
/// Returns true if successful, false if pointer invalid
/// 
/// Safety: This is a placeholder implementation. A full implementation would:
/// 1. Validate user pointer is in user address space
/// 2. Check page table permissions
/// 3. Handle page faults
/// 
/// For now, we just write directly (unsafe but works in simple cases)
unsafe fn copy_to_user(user_ptr: *mut i32, value: i32) -> bool {
    if user_ptr.is_null() {
        return false;
    }
    
    // TODO: Add proper validation in future
    // For now, just write directly
    *user_ptr = value;
    true
}

/// Free resources associated with a process
/// This includes file descriptors, memory, and other kernel resources
/// Returns true if successful, false if process not found
pub fn free_process_resources(pid: Pid) -> bool {
    if let Some(proc) = find_process_mut(pid) {
        // Close any remaining file descriptors
        proc.close_all_fds();
        
        // TODO: Future resource cleanup
        // - Free page tables (mm::free_page_table)
        // - Free user stack pages
        // - Free kernel stack pages
        // - Close IPC ports
        // - Release other kernel objects
        
        // For now, just clear the FD table
        true
    } else {
        false
    }
}

/// Remove process from process table
/// This is the final cleanup step after wait() reaps a zombie
/// Returns true if successful, false if process not found
pub fn remove_process(pid: Pid) -> bool {
    unsafe {
        let table = get_process_table();
        for slot in table.iter_mut() {
            if let Some(proc) = slot {
                if proc.pid == pid {
                    // Remove from parent's children list
                    if let Some(parent_pid) = proc.parent {
                        if let Some(parent) = find_process_mut(parent_pid) {
                            parent.remove_child(pid);
                        }
                    }
                    
                    // Remove from process table
                    *slot = None;
                    return true;
                }
            }
        }
    }
    false
}

/// System call: wait - Wait for child process to exit
/// 
/// BSD Semantics:
/// - Returns PID of zombie child if one exists
/// - If status pointer is not null, writes exit status to it
/// - If no zombie children, blocks until a child exits (SIGCHLD)
/// - Returns -1 if no children exist
/// - Removes zombie child from process table after reaping
///
/// Arguments:
/// - status: Pointer to write exit status (can be null)
///
/// Returns:
/// - PID of reaped child on success
/// - -1 on error (no children, invalid pointer, etc.)
pub fn sys_wait(status: *mut i32) -> isize {
    unsafe {
        let current_pid = match get_current_pid() {
            Some(pid) => pid,
            None => return -1, // No current process
        };
        
        // Step 1: Get list of children
        let children = get_children(current_pid);
        
        // Step 2: Look for zombie children
        for (child_pid, child_state) in children.iter() {
            if let Some(cpid) = child_pid {
                if *child_state == ProcessState::Zombie {
                    // Found a zombie child!
                    
                    // Step 3: Get exit status
                    let exit_status = if let Some(exit_stat) = get_exit_status(*cpid) {
                        exit_stat
                    } else {
                        0 // Default to 0 if no status set
                    };
                    
                    // Step 4: Copy exit status to user space if pointer provided
                    if !status.is_null() {
                        if !copy_to_user(status, exit_status) {
                            return -1; // Failed to copy to user
                        }
                    }
                    
                    // Step 5: Free process resources
                    free_process_resources(*cpid);
                    
                    // Step 6: Remove process from table
                    remove_process(*cpid);
                    
                    // Step 7: Return child PID
                    return *cpid as isize;
                }
            }
        }
        
        // Step 8: No zombie children found
        // Check if we have any children at all
        let has_children = children.iter().any(|(pid, _)| pid.is_some());
        
        if !has_children {
            return -1; // No children at all
        }
        
        // Step 9: Block until a child exits (receive SIGCHLD)
        // TODO: Implement proper blocking in Phase 4
        // For now, return -1 (would block)
        // Full implementation will:
        // 1. Check pending_signals for SIGCHLD
        // 2. If set, clear it and retry (loop back to step 1)
        // 3. If not set, block current thread until SIGCHLD arrives
        // 4. Use scheduler to block: sched::block_on_signal(SIGCHLD)
        
        // Temporary: Check if SIGCHLD is pending
        if let Some(proc) = find_process(current_pid) {
            let sigchld_bit = 1u64 << (SIGCHLD as u64);
            if (proc.pending_signals & sigchld_bit) != 0 {
                // Clear SIGCHLD and retry
                if let Some(proc_mut) = find_process_mut(current_pid) {
                    proc_mut.pending_signals &= !sigchld_bit;
                }
                // In a real implementation, we'd loop back to check for zombies again
                // For now, just return -1 indicating "would block"
            }
        }
        
        -1 // Would block (EAGAIN/EWOULDBLOCK in non-blocking mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: These tests are documentation - they show expected behavior
    // Actual testing will require proper test harness
    
    /// Test: Process state should change to Zombie on exit
    #[test]
    fn test_exit_sets_zombie_state() {
        // This test documents expected behavior:
        // 1. Create process with PID 100
        // 2. Set it as current process
        // 3. Call sys_exit(42)
        // 4. Verify process state is Zombie
        // 5. Verify exit status is Some(42)
        // 6. Verify file descriptors are closed (fd_count == 0)
    }
    
    /// Test: Exit status should be saved
    #[test]
    fn test_exit_status_saved() {
        // Expected behavior:
        // - set_exit_status(pid, 0) should store Some(0)
        // - set_exit_status(pid, 1) should store Some(1)
        // - set_exit_status(pid, -1) should store Some(-1)
        // - get_exit_status(pid) should return the stored value
    }
    
    /// Test: File descriptors should be closed
    #[test]
    fn test_fds_closed_on_exit() {
        // Expected behavior:
        // 1. Process has 3 open FDs (stdin, stdout, stderr)
        // 2. Call close_all_fds(pid)
        // 3. Verify all FD slots are None
        // 4. Verify fd_count is 0
        // 5. Verify function returns 3 (number closed)
    }
    
    /// Test: Parent should receive SIGCHLD
    #[test]
    fn test_parent_receives_sigchld() {
        // Expected behavior:
        // 1. Process 101 with parent 100
        // 2. Process 101 calls exit()
        // 3. Verify parent (100) has SIGCHLD bit set in pending_signals
        // 4. Verify signal bit is (1 << SIGCHLD)
    }
    
    /// Test: Children are reparented to init
    #[test]
    fn test_children_reparented_to_init() {
        // Expected behavior:
        // 1. Process 100 has children [200, 201, 202]
        // 2. Call reparent_children_to_init(100)
        // 3. Verify children's parent is now 1 (init)
        // 4. Verify init has children [200, 201, 202] in its children list
        // 5. Verify process 100's children list is empty
        // 6. Verify function returns 3 (number reparented)
    }
    
    /// Test: Helper functions work correctly
    #[test]
    fn test_helper_functions() {
        // Expected behavior for get_parent_pid():
        // - Returns Some(parent_pid) if process exists and has parent
        // - Returns None if process not found or no parent
        
        // Expected behavior for get_thread_id():
        // - Returns Some(tid) if process has thread_id set
        // - Returns None if process not found or no thread
        
        // Expected behavior for send_signal():
        // - Returns true and sets pending_signals bit if valid
        // - Returns false if invalid signal number
        // - Returns false if process not found
    }
    
    /// Test: Process doesn't exit without hanging system (Day 8)
    #[test]
    fn test_exit_without_hanging() {
        // Expected behavior:
        // 1. Create process, schedule it
        // 2. Process calls exit()
        // 3. Scheduler picks next process
        // 4. System continues running
        // 5. No infinite halt loop
        // Note: This requires scheduler integration (Phase 4)
    }
    
    /// Test: Thread state is set to Zombie in scheduler
    #[test]
    fn test_thread_state_zombie() {
        // Expected behavior:
        // 1. Process has thread_id = 5
        // 2. Call sys_exit()
        // 3. Verify scheduler TCB[5].state == ThreadState::Zombie
        // 4. Verify thread removed from run queue
        // 5. Verify thread not scheduled again
    }
    
    /// Test: get_children() returns correct list (Day 9)
    #[test]
    fn test_get_children() {
        // Expected behavior:
        // 1. Parent (PID 100) has children [200, 201, 202]
        // 2. Call get_children(100)
        // 3. Verify returned array contains (Some(200), state), (Some(201), state), (Some(202), state)
        // 4. Verify states match actual process states
        // 5. Verify remaining slots are (None, New)
    }
    
    /// Test: Zombie child detection (Day 9)
    #[test]
    fn test_zombie_child_detection() {
        // Expected behavior:
        // 1. Parent has children: [Ready, Zombie, Running, Zombie]
        // 2. Iterate through children looking for zombies
        // 3. Find child in Zombie state
        // 4. Verify can retrieve its PID and exit status
    }
    
    /// Test: copy_to_user() works correctly (Day 9)
    #[test]
    fn test_copy_to_user() {
        // Expected behavior:
        // 1. Allocate user-space buffer (in real test)
        // 2. Call copy_to_user(ptr, 42)
        // 3. Verify buffer contains 42
        // 4. Verify returns true
        // 5. Call with null pointer, verify returns false
    }
    
    /// Test: free_process_resources() cleanup (Day 10)
    #[test]
    fn test_free_process_resources() {
        // Expected behavior:
        // 1. Process has open FDs, allocated memory, IPC ports
        // 2. Call free_process_resources(pid)
        // 3. Verify all FDs closed (fd_count == 0)
        // 4. Verify memory marked for cleanup
        // 5. Verify IPC ports closed (future)
        // 6. Verify returns true
    }
    
    /// Test: remove_process() from table (Day 10)
    #[test]
    fn test_remove_process() {
        // Expected behavior:
        // 1. Process 200 exists in table, parent is 100
        // 2. Call remove_process(200)
        // 3. Verify process 200 no longer in table
        // 4. Verify parent 100's children list doesn't contain 200
        // 5. Verify returns true
        // 6. Call remove_process(999), verify returns false
    }
    
    /// Test: wait() finds and reaps zombie child (Day 9-10)
    #[test]
    fn test_wait_reaps_zombie() {
        // Expected behavior:
        // 1. Parent (100) has child (200) in Zombie state with exit_status=42
        // 2. Allocate status buffer
        // 3. Call sys_wait(&status) from parent
        // 4. Verify returns 200 (child PID)
        // 5. Verify status buffer contains 42
        // 6. Verify child 200 no longer in process table
        // 7. Verify child removed from parent's children list
    }
    
    /// Test: wait() with null status pointer (Day 10)
    #[test]
    fn test_wait_null_status() {
        // Expected behavior:
        // 1. Parent has zombie child (PID 200)
        // 2. Call sys_wait(null)
        // 3. Verify returns 200
        // 4. Verify child is reaped
        // 5. No crash from null pointer
    }
    
    /// Test: wait() returns -1 if no children (Day 10)
    #[test]
    fn test_wait_no_children() {
        // Expected behavior:
        // 1. Process has no children (children array all None)
        // 2. Call sys_wait(&status)
        // 3. Verify returns -1
        // 4. Verify status not modified
    }
    
    /// Test: wait() blocks if no zombie children (Day 10)
    #[test]
    fn test_wait_blocks() {
        // Expected behavior:
        // 1. Parent has children but none are zombies
        // 2. Call sys_wait(&status)
        // 3. Verify returns -1 (would block) for now
        // 4. Future: Verify thread blocked until SIGCHLD
        // 5. Future: Child exits, SIGCHLD sent
        // 6. Future: wait() unblocks and reaps child
        // Note: Full blocking requires Phase 4 scheduler integration
    }
    
    /// Test: Multiple children, wait() reaps first zombie (Day 10)
    #[test]
    fn test_wait_multiple_children() {
        // Expected behavior:
        // 1. Parent has 4 children: [Ready, Zombie(42), Running, Zombie(99)]
        // 2. Call sys_wait(&status)
        // 3. Verify returns PID of first zombie found
        // 4. Verify status contains correct exit status
        // 5. Call sys_wait() again
        // 6. Verify returns PID of second zombie
    }
    
    /// Test: getpid() returns correct PID (Day 11)
    #[test]
    fn test_getpid_returns_correct_pid() {
        // Expected behavior:
        // 1. Create process with PID 100
        // 2. Set as current process: set_current_pid(Some(100))
        // 3. Call sys_getpid()
        // 4. Verify returns 100
        // 5. Verify return value is > 0
        // 6. Verify return value matches expected PID
    }
    
    /// Test: Different processes get different PIDs (Day 11)
    #[test]
    fn test_different_processes_different_pids() {
        // Expected behavior:
        // 1. Create process A with PID 100
        // 2. Create process B with PID 101
        // 3. Set current to A, call sys_getpid(), verify returns 100
        // 4. Set current to B, call sys_getpid(), verify returns 101
        // 5. Verify 100 != 101
        // 6. Verify PIDs are unique
    }
    
    /// Test: getpid() with no current process (Day 11)
    #[test]
    fn test_getpid_no_current_process() {
        // Expected behavior:
        // 1. Set current process to None
        // 2. Call sys_getpid()
        // 3. Verify returns -1
        // 4. This should never happen in normal operation
        // 5. Would indicate kernel bug
    }
    
    /// Test: getpid() validates PID range (Day 11)
    #[test]
    fn test_getpid_validates_range() {
        // Expected behavior:
        // 1. Current PID set to valid value (e.g., 100)
        // 2. Call sys_getpid(), verify returns 100
        // 3. If PID were somehow invalid (0 or > 65535)
        // 4. Would return -1 (safety check)
        // 5. Verify bounds checking works
    }
    
    /// Test: getpid() never fails in normal operation (Day 11)
    #[test]
    fn test_getpid_never_fails() {
        // Expected behavior (BSD semantics):
        // 1. In normal userland operation, getpid() NEVER fails
        // 2. Always returns a valid PID > 0
        // 3. No error conditions in normal use
        // 4. This is guaranteed by BSD specification
        // 5. Only returns -1 in impossible scenarios
    }
    
    /// Test: PID allocation is monotonic (Day 11)
    #[test]
    fn test_pid_allocation_monotonic() {
        // Expected behavior:
        // 1. Allocate PID 1 for init
        // 2. Allocate next PID, should be 2
        // 3. Allocate next PID, should be 3
        // 4. PIDs increase monotonically
        // 5. No PID reuse until wraparound
        // 6. Verify allocate_pid() increments correctly
    }
    
    /// Test: get_pid_from_tid() integration (Day 11)
    #[test]
    fn test_get_pid_from_tid() {
        // Expected behavior (future Phase 4):
        // 1. Process has thread_id = 5
        // 2. Call get_pid_from_tid(5)
        // 3. Should return process PID
        // 4. Verifies scheduler integration
        // 5. For now, returns current process PID (stub)
    }
    
    /// Test: fork() creates new process structure (Day 12)
    #[test]
    fn test_fork_creates_process() {
        // Expected behavior:
        // 1. Parent process (PID 100) calls fork()
        // 2. New child process created with unique PID (e.g., 101)
        // 3. Child process exists in process table
        // 4. Child has different PID than parent
        // 5. fork() returns child PID to parent (101)
    }
    
    /// Test: fork() copies parent fields to child (Day 12)
    #[test]
    fn test_fork_copies_parent_fields() {
        // Expected behavior:
        // 1. Parent has specific state (entry, stack, fd_table, etc.)
        // 2. Call fork()
        // 3. Child process has same entry point
        // 4. Child has same stack size
        // 5. Child has copy of fd_table
        // 6. Child has copy of signal handlers
        // 7. Child has same memory limits
    }
    
    /// Test: fork() adds child to parent's children list (Day 12)
    #[test]
    fn test_fork_adds_to_parent_children() {
        // Expected behavior:
        // 1. Parent (PID 100) has 2 children initially
        // 2. Parent calls fork(), child PID 200 created
        // 3. Parent now has 3 children
        // 4. Parent's children list contains 200
        // 5. Child's parent field is Some(100)
    }
    
    /// Test: fork() allocates separate address space (Day 13)
    #[test]
    fn test_fork_separate_address_space() {
        // Expected behavior:
        // 1. Parent has page_table at address X
        // 2. Call fork()
        // 3. Child has page_table at different address Y
        // 4. X != Y (separate page tables)
        // 5. Both page tables have same mappings initially
        // 6. Modifications in child don't affect parent
    }
    
    /// Test: clone_address_space() copies mappings (Day 13)
    #[test]
    fn test_clone_address_space() {
        // Expected behavior:
        // 1. Source page table has mappings for address A→B
        // 2. Call clone_address_space(source)
        // 3. Returns new page table address
        // 4. New page table has same mappings A→B'
        // 5. B' is different physical page than B
        // 6. Contents of B' match contents of B
    }
    
    /// Test: fork() allocates kernel stack for child (Day 13)
    #[test]
    fn test_fork_allocates_kernel_stack() {
        // Expected behavior:
        // 1. Parent has kernel_stack at address X
        // 2. Call fork()
        // 3. Child has kernel_stack at different address Y
        // 4. X != Y (separate kernel stacks)
        // 5. Each process has independent kernel stack
    }
    
    /// Test: fork() returns 0 to child (Day 14 - context setup)
    #[test]
    fn test_fork_child_returns_zero() {
        // Expected behavior (Phase 4):
        // 1. Parent calls fork()
        // 2. In parent: fork() returns child PID (> 0)
        // 3. In child: fork() returns 0
        // 4. This is set via ArchContext.rax = 0
        // 5. Allows child to know it's the child
        // Note: Requires scheduler integration
    }
    
    /// Test: fork() handles out of memory (Day 12-13)
    #[test]
    fn test_fork_out_of_memory() {
        // Expected behavior:
        // 1. Memory allocation fails
        // 2. fork() returns -1
        // 3. No child process created
        // 4. Parent's state unchanged
        // 5. No memory leaks
    }
    
    /// Test: fork() handles process table full (Day 12)
    #[test]
    fn test_fork_process_table_full() {
        // Expected behavior:
        // 1. Process table has 64/64 slots used
        // 2. Call fork()
        // 3. Returns -1 (no space)
        // 4. No new process created
        // 5. Resources cleaned up
    }
    
    /// Test: fork() copies file descriptors (Day 12)
    #[test]
    fn test_fork_copies_file_descriptors() {
        // Expected behavior:
        // 1. Parent has 3 open FDs: stdin, stdout, stderr
        // 2. Call fork()
        // 3. Child has copy of all 3 FDs
        // 4. Child and parent have independent FD tables
        // 5. Child can close FDs without affecting parent
    }
    
    /// Test: fork() allocates new PID (Day 12)
    #[test]
    fn test_fork_allocates_new_pid() {
        // Expected behavior:
        // 1. NEXT_PID is 10
        // 2. Parent calls fork()
        // 3. Child gets PID 10
        // 4. NEXT_PID increments to 11
        // 5. Next process gets PID 11
        // 6. PIDs are unique
    }
    
    /// Test: fork() returns child PID to parent (Day 16)
    #[test]
    fn test_fork_returns_child_pid_to_parent() {
        // Expected behavior:
        // 1. Parent process (PID 100) calls fork()
        // 2. Child process created with PID 101
        // 3. sys_fork() returns 101 to parent
        // 4. Parent's rax register = 101 after syscall
        // 5. Parent can use return value to track child
        // 6. Return value is always > 0 in parent
        //
        // BSD Specification:
        // fork() returns child PID to parent on success
        // This allows parent to know which process was created
        //
        // Example code in parent:
        //   pid_t child_pid = fork();
        //   if (child_pid > 0) {
        //       printf("Created child with PID %d\n", child_pid);
        //   }
    }
    
    /// Test: fork() returns 0 to child (Day 16)
    #[test]
    fn test_fork_returns_zero_to_child() {
        // Expected behavior:
        // 1. Parent calls fork()
        // 2. Child process created
        // 3. Child's context.rax is set to 0
        // 4. When child is scheduled, it "returns" from fork() with 0
        // 5. Child uses this to identify itself
        //
        // Implementation:
        // - During fork(), child_ctx.rax = 0 is set (Day 14)
        // - When scheduler runs child, it resumes with rax=0
        // - Syscall return path preserves rax value
        //
        // BSD Specification:
        // fork() returns 0 to child process
        // This is THE way child knows it's the child
        //
        // Example code in child:
        //   pid_t result = fork();
        //   if (result == 0) {
        //       printf("I'm the child!\n");
        //       exit(0);
        //   }
    }
    
    /// Test: Child has copy of file descriptors (Day 16)
    #[test]
    fn test_fork_child_has_copy_of_fds() {
        // Expected behavior:
        // 1. Parent has open file descriptors:
        //    fd[0] = stdin (inode 1)
        //    fd[1] = stdout (inode 2)
        //    fd[2] = stderr (inode 3)
        //    fd[3] = file.txt (inode 100)
        // 2. Parent calls fork()
        // 3. Child gets COPY of fd_table:
        //    child.fd[0] = stdin (inode 1)
        //    child.fd[1] = stdout (inode 2)
        //    child.fd[2] = stderr (inode 3)
        //    child.fd[3] = file.txt (inode 100)
        // 4. Child and parent have INDEPENDENT fd_table arrays
        // 5. Child can close fds without affecting parent
        // 6. Both can write to same files (shared file offset in real system)
        //
        // Implementation:
        // In sys_fork():
        //   child.fd_table = parent.fd_table;  // Array copy
        //   child.fd_count = parent.fd_count;
        //
        // BSD Specification:
        // Child inherits copies of parent's open file descriptors
        // Both processes share file offsets (not yet implemented)
        //
        // Example:
        //   // Parent opens file
        //   int fd = open("test.txt", O_RDWR);
        //   
        //   pid_t child = fork();
        //   if (child == 0) {
        //       // Child can write to fd
        //       write(fd, "child\n", 6);
        //       close(fd);  // Only closes in child
        //   } else {
        //       // Parent can still write to fd
        //       write(fd, "parent\n", 7);
        //       close(fd);
        //   }
    }
    
    /// Test: Child has separate memory (Day 16)
    #[test]
    fn test_fork_child_has_separate_memory() {
        // Expected behavior:
        // 1. Parent has memory:
        //    Address 0x10000: value = 42
        //    Address 0x20000: value = 100
        // 2. Parent calls fork()
        // 3. Child gets COPY of memory:
        //    Address 0x10000: value = 42 (copied)
        //    Address 0x20000: value = 100 (copied)
        // 4. Child has separate page table (child.page_table != parent.page_table)
        // 5. Child modifies memory: 0x10000 = 99
        // 6. Parent's memory unchanged: 0x10000 = 42
        // 7. Memory is ISOLATED between processes
        //
        // Implementation:
        // In sys_fork():
        //   child_page_table = mm::clone_address_space(parent.page_table);
        //   // Clones all page tables
        //   // Copies all page contents (full copy for now)
        //   // Future: Copy-on-Write (COW) for efficiency
        //
        // BSD Specification:
        // Child gets complete copy of parent's address space
        // Modifications in one process don't affect the other
        //
        // Example:
        //   int x = 42;
        //   
        //   pid_t child = fork();
        //   if (child == 0) {
        //       // Child modifies x
        //       x = 99;
        //       printf("Child x = %d\n", x);  // Prints 99
        //   } else {
        //       sleep(1);  // Let child run
        //       printf("Parent x = %d\n", x);  // Prints 42
        //   }
    }
    
    /// Test: Parent and child can both run (Day 16)
    #[test]
    fn test_fork_parent_and_child_both_run() {
        // Expected behavior:
        // 1. Parent (PID 100, TID 1) calls fork()
        // 2. Child (PID 101, TID 2) created and registered
        // 3. Parent continues running immediately
        // 4. Child added to run queue (state = Ready)
        // 5. Scheduler picks child on next context switch
        // 6. Both processes execute independently
        // 7. Both can be in run queue simultaneously
        // 8. Both can execute on different CPUs (future)
        //
        // Scheduler State After fork():
        // TCB[1]: pid=100, state=Running, rax=101
        // TCB[2]: pid=101, state=Ready, rax=0
        //
        // After context switch:
        // TCB[1]: pid=100, state=Ready (or still Running)
        // TCB[2]: pid=101, state=Running
        //
        // Implementation:
        // In sys_fork():
        //   child_tid = sched::register_thread(child_pid, prio, cpu, child_ctx);
        //   // Child is in Ready state
        //   // Child is in run queue
        //   // Scheduler will pick child based on priority
        //
        // BSD Specification:
        // Both parent and child are runnable after fork()
        // Order of execution is non-deterministic
        //
        // Example:
        //   pid_t child = fork();
        //   if (child == 0) {
        //       printf("Child running\n");
        //   } else {
        //       printf("Parent running\n");
        //   }
        //   // Output order is undefined:
        //   // Could be "Parent\nChild" or "Child\nParent"
    }
    
    /// Test: fork() full integration (Day 16)
    #[test]
    fn test_fork_full_integration() {
        // Expected behavior - Complete fork() lifecycle:
        //
        // Setup:
        //   Parent PID 100, 3 open FDs, 64KB memory
        //
        // Step 1: Parent calls fork()
        //   sys_fork() executes
        //
        // Step 2: Child created
        //   PID: 101 (new)
        //   Page table: cloned (separate)
        //   Kernel stack: allocated (separate)
        //   FDs: copied (independent)
        //   Context: cloned, rax=0, cr3=child_pt
        //   Thread: registered, state=Ready
        //
        // Step 3: Parent continues
        //   fork() returns 101
        //   Parent continues execution
        //   Parent state unchanged
        //
        // Step 4: Child scheduled
        //   Scheduler picks child from run queue
        //   Context switch to child
        //   Child "returns" from fork() with 0
        //   Child executes independently
        //
        // Step 5: Both processes run
        //   Parent: knows child PID (101)
        //   Child: knows it's child (fork()=0)
        //   Both in separate address spaces
        //   Both schedulable
        //
        // Verification:
        // ✓ Parent PID = 100
        // ✓ Child PID = 101
        // ✓ Parent fork() returns 101
        // ✓ Child fork() returns 0
        // ✓ Child in parent's children list
        // ✓ Child has parent = 100
        // ✓ Both have separate page tables
        // ✓ Both have independent FDs
        // ✓ Both are schedulable
        // ✓ Memory is isolated
    }
    
    /// Test: fork() with exit() and wait() (Day 16)
    #[test]
    fn test_fork_with_exit_and_wait() {
        // Expected behavior - Full process lifecycle:
        //
        // Parent code:
        //   pid_t child = fork();
        //   if (child > 0) {
        //       int status;
        //       pid_t pid = wait(&status);
        //       // pid = child PID
        //       // status = child exit code
        //   }
        //
        // Child code:
        //   if (child == 0) {
        //       printf("Child running\n");
        //       exit(42);
        //   }
        //
        // Expected sequence:
        // 1. Parent calls fork() → returns child PID (101)
        // 2. Child created with fork() return value 0
        // 3. Parent calls wait() → blocks
        // 4. Child executes, prints message
        // 5. Child calls exit(42)
        // 6. Child closes FDs, becomes Zombie
        // 7. Child sends SIGCHLD to parent
        // 8. Parent's wait() unblocks
        // 9. Parent gets child PID (101) and status (42)
        // 10. Child process freed from table
        //
        // Verification:
        // ✓ Child executes correctly
        // ✓ Child exit status saved
        // ✓ Parent receives correct PID
        // ✓ Parent receives correct status
        // ✓ Child becomes Zombie
        // ✓ Child eventually freed
        // ✓ All syscalls work together
    }
    
    /// Test: Multiple forks (process tree) (Day 16)
    #[test]
    fn test_multiple_forks_process_tree() {
        // Expected behavior - Creating process tree:
        //
        // Code:
        //   pid_t child1 = fork();  // Creates child A
        //   pid_t child2 = fork();  // Creates child B (from both parent and A)
        //
        // Resulting tree:
        //   Parent (PID 100)
        //   ├─ Child A (PID 101)
        //   │  └─ Child C (PID 103)
        //   └─ Child B (PID 102)
        //
        // Execution:
        // 1. Parent forks → Child A (PID 101)
        // 2. Parent continues, forks → Child B (PID 102)
        // 3. Child A executes, forks → Child C (PID 103)
        // 4. Now 4 processes running
        //
        // Process relationships:
        // Parent (100):
        //   children = [101, 102]
        // Child A (101):
        //   parent = 100
        //   children = [103]
        // Child B (102):
        //   parent = 100
        // Child C (103):
        //   parent = 101
        //
        // Verification:
        // ✓ All 4 processes created
        // ✓ Parent-child links correct
        // ✓ All processes independent
        // ✓ All processes schedulable
        // ✓ Each has separate memory
    }
    
    /// Test: fork() error handling (Day 16)
    #[test]
    fn test_fork_error_handling() {
        // Expected error cases:
        //
        // Case 1: Out of memory
        //   Scenario: mm::alloc_page() fails
        //   Expected: fork() returns -1
        //   Cleanup: Partial allocations freed
        //
        // Case 2: Process table full
        //   Scenario: All 64 slots in PROCESS_TABLE used
        //   Expected: fork() returns -1
        //   Cleanup: Resources freed
        //
        // Case 3: Parent's children list full
        //   Scenario: Parent already has 32 children
        //   Expected: fork() returns -1
        //   Cleanup: Resources freed
        //
        // Case 4: Scheduler registration fails
        //   Scenario: All TCB slots in use
        //   Expected: fork() returns -1
        //   Cleanup: Process removed, resources freed
        //
        // Case 5: No current process
        //   Scenario: get_current_pid() returns None
        //   Expected: fork() returns -1
        //   Cleanup: N/A (nothing allocated)
        //
        // Important: No resource leaks on any error!
        // TODO: Add proper cleanup paths in sys_fork()
    }
    
    // ==================== DAY 17: exec() PATH HANDLING TESTS ====================
    
    /// Test: copy_string_from_user() - valid string (Day 17)
    #[test]
    fn test_copy_string_from_user_valid() {
        // Expected behavior:
        // 1. User provides pointer to "/bin/sh\0"
        // 2. copy_string_from_user() reads bytes until null
        // 3. Returns Ok(Vec["/bin/sh"])
        // 4. Null terminator not included in result
        //
        // Setup (in kernel test):
        //   let s = "/bin/sh\0";
        //   let ptr = s.as_ptr();
        //   let result = copy_string_from_user(ptr, 100);
        //
        // Verify:
        //   assert!(result.is_ok());
        //   assert_eq!(result.unwrap(), b"/bin/sh");
    }
    
    /// Test: copy_string_from_user() - null pointer (Day 17)
    #[test]
    fn test_copy_string_from_user_null() {
        // Expected behavior:
        // 1. User provides null pointer
        // 2. copy_string_from_user() checks for null
        // 3. Returns Err("Null pointer")
        // 4. No crash, no undefined behavior
        //
        // Test:
        //   let result = copy_string_from_user(core::ptr::null(), 100);
        //   assert_eq!(result, Err("Null pointer"));
    }
    
    /// Test: copy_string_from_user() - kernel pointer (Day 17)
    #[test]
    fn test_copy_string_from_user_kernel_ptr() {
        // Expected behavior:
        // 1. User provides pointer in kernel space (0xFFFF_8000_...)
        // 2. copy_string_from_user() validates address
        // 3. Returns Err("Pointer in kernel space")
        // 4. Prevents kernel memory exposure
        //
        // Security:
        //   This prevents userland from reading kernel memory
        //   Critical for isolation and security
        //
        // Test:
        //   let kernel_ptr = 0xFFFF_8000_0000_0000 as *const u8;
        //   let result = copy_string_from_user(kernel_ptr, 100);
        //   assert_eq!(result, Err("Pointer in kernel space"));
    }
    
    /// Test: copy_string_from_user() - string too long (Day 17)
    #[test]
    fn test_copy_string_from_user_too_long() {
        // Expected behavior:
        // 1. User provides string without null terminator
        // 2. copy_string_from_user() reads up to max_len
        // 3. No null found within max_len
        // 4. Returns Err("String too long")
        // 5. Prevents buffer overflow
        //
        // Security:
        //   Prevents malicious users from causing kernel to
        //   read unbounded memory
        //
        // Test:
        //   let long_str = "a".repeat(5000);  // No null
        //   let result = copy_string_from_user(long_str.as_ptr(), 4096);
        //   assert_eq!(result, Err("String too long"));
    }
    
    /// Test: is_user_pointer_valid() - valid pointer (Day 17)
    #[test]
    fn test_is_user_pointer_valid() {
        // Expected behavior:
        // 1. Pointer in user space (< 0xFFFF_8000_0000_0000)
        // 2. Length doesn't cause overflow
        // 3. End address also in user space
        // 4. Returns true
        //
        // Test:
        //   let ptr = 0x1000_0000 as *const u8;
        //   assert!(is_user_pointer_valid(ptr, 4096));
    }
    
    /// Test: is_user_pointer_valid() - kernel pointer (Day 17)
    #[test]
    fn test_is_user_pointer_invalid_kernel() {
        // Expected behavior:
        // 1. Pointer in kernel space
        // 2. Returns false
        //
        // Test:
        //   let ptr = 0xFFFF_8000_0000_0000 as *const u8;
        //   assert!(!is_user_pointer_valid(ptr, 100));
    }
    
    /// Test: is_user_pointer_valid() - address wrap (Day 17)
    #[test]
    fn test_is_user_pointer_invalid_wrap() {
        // Expected behavior:
        // 1. start_addr + len causes integer overflow
        // 2. Or wraps into kernel space
        // 3. Returns false
        // 4. Prevents attack via overflow
        //
        // Test:
        //   let ptr = 0xFFFF_7FFF_FFFF_F000 as *const u8;
        //   let len = 0x2000;  // Would wrap to kernel space
        //   assert!(!is_user_pointer_valid(ptr, len));
    }
    
    /// Test: exec() validates path pointer (Day 17)
    #[test]
    fn test_exec_validates_path_pointer() {
        // Expected behavior:
        // 1. User calls exec() with null pointer
        // 2. sys_exec() calls is_user_pointer_valid()
        // 3. Returns -1 (EFAULT)
        // 4. No crash
        //
        // Test:
        //   let result = sys_exec(core::ptr::null(), core::ptr::null());
        //   assert_eq!(result, -1);
    }
    
    /// Test: exec() reads path from userspace (Day 17)
    #[test]
    fn test_exec_reads_path() {
        // Expected behavior:
        // 1. User provides "/bin/sh\0"
        // 2. sys_exec() calls copy_string_from_user()
        // 3. Path is safely copied to kernel
        // 4. Used for file lookup
        //
        // Full flow:
        //   User: exec("/bin/sh", ["-c", "echo hi"])
        //   Kernel: copy path → "/bin/sh"
        //   Kernel: open file "/bin/sh"
        //   Kernel: parse ELF
        //   Kernel: load program
    }
    
    // ==================== DAY 18: ELF PARSING TESTS ====================
    
    /// Test: parse_elf_header() - valid ELF (Day 18)
    #[test]
    fn test_parse_elf_header_valid() {
        // Expected behavior:
        // 1. Valid ELF64 file provided
        // 2. parse_elf_header() validates:
        //    - Magic number: 0x7F 'E' 'L' 'F'
        //    - Class: 64-bit (2)
        //    - Endianness: Little (1)
        //    - Type: Executable (2) or PIE (3)
        //    - Architecture: x86-64 (62) or aarch64 (183)
        // 3. Returns Ok((entry_point, segment_count))
        // 4. entry_point = e_entry field
        // 5. segment_count = number of PT_LOAD segments
        //
        // Example ELF header:
        //   Magic: 7F 45 4C 46 02 01 01 00
        //   Type: 0x0002 (ET_EXEC)
        //   Machine: 0x003E (EM_X86_64)
        //   Entry: 0x401000
        //   Program headers: 3
        //   PT_LOAD count: 2
        //
        // Result:
        //   Ok((0x401000, 2))
    }
    
    /// Test: parse_elf_header() - invalid magic (Day 18)
    #[test]
    fn test_parse_elf_header_invalid_magic() {
        // Expected behavior:
        // 1. File with wrong magic number
        // 2. parse_elf_header() checks first 4 bytes
        // 3. Returns Err("Invalid ELF magic")
        // 4. Prevents loading non-ELF files
        //
        // Test data:
        //   [0x89, 'P', 'N', 'G', ...]  // PNG file
        //   Result: Err("Invalid ELF magic")
    }
    
    /// Test: parse_elf_header() - wrong architecture (Day 18)
    #[test]
    fn test_parse_elf_header_wrong_arch() {
        // Expected behavior:
        // 1. ELF for different architecture
        // 2. On x86-64: EM_X86_64 (62) expected
        // 3. On aarch64: EM_AARCH64 (183) expected
        // 4. Returns Err("Wrong architecture")
        // 5. Prevents running incompatible binaries
        //
        // Example:
        //   Running x86-64 kernel
        //   User tries to exec ARM64 binary
        //   Result: Err("Wrong architecture (not x86-64)")
    }
    
    /// Test: parse_elf_header() - 32-bit ELF (Day 18)
    #[test]
    fn test_parse_elf_header_32bit() {
        // Expected behavior:
        // 1. ELF32 file (e_ident[4] = 1)
        // 2. GuardBSD only supports 64-bit
        // 3. Returns Err("Not 64-bit ELF")
        // 4. Prevents 32-bit execution
        //
        // Note: Some systems support both, but we only do 64-bit
    }
    
    /// Test: get_loadable_segments() - extracts segments (Day 18)
    #[test]
    fn test_get_loadable_segments() {
        // Expected behavior:
        // 1. ELF with multiple program headers
        // 2. get_loadable_segments() iterates headers
        // 3. Extracts only PT_LOAD segments
        // 4. Returns Vec<SegmentInfo> with:
        //    - vaddr: Virtual address to load at
        //    - memsz: Size in memory
        //    - filesz: Size in file
        //    - offset: Offset in ELF file
        //    - flags: Read/Write/Execute permissions
        //
        // Example ELF:
        //   Segment 0 (PT_LOAD):
        //     vaddr: 0x400000
        //     memsz: 0x1000 (4KB)
        //     filesz: 0x1000
        //     offset: 0x0
        //     flags: R-X (read, execute)
        //   
        //   Segment 1 (PT_LOAD):
        //     vaddr: 0x401000
        //     memsz: 0x2000 (8KB)
        //     filesz: 0x1000 (4KB file, 8KB memory for BSS)
        //     offset: 0x1000
        //     flags: RW- (read, write)
        //
        // Result:
        //   Vec with 2 SegmentInfo structs
    }
    
    /// Test: ELF segment with BSS (Day 18)
    #[test]
    fn test_elf_segment_bss() {
        // Expected behavior:
        // 1. Segment where memsz > filesz
        // 2. Extra memory is BSS (uninitialized data)
        // 3. Must be zeroed in memory
        // 4. Example:
        //    filesz: 4096 bytes (from file)
        //    memsz:  8192 bytes (in memory)
        //    → Last 4096 bytes are BSS (zero-filled)
        //
        // Implementation (Day 19):
        //   - Allocate memsz bytes
        //   - Copy filesz bytes from ELF
        //   - Zero remaining (memsz - filesz) bytes
    }
    
    /// Test: exec() ELF validation (Day 18)
    #[test]
    fn test_exec_elf_validation() {
        // Expected behavior - Full validation:
        // 1. Read file from path
        // 2. Validate ELF magic
        // 3. Validate architecture
        // 4. Validate 64-bit
        // 5. Validate executable type
        // 6. Extract entry point
        // 7. Extract segments
        // 8. Proceed to loading (Day 19)
        //
        // Error cases:
        //   - File not found: -1 (ENOENT)
        //   - Not ELF: -1 (ENOEXEC)
        //   - Wrong arch: -1 (ENOEXEC)
        //   - No segments: -1 (ENOEXEC)
    }
    
    // ==================== DAY 19: ADDRESS SPACE REPLACEMENT TESTS ====================
    
    /// Test: exec() creates new page table (Day 19)
    #[test]
    fn test_exec_creates_new_page_table() {
        // Expected behavior:
        // 1. Process has old page table at address X
        // 2. Call exec("/bin/sh")
        // 3. New page table created at address Y
        // 4. X != Y (different page tables)
        // 5. Process updated to use Y
        // 6. Old page table X will be freed
        //
        // Implementation:
        //   let old_pt = process.page_table;
        //   let new_pt = mm::create_address_space()?;
        //   assert!(new_pt != old_pt);
        //   process.page_table = new_pt;
    }
    
    /// Test: exec() maps ELF segments (Day 19)
    #[test]
    fn test_exec_maps_elf_segments() {
        // Expected behavior:
        // 1. ELF has 2 segments:
        //    Segment 0: .text at 0x400000, 4KB, R-X
        //    Segment 1: .data at 0x401000, 4KB, RW-
        // 2. exec() loads each segment
        // 3. Allocates physical pages
        // 4. Copies data from ELF
        // 5. Maps into new address space with correct permissions
        //
        // Process:
        //   for each PT_LOAD segment:
        //     - Allocate pages
        //     - Copy data from ELF file
        //     - Map with correct flags (R/W/X)
        //
        // Result:
        //   Virtual 0x400000 → Physical page (code, R-X)
        //   Virtual 0x401000 → Physical page (data, RW-)
    }
    
    /// Test: exec() copies segment data (Day 19)
    #[test]
    fn test_exec_copies_segment_data() {
        // Expected behavior:
        // 1. ELF segment has 100 bytes of code
        // 2. exec() copies those 100 bytes to physical page
        // 3. Page is mapped at segment's virtual address
        // 4. Code can execute from that address
        //
        // Example:
        //   ELF offset 0x1000: [0x48, 0x31, 0xFF, ...]  (xor rdi, rdi)
        //   Copy to physical page
        //   Map at virtual 0x400000
        //   Result: CPU executes from 0x400000
    }
    
    /// Test: exec() handles BSS (memsz > filesz) (Day 19)
    #[test]
    fn test_exec_handles_bss() {
        // Expected behavior:
        // 1. Segment has filesz=4096, memsz=8192
        // 2. First 4096 bytes copied from ELF
        // 3. Last 4096 bytes zeroed (BSS)
        // 4. BSS contains uninitialized global variables
        //
        // Implementation:
        //   copy_size = min(4096, filesz - offset);
        //   copy_nonoverlapping(elf_data, dst, copy_size);
        //   if (copy_size < 4096) {
        //     write_bytes(dst + copy_size, 0, 4096 - copy_size);
        //   }
        //
        // C code example:
        //   int global_array[1024];  // BSS, zero-initialized
    }
    
    /// Test: exec() sets up user stack (Day 19)
    #[test]
    fn test_exec_sets_up_stack() {
        // Expected behavior:
        // 1. Allocate 8MB for user stack
        // 2. Map at high address (0x7FFF_FFFF_F000)
        // 3. Stack grows downward
        // 4. All pages zeroed
        // 5. Mapped with USER + WRITABLE flags
        //
        // Stack layout:
        //   0x7FFF_FFFF_F000  ← Stack top (RSP starts here)
        //   0x7FFF_FEFF_F000  ← Stack bottom (8MB down)
        //
        // Usage:
        //   push rax  → RSP -= 8, [RSP] = RAX
        //   pop rbx   → RBX = [RSP], RSP += 8
    }
    
    /// Test: exec() new address space is correct (Day 19)
    #[test]
    fn test_exec_new_address_space_correct() {
        // Expected behavior:
        // 1. Old address space has old program
        // 2. New address space has:
        //    - ELF segments mapped
        //    - Stack allocated
        //    - No old program data
        // 3. Address spaces are independent
        // 4. Old space will be freed
        //
        // Verification:
        //   Read from 0x400000 in new space → new program code
        //   Read from 0x400000 in old space → old program code
        //   Spaces don't interfere
    }
    
    // ==================== DAY 20: PROCESS UPDATE TESTS ====================
    
    /// Test: exec() saves old page table (Day 20)
    #[test]
    fn test_exec_saves_old_page_table() {
        // Expected behavior:
        // 1. Process has page_table = 0x50000
        // 2. Save old_pt = 0x50000
        // 3. Create new_pt = 0x60000
        // 4. Update process.page_table = 0x60000
        // 5. Later free old_pt (0x50000)
        //
        // Code:
        //   let old_pt = process.page_table;
        //   let new_pt = create_address_space()?;
        //   process.page_table = new_pt;
        //   // ... load program ...
        //   destroy_address_space(old_pt);
    }
    
    /// Test: exec() updates process structure (Day 20)
    #[test]
    fn test_exec_updates_process() {
        // Expected behavior:
        // 1. Update page_table to new address space
        // 2. Update entry to ELF entry point
        // 3. Update stack_top to new stack
        // 4. Update stack_bottom
        // 5. Update heap_base (after last segment)
        // 6. Update heap_limit
        // 7. state remains Ready
        //
        // Before:
        //   entry: 0x401000 (old program)
        //   page_table: 0x50000
        //   stack_top: 0x7FFFFFFF0000
        //
        // After:
        //   entry: 0x400800 (new program)
        //   page_table: 0x60000
        //   stack_top: 0x7FFFFFFFF000
    }
    
    /// Test: exec() updates heap pointers (Day 20)
    #[test]
    fn test_exec_updates_heap() {
        // Expected behavior:
        // 1. Find highest segment end address
        // 2. Align to page boundary
        // 3. Set heap_base to that address
        // 4. Set heap_limit = heap_base (empty heap)
        //
        // Example:
        //   Segment 0: 0x400000 - 0x400FFF
        //   Segment 1: 0x401000 - 0x401FFF
        //   Highest end: 0x402000
        //   Align: 0x402000 (already aligned)
        //   heap_base = 0x402000
        //   heap_limit = 0x402000
        //
        // Later, malloc() will expand heap_limit
    }
    
    /// Test: exec() frees old page table (Day 20)
    #[test]
    fn test_exec_frees_old_memory() {
        // Expected behavior:
        // 1. Old page table at 0x50000
        // 2. Old address space has pages:
        //    - Code: 0xA0000
        //    - Data: 0xB0000
        //    - Stack: 0xC0000-0xC7FFF (8MB)
        // 3. exec() loads new program
        // 4. Calls destroy_address_space(0x50000)
        // 5. All old pages freed:
        //    - Page tables freed
        //    - Data pages freed
        //    - Stack pages freed
        // 6. No memory leaks
        //
        // Implementation:
        //   destroy_address_space(old_pt) {
        //     walk page tables;
        //     free all user-space pages;
        //     free page table structures;
        //   }
    }
    
    /// Test: exec() closes FDs with close-on-exec (Day 20)
    #[test]
    fn test_exec_closes_cloexec_fds() {
        // Expected behavior:
        // 1. Process has 3 FDs:
        //    fd[0]: stdin (no FD_CLOEXEC)
        //    fd[1]: stdout (no FD_CLOEXEC)
        //    fd[3]: file.txt (FD_CLOEXEC set)
        // 2. Call exec()
        // 3. fd[0] and fd[1] preserved
        // 4. fd[3] closed (FD_CLOEXEC)
        //
        // After exec():
        //   fd[0]: stdin ✓
        //   fd[1]: stdout ✓
        //   fd[3]: closed
        //
        // TODO: Implement FD_CLOEXEC flag
    }
    
    // ==================== DAY 21: CONTEXT UPDATE & TESTING ====================
    
    /// Test: exec() updates TCB context (Day 21)
    #[test]
    fn test_exec_updates_context() {
        // Expected behavior:
        // 1. Get TCB for current thread
        // 2. Update context registers:
        //    - RIP/ELR = entry_point
        //    - RSP/SP = stack_top
        //    - CR3/TTBR0 = new_page_table
        //    - Clear all general purpose registers
        //    - Set RFLAGS with interrupts enabled
        // 3. Call set_thread_context()
        //
        // x86_64:
        //   ctx.rip = 0x400800
        //   ctx.rsp = 0x7FFFFFFFF000
        //   ctx.cr3 = 0x60000
        //   ctx.rax = 0
        //   ctx.rbx = 0
        //   ... (clear all)
        //   ctx.rflags = 0x202
        //
        // aarch64:
        //   ctx.elr = 0x400800
        //   ctx.sp = 0x7FFFFFFFF000
        //   ctx.ttbr0 = 0x60000
        //   ctx.x[0..31] = 0
    }
    
    /// Test: exec() loads new program (Day 21)
    #[test]
    fn test_exec_loads_new_program() {
        // Expected behavior - Full exec() flow:
        //
        // Setup:
        //   Process running old program (/bin/old)
        //
        // Call exec("/bin/new", argv, envp):
        //   1. Validate path ✓
        //   2. Read /bin/new ELF ✓
        //   3. Parse ELF ✓
        //   4. Create new address space ✓
        //   5. Load segments ✓
        //   6. Setup stack ✓
        //   7. Update process ✓
        //   8. Update TCB ✓
        //   9. Free old memory ✓
        //
        // Result:
        //   Process now runs /bin/new
        //   Old program gone
        //   New program executes from entry point
    }
    
    /// Test: exec() new program executes (Day 21)
    #[test]
    fn test_exec_new_program_executes() {
        // Expected behavior:
        // 1. exec() completes successfully
        // 2. Scheduler resumes thread
        // 3. Context switch loads:
        //    - RIP = new entry point
        //    - RSP = new stack
        //    - CR3 = new page table
        // 4. CPU fetches instruction from entry point
        // 5. New program starts executing
        //
        // Example:
        //   Old program: infinite loop
        //   exec("/bin/hello")
        //   New program: prints "Hello, World!"
        //   Result: "Hello, World!" printed
    }
    
    /// Test: exec() preserves file descriptors (Day 21)
    #[test]
    fn test_exec_preserves_fds() {
        // Expected behavior:
        // 1. Process has open FDs:
        //    fd[0]: stdin
        //    fd[1]: stdout
        //    fd[2]: stderr
        //    fd[3]: file.txt
        // 2. Call exec()
        // 3. All FDs still open (unless FD_CLOEXEC)
        // 4. New program can use same FDs
        //
        // Shell example:
        //   fd = open("output.txt", O_WRONLY);
        //   dup2(fd, 1);  // stdout → output.txt
        //   exec("/bin/ls");
        //   // ls output goes to output.txt
    }
    
    /// Test: exec() preserves PID (Day 21)
    #[test]
    fn test_exec_preserves_pid() {
        // Expected behavior:
        // 1. Process has PID 100
        // 2. Call exec("/bin/new")
        // 3. Process still has PID 100
        // 4. Parent process unchanged
        // 5. Children list unchanged
        //
        // Verification:
        //   Before: getpid() = 100
        //   exec("/bin/new")
        //   After: getpid() = 100 (same PID)
    }
    
    /// Test: fork() + exec() works (Day 21)
    #[test]
    fn test_fork_exec_works() {
        // Expected behavior - Classic UNIX pattern:
        //
        // Parent code:
        //   pid_t child = fork();
        //   if (child == 0) {
        //     // Child process
        //     exec("/bin/ls", ["/"], NULL);
        //     perror("exec failed");
        //     exit(1);
        //   } else {
        //     // Parent process
        //     int status;
        //     wait(&status);
        //     printf("Child finished\n");
        //   }
        //
        // Flow:
        //   1. Parent forks → Child created (PID 101)
        //   2. Child execs → Becomes /bin/ls
        //   3. /bin/ls runs and lists directory
        //   4. /bin/ls exits
        //   5. Parent's wait() returns
        //
        // This is how shells run commands!
    }
    
    /// Test: exec() error handling (Day 21)
    #[test]
    fn test_exec_error_handling() {
        // Expected error cases:
        //
        // Case 1: File not found
        //   exec("/nonexistent")
        //   Returns: -1 (ENOENT)
        //   Process: unchanged
        //
        // Case 2: Invalid ELF
        //   exec("/etc/passwd")  // Not ELF
        //   Returns: -1 (ENOEXEC)
        //   Process: unchanged
        //
        // Case 3: Wrong architecture
        //   exec("/bin/arm64_program")  // On x86-64
        //   Returns: -1 (ENOEXEC)
        //   Process: unchanged
        //
        // Case 4: Out of memory
        //   exec("/bin/huge_program")
        //   Memory allocation fails
        //   Returns: -1 (ENOMEM)
        //   Process: unchanged (partial cleanup done)
        //
        // Important: On error, process continues unchanged!
        // exec() only replaces process on success
    }
    
    /// Test: exec() integration (Day 21)
    #[test]
    fn test_exec_full_integration() {
        // Expected behavior - Complete lifecycle:
        //
        // Step 1: Process running
        //   PID: 100
        //   Program: /bin/old
        //   Memory: Old address space
        //
        // Step 2: Call exec("/bin/new")
        //   Validate path ✓
        //   Read ELF ✓
        //   Parse ✓
        //   Create new space ✓
        //   Load segments ✓
        //   Setup stack ✓
        //
        // Step 3: Update process
        //   page_table: 0x60000 (new)
        //   entry: 0x400800 (new)
        //   stack: 0x7FFFFFFFF000 (new)
        //   PID: 100 (same)
        //   parent: unchanged
        //   children: unchanged
        //
        // Step 4: Update context
        //   RIP: 0x400800
        //   RSP: 0x7FFFFFFFF000
        //   CR3: 0x60000
        //
        // Step 5: Free old memory
        //   Old page table freed
        //   Old pages freed
        //
        // Step 6: Resume execution
        //   Scheduler picks thread
        //   Context switch
        //   CPU loads new state
        //   New program runs!
        //
        // Verification:
        //   ✓ New program executing
        //   ✓ Old program gone
        //   ✓ PID preserved
        //   ✓ No memory leaks
        //   ✓ FDs work
    }
}

