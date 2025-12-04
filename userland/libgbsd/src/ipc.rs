// userland/libgbsd/src/ipc.rs
// IPC system call wrappers
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::error::{Error, Result};
use crate::syscall::*;

pub type PortId = u64;
pub type CapId = u64;

pub const IPC_FLAG_NONBLOCK: u64 = 1 << 0;
pub const IPC_FLAG_ZERO_COPY: u64 = 1 << 1;
pub const IPC_FLAG_DROP_OLDEST: u64 = 1 << 2;
pub const IPC_FLAG_REJECT_SENDER: u64 = 1 << 3;

#[inline]
pub fn port_create() -> Result<PortId> {
    let ret = unsafe { syscall1(SYS_PORT_CREATE, 0) };
    decode_result(ret).map(|v| v as PortId)
}

#[inline]
pub fn port_destroy(port: PortId) -> Result<()> {
    let ret = unsafe { syscall1(SYS_PORT_DESTROY, port) };
    decode_result(ret).map(|_| ())
}

#[inline]
pub fn ipc_send(port: PortId, buffer: *const u8, length: usize, flags: u64) -> Result<()> {
    let ret = unsafe { syscall4(SYS_PORT_SEND, port, buffer as u64, length as u64, flags) };
    decode_result(ret).map(|_| ())
}

#[inline]
pub fn ipc_recv(port: PortId, buffer: *mut u8, length: usize, flags: u64) -> Result<u64> {
    let ret = unsafe { syscall4(SYS_PORT_RECEIVE, port, buffer as u64, length as u64, flags) };
    decode_result(ret)
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
pub fn cap_grant(target_tid: u64, cap: CapId) -> Result<()> {
    let ret = unsafe { syscall2(SYS_CAP_GRANT, target_tid, cap) };
    decode_result(ret).map(|_| ())
}

#[inline]
pub fn cap_revoke(cap: CapId) -> Result<()> {
    let ret = unsafe { syscall1(SYS_CAP_REVOKE, cap) };
    decode_result(ret).map(|_| ())
}

#[inline]
pub fn cap_delegate(cap: CapId, rights: u64) -> Result<CapId> {
    let ret = unsafe { syscall2(SYS_CAP_DELEGATE, cap, rights) };
    decode_result(ret).map(|v| v as CapId)
}

#[inline]
pub fn cap_copy(cap: CapId) -> Result<CapId> {
    let ret = unsafe { syscall1(SYS_CAP_COPY, cap) };
    decode_result(ret).map(|v| v as CapId)
}

fn decode_result(ret: u64) -> Result<u64> {
    if ret >= 0xFFFF_FFFF_0000_0000 {
        Err(Error::from_code(ret))
    } else {
        Ok(ret)
    }
}
