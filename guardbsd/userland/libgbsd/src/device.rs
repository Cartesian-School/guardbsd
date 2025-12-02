// userland/libgbsd/src/device.rs
// Device management syscall wrappers
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::error::{Error, Result};

pub type DevId = u32;

pub const DEV_CHAR: u32 = 0;
pub const DEV_BLOCK: u32 = 1;
pub const DEV_NET: u32 = 2;

#[repr(C)]
pub struct DevRequest {
    pub op: u32,
    pub dev_id: u32,
    pub major: u16,
    pub minor: u16,
    pub flags: u32,
}

impl DevRequest {
    pub const fn new(op: u32, dev_id: u32, major: u16, minor: u16, flags: u32) -> Self {
        Self {
            op,
            dev_id,
            major,
            minor,
            flags,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}

#[inline]
pub fn dev_register(dev_type: u32, major: u16, minor: u16) -> Result<DevId> {
    let req = DevRequest::new(1, 0, major, minor, dev_type);
    // Future: IPC call to devd
    let _ = req;
    Ok(0)
}

#[inline]
pub fn dev_unregister(dev_id: DevId) -> Result<()> {
    let req = DevRequest::new(2, dev_id, 0, 0, 0);
    // Future: IPC call to devd
    let _ = req;
    Ok(())
}

#[inline]
pub fn dev_open(dev_id: DevId) -> Result<DevId> {
    let req = DevRequest::new(3, dev_id, 0, 0, 0);
    // Future: IPC call to devd
    let _ = req;
    Ok(dev_id)
}

#[inline]
pub fn dev_close(dev_id: DevId) -> Result<()> {
    let req = DevRequest::new(4, dev_id, 0, 0, 0);
    // Future: IPC call to devd
    let _ = req;
    Ok(())
}
