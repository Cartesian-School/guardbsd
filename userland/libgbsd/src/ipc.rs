// userland/libgbsd/src/ipc.rs
// IPC system call wrappers
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::syscall::*;
use crate::error::{Error, Result};

pub type PortId = u64;
pub type CapId = u64;

#[inline]
pub fn port_create() -> Result<PortId> {
    let ret = unsafe { syscall1(SYS_PORT_CREATE, 0) };
    if ret == 0 {
        Err(Error::NoMemory)
    } else {
        Ok(ret)
    }
}

#[inline]
pub fn port_destroy(port: PortId) -> Result<()> {
    let ret = unsafe { syscall1(SYS_PORT_DESTROY, port) };
    if ret == 0 {
        Ok(())
    } else {
        Err(Error::from_code(ret))
    }
}

#[inline]
pub fn port_send(port: PortId, msg_ptr: u64) -> Result<()> {
    let ret = unsafe { syscall2(SYS_PORT_SEND, port, msg_ptr) };
    if ret == 0 {
        Ok(())
    } else {
        Err(Error::from_code(ret))
    }
}

#[inline]
pub fn port_receive(port: PortId, msg_ptr: u64) -> Result<()> {
    let ret = unsafe { syscall2(SYS_PORT_RECEIVE, port, msg_ptr) };
    if ret == 0 {
        Ok(())
    } else {
        Err(Error::from_code(ret))
    }
}

#[inline]
pub fn port_call(port: PortId, req_ptr: u64, resp_ptr: u64) -> Result<()> {
    let ret = unsafe { syscall3(SYS_PORT_CALL, port, req_ptr, resp_ptr) };
    if ret == 0 {
        Ok(())
    } else {
        Err(Error::from_code(ret))
    }
}

#[inline]
pub fn port_sync_call(port: PortId, req_ptr: u64, resp_ptr: u64) -> Result<()> {
    let ret = unsafe { syscall3(SYS_PORT_SYNC_CALL, port, req_ptr, resp_ptr) };
    if ret == 0 {
        Ok(())
    } else {
        Err(Error::from_code(ret))
    }
}

#[inline]
pub fn port_reply(sender_tid: u64, msg_ptr: u64) -> Result<()> {
    let ret = unsafe { syscall2(SYS_PORT_REPLY, sender_tid, msg_ptr) };
    if ret == 0 {
        Ok(())
    } else {
        Err(Error::from_code(ret))
    }
}

#[inline]
pub fn cap_grant(target_tid: u64, cap: CapId) -> Result<()> {
    let ret = unsafe { syscall2(SYS_CAP_GRANT, target_tid, cap) };
    if ret == 0 {
        Ok(())
    } else {
        Err(Error::from_code(ret))
    }
}

#[inline]
pub fn cap_revoke(cap: CapId) -> Result<()> {
    let ret = unsafe { syscall1(SYS_CAP_REVOKE, cap) };
    if ret == 0 {
        Ok(())
    } else {
        Err(Error::from_code(ret))
    }
}

#[inline]
pub fn cap_delegate(cap: CapId, rights: u64) -> Result<CapId> {
    let ret = unsafe { syscall2(SYS_CAP_DELEGATE, cap, rights) };
    if ret == 0 {
        Err(Error::NoRights)
    } else {
        Ok(ret)
    }
}

#[inline]
pub fn cap_copy(cap: CapId) -> Result<CapId> {
    let ret = unsafe { syscall1(SYS_CAP_COPY, cap) };
    if ret == 0 {
        Err(Error::CapInvalid)
    } else {
        Ok(ret)
    }
}
