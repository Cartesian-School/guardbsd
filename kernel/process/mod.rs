// Process Module
// BSD 3-Clause License

#![no_std]

pub mod elf_loader;
pub mod process;

pub use process::{Process, Pid, ProcessState, Registers};
pub use process::{create_process, exec, schedule, switch_to, get_current};

extern "C" {
    pub fn context_switch(old: *mut Registers, new: *const Registers);
    pub fn jump_to_user(entry: u64, stack: u64) -> !;
}
