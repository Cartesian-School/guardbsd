// servers/vfs/src/ops.rs
// VFS operations
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::vnode::*;
use gbsd::*;

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum VfsOp {
    Open = 1,
    Close = 2,
    Read = 3,
    Write = 4,
    Stat = 5,
    Mkdir = 6,
    Rmdir = 7,
    Unlink = 8,
}

pub struct VfsRequest {
    pub op: VfsOp,
    pub path: [u8; 256],
    pub flags: u32,
    pub mode: u32,
}

pub struct VfsResponse {
    pub result: i64,
    pub data_len: u32,
}

impl VfsRequest {
    pub const fn new() -> Self {
        Self {
            op: VfsOp::Open,
            path: [0; 256],
            flags: 0,
            mode: 0,
        }
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        if data.len() < 8 {
            return Self::new();
        }

        let op = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let op = match op {
            1 => VfsOp::Open,
            2 => VfsOp::Close,
            3 => VfsOp::Read,
            4 => VfsOp::Write,
            5 => VfsOp::Stat,
            6 => VfsOp::Mkdir,
            7 => VfsOp::Rmdir,
            8 => VfsOp::Unlink,
            _ => VfsOp::Open,
        };

        let mut path = [0u8; 256];
        let path_start = 8;
        let path_len = (data.len() - path_start).min(256);
        path[..path_len].copy_from_slice(&data[path_start..path_start + path_len]);

        let flags = if data.len() >= 264 {
            u32::from_le_bytes([data[256], data[257], data[258], data[259]])
        } else {
            0
        };

        let mode = if data.len() >= 268 {
            u32::from_le_bytes([data[260], data[261], data[262], data[263]])
        } else {
            0
        };

        Self {
            op,
            path,
            flags,
            mode,
        }
    }
}

impl VfsResponse {
    pub const fn new(result: i64) -> Self {
        Self {
            result,
            data_len: 0,
        }
    }

    pub const fn ok(fd: u64) -> Self {
        Self {
            result: fd as i64,
            data_len: 0,
        }
    }

    pub const fn err(code: i64) -> Self {
        Self {
            result: -code,
            data_len: 0,
        }
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&self.result.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.data_len.to_le_bytes());
        bytes
    }
}

// VFS operation processing - routes to appropriate filesystem servers
pub fn process_vfs_request(
    req: &VfsRequest,
    mounts: &mut crate::MountTable,
    vfs_port: usize,
) -> VfsResponse {
    // Extract path string safely
    let path_len = req
        .path
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(req.path.len());
    let path = core::str::from_utf8(&req.path[..path_len]).unwrap_or("<invalid>");

    // Find the appropriate mount point
    if let Some(mount) = mounts.find_mount(path) {
        match mount.mount_type {
            crate::MountType::RamFs => {
                // Forward to RAMFS server via IPC
                return forward_to_ramfs(req, mount.port, vfs_port);
            }
            crate::MountType::DevFs => {
                // Forward to device filesystem
                return VfsResponse::err(1); // Not implemented yet
            }
        }
    } else {
        klog_warn!("vfs", "no mount point found for path={}", path);
        return VfsResponse::err(2); // ENOENT
    }
}

fn forward_to_ramfs(req: &VfsRequest, ramfs_port: usize, vfs_port: usize) -> VfsResponse {
    use gbsd::*;

    if ramfs_port == 0 {
        klog_warn!("vfs", "RAMFS port not configured");
        return VfsResponse::err(19); // ENODEV
    }

    // Prepare IPC message to RAMFS
    // Format: [op:u32][reply_port:u32][path:256][flags:u32][mode:u32]
    let mut ipc_buf = [0u8; 512];

    // Copy operation code
    ipc_buf[0..4].copy_from_slice(&(req.op as u32).to_le_bytes());

    // Copy reply-to port (VFS port where RAMFS should send response)
    ipc_buf[4..8].copy_from_slice(&(vfs_port as u32).to_le_bytes());

    // Copy path
    let path_len = req
        .path
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(req.path.len());
    ipc_buf[8..8 + path_len].copy_from_slice(&req.path[..path_len]);

    // Copy flags and mode if needed
    ipc_buf[256..260].copy_from_slice(&req.flags.to_le_bytes());
    ipc_buf[260..264].copy_from_slice(&req.mode.to_le_bytes());

    // Send to RAMFS server
    if port_send(ramfs_port as u64, ipc_buf.as_ptr(), 512).is_ok() {
        // Wait for response
        let mut resp_buf = [0u8; 512];
        if port_receive(vfs_port as u64, resp_buf.as_mut_ptr(), 512).is_ok() {
            // Parse response
            let result = i64::from_le_bytes([
                resp_buf[0],
                resp_buf[1],
                resp_buf[2],
                resp_buf[3],
                resp_buf[4],
                resp_buf[5],
                resp_buf[6],
                resp_buf[7],
            ]);

            if result >= 0 {
                // Check if this is a device node by checking path prefix
                let path_str = core::str::from_utf8(&req.path[..path_len]).unwrap_or("");
                if path_str.starts_with("/dev/") && matches!(req.op, VfsOp::Open) {
                    // This is a device node - result is node index
                    // We need to query RAMFS for the dev_id
                    // For now, return a special marker that indicates device
                    // The node index will encode the device info
                    VfsResponse::ok(result as u64)
                } else {
                    VfsResponse::ok(result as u64)
                }
            } else {
                VfsResponse::err(-result)
            }
        } else {
            klog_error!("vfs", "failed to receive response from RAMFS");
            VfsResponse::err(5) // EIO
        }
    } else {
        klog_error!("vfs", "failed to send request to RAMFS");
        VfsResponse::err(5) // EIO
    }
}

// Helper to create device node in RAMFS
pub fn create_device_node(ramfs_port: usize, vfs_port: usize, path: &[u8], dev_id: u32) -> VfsResponse {
    use gbsd::*;

    if ramfs_port == 0 {
        return VfsResponse::err(19); // ENODEV
    }

    // Prepare mknod IPC message to RAMFS
    // Op 9 = mknod, Format: [op:u32][reply_port:u32][path:256][dev_id:u32]
    let mut ipc_buf = [0u8; 512];

    ipc_buf[0..4].copy_from_slice(&9u32.to_le_bytes()); // mknod op
    ipc_buf[4..8].copy_from_slice(&(vfs_port as u32).to_le_bytes());

    let path_len = path.iter().position(|&c| c == 0).unwrap_or(path.len()).min(256);
    ipc_buf[8..8 + path_len].copy_from_slice(&path[..path_len]);

    ipc_buf[264..268].copy_from_slice(&dev_id.to_le_bytes());

    if port_send(ramfs_port as u64, ipc_buf.as_ptr(), 512).is_ok() {
        let mut resp_buf = [0u8; 512];
        if port_receive(vfs_port as u64, resp_buf.as_mut_ptr(), 512).is_ok() {
            let result = i64::from_le_bytes([
                resp_buf[0], resp_buf[1], resp_buf[2], resp_buf[3],
                resp_buf[4], resp_buf[5], resp_buf[6], resp_buf[7],
            ]);

            if result >= 0 {
                VfsResponse::ok(result as u64)
            } else {
                VfsResponse::err(-result)
            }
        } else {
            VfsResponse::err(5) // EIO
        }
    } else {
        VfsResponse::err(5) // EIO
    }
}
