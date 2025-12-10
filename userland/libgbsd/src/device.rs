// userland/libgbsd/src/device.rs
// Device management IPC wrappers
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::error::{Error, Result};
use crate::ipc::{port_receive, port_send};

pub type DevId = u32;

pub const DEV_CHAR: u32 = 0;
pub const DEV_BLOCK: u32 = 1;
pub const DEV_NET: u32 = 2;

// Fixed IPC port for devd server
// TODO: Replace with service discovery in future
pub const DEVD_PORT: u64 = 1100;

#[repr(C)]
pub struct DevRequest {
    pub op: u32,
    pub dev_id: u32,
    pub major: u16,
    pub minor: u16,
    pub flags: u32,
}

impl DevRequest {
    #[must_use]
    pub const fn new(op: u32, dev_id: u32, major: u16, minor: u16, flags: u32) -> Self {
        Self {
            op,
            dev_id,
            major,
            minor,
            flags,
        }
    }

    #[must_use]
    #[allow(clippy::ptr_as_ptr, clippy::ref_as_ptr)]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }

    fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&self.op.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.dev_id.to_le_bytes());
        bytes[8..10].copy_from_slice(&self.major.to_le_bytes());
        bytes[10..12].copy_from_slice(&self.minor.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.flags.to_le_bytes());
        bytes
    }
}

// Helper function to send request and receive response from devd
fn devd_call(req: &DevRequest) -> Result<DevId> {
    let req_bytes = req.to_bytes();

    // Send request to devd
    port_send(DEVD_PORT, req_bytes.as_ptr(), req_bytes.len())?;

    // Receive response
    let mut resp_buf = [0u8; 8];
    port_receive(DEVD_PORT, resp_buf.as_mut_ptr(), resp_buf.len())?;

    // Parse response (i64 result)
    let result = i64::from_le_bytes(resp_buf);

    if result < 0 {
        Err(Error::from_code((-result) as i32))
    } else {
        Ok(result as DevId)
    }
}

/// # Errors
///
/// Returns error if device registration fails or devd is not available
#[inline]
pub fn dev_register(dev_type: u32, major: u16, minor: u16) -> Result<DevId> {
    let req = DevRequest::new(1, 0, major, minor, dev_type);
    devd_call(&req)
}

/// # Errors
///
/// Returns error if device unregistration fails or devd is not available
#[inline]
pub fn dev_unregister(dev_id: DevId) -> Result<()> {
    let req = DevRequest::new(2, dev_id, 0, 0, 0);
    devd_call(&req)?;
    Ok(())
}

/// # Errors
///
/// Returns error if device open fails or devd is not available
#[inline]
pub fn dev_open(dev_id: DevId) -> Result<DevId> {
    let req = DevRequest::new(3, dev_id, 0, 0, 0);
    devd_call(&req)
}

/// # Errors
///
/// Returns error if device close fails or devd is not available
#[inline]
pub fn dev_close(dev_id: DevId) -> Result<()> {
    let req = DevRequest::new(4, dev_id, 0, 0, 0);
    devd_call(&req)?;
    Ok(())
}

/// # Errors
///
/// Returns error if device read fails or devd is not available
#[inline]
pub fn dev_read(dev_id: DevId, _buffer: &mut [u8]) -> Result<usize> {
    let req = DevRequest::new(5, dev_id, 0, 0, 0);
    let bytes_read = devd_call(&req)?;
    Ok(bytes_read as usize)
}

/// # Errors
///
/// Returns error if device write fails or devd is not available
#[inline]
pub fn dev_write(dev_id: DevId, buffer: &[u8]) -> Result<usize> {
    let req = DevRequest::new(6, dev_id, 0, 0, buffer.len() as u32);
    let bytes_written = devd_call(&req)?;
    Ok(bytes_written as usize)
}

/// # Errors
///
/// Returns error if ioctl fails or devd is not available
#[inline]
pub fn dev_ioctl(dev_id: DevId, cmd: u32, arg: u64) -> Result<u64> {
    let req = DevRequest::new(7, dev_id, (cmd >> 16) as u16, cmd as u16, arg as u32);
    let result = devd_call(&req)?;
    Ok(result as u64)
}
