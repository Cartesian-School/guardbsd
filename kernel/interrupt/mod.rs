// Interrupt Module (legacy x86 only)
// BSD 3-Clause License

#![cfg(feature = "x86_legacy")]

pub mod idt;

use crate::syscall::syscall_handler;

#[no_mangle]
pub extern "C" fn syscall_dispatch(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    syscall_handler(syscall_num, arg1, arg2, arg3)
}
