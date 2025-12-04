// userland/libgbsd/src/ipc.rs
// IPC system call wrappers
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::error::{Error, Result};

pub type PortId = u64;
pub type CapId = u64;

pub const IPC_FLAG_NONBLOCK: u64 = 1 << 0;
pub const IPC_FLAG_ZERO_COPY: u64 = 1 << 1;
pub const IPC_FLAG_DROP_OLDEST: u64 = 1 << 2;
pub const IPC_FLAG_REJECT_SENDER: u64 = 1 << 3;

// IPC syscalls are not implemented in the current kernel.
// These are placeholder implementations that will be replaced
// when IPC syscalls are added to the kernel.

#[inline]
pub fn port_create() -> Result<PortId> {
    Err(Error::NoSys)
}

#[inline]
pub fn port_destroy(_port: PortId) -> Result<()> {
    Err(Error::NoSys)
}

#[inline]
pub fn ipc_send(_port: PortId, _buffer: *const u8, _length: usize, _flags: u64) -> Result<()> {
    Err(Error::NoSys)
}

#[inline]
pub fn ipc_recv(_port: PortId, _buffer: *mut u8, _length: usize, _flags: u64) -> Result<u64> {
    Err(Error::NoSys)
}

#[inline]
pub fn port_send(port: PortId, buffer: *const u8, length: usize) -> Result<()> {
    ipc_send(port, buffer, length, 0)
}

#[inline]
pub fn port_receive(port: PortId, buffer: *mut u8, length: usize) -> Result<u64> {
    ipc_recv(port, buffer, length, 0)
}

#[inline]
pub fn cap_grant(_target_tid: u64, _cap: CapId) -> Result<()> {
    Err(Error::NoSys)
}

#[inline]
pub fn cap_revoke(_cap: CapId) -> Result<()> {
    Err(Error::NoSys)
}

#[inline]
pub fn cap_delegate(_cap: CapId, _rights: u64) -> Result<CapId> {
    Err(Error::NoSys)
}

#[inline]
pub fn cap_copy(_cap: CapId) -> Result<CapId> {
    Err(Error::NoSys)
}
