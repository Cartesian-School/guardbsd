// userland/libgbsd/src/ipc.rs
// IPC system call wrappers
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::error::{Error, Result};
use crate::syscall::{syscall0, syscall2, syscall3};

// Import IPC syscall numbers
include!("../../../shared/syscall_numbers.rs");

pub type PortId = u64;
pub type CapId = u64;

pub const IPC_FLAG_NONBLOCK: u64 = 1 << 0;
pub const IPC_FLAG_ZERO_COPY: u64 = 1 << 1;
pub const IPC_FLAG_DROP_OLDEST: u64 = 1 << 2;
pub const IPC_FLAG_REJECT_SENDER: u64 = 1 << 3;

/// # Errors
///
/// Returns error if IPC port creation fails
#[inline]
pub fn port_create() -> Result<PortId> {
    let ret = unsafe { syscall0(SYS_IPC_PORT_CREATE as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as i32))
    } else {
        Ok(ret)
    }
}

/// # Errors
///
/// Returns error if port destruction fails
#[inline]
pub fn port_destroy(_port: PortId) -> Result<()> {
    // Not yet implemented in kernel
    Err(Error::NoSys)
}

/// # Errors
///
/// Returns error if IPC send fails
#[inline]
pub fn ipc_send(port: PortId, buffer: *const u8, length: usize, _flags: u64) -> Result<()> {
    let ret = unsafe { syscall3(SYS_IPC_SEND as u64, port, buffer as u64, length as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as i32))
    } else {
        Ok(())
    }
}

/// # Errors
///
/// Returns error if IPC receive fails
#[inline]
pub fn ipc_recv(port: PortId, buffer: *mut u8, length: usize, _flags: u64) -> Result<u64> {
    let ret = unsafe { syscall3(SYS_IPC_RECV as u64, port, buffer as u64, length as u64) };
    let ret_i64 = ret as i64;
    if ret_i64 < 0 {
        Err(Error::from_code((-ret_i64) as i32))
    } else {
        Ok(ret)
    }
}

/// # Errors
///
/// Returns error if IPC send fails
#[inline]
pub fn port_send(port: PortId, buffer: *const u8, length: usize) -> Result<()> {
    ipc_send(port, buffer, length, 0)
}

/// # Errors
///
/// Returns error if IPC receive fails
#[inline]
pub fn port_receive(port: PortId, buffer: *mut u8, length: usize) -> Result<u64> {
    ipc_recv(port, buffer, length, 0)
}

/// # Errors
///
/// Always returns `Error::NoSys` as capability management is not implemented.
#[inline]
pub fn cap_grant(_target_tid: u64, _cap: CapId) -> Result<()> {
    Err(Error::NoSys)
}

/// # Errors
///
/// Always returns `Error::NoSys` as capability management is not implemented.
#[inline]
pub fn cap_revoke(_cap: CapId) -> Result<()> {
    Err(Error::NoSys)
}

/// # Errors
///
/// Always returns `Error::NoSys` as capability management is not implemented.
#[inline]
pub fn cap_delegate(_cap: CapId, _rights: u64) -> Result<CapId> {
    Err(Error::NoSys)
}

/// # Errors
///
/// Always returns `Error::NoSys` as capability management is not implemented.
#[inline]
pub fn cap_copy(_cap: CapId) -> Result<CapId> {
    Err(Error::NoSys)
}
