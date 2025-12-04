// servers/vfs/src/ops.rs
// VFS operations
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::vnode::*;
use gbsd::*;

#[repr(u32)]
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
}

// VFS operation processing with logging
pub fn process_vfs_request(req: &VfsRequest) -> VfsResponse {
    // Extract path string safely
    let path_len = req.path.iter().position(|&c| c == 0).unwrap_or(req.path.len());
    let path = core::str::from_utf8(&req.path[..path_len]).unwrap_or("<invalid>");

    match req.op {
        VfsOp::Open => {
            klog_warn!("vfs", "unimplemented opcode={} for path={}", 1, path);
            VfsResponse::err(1) // ENOSYS
        }
        VfsOp::Close => {
            klog_warn!("vfs", "unimplemented opcode={} for path={}", 2, path);
            VfsResponse::err(1)
        }
        VfsOp::Read => {
            klog_warn!("vfs", "unimplemented opcode={} for path={}", 3, path);
            VfsResponse::err(1)
        }
        VfsOp::Write => {
            klog_warn!("vfs", "unimplemented opcode={} for path={}", 4, path);
            VfsResponse::err(1)
        }
        VfsOp::Stat => {
            klog_warn!("vfs", "unimplemented opcode={} for path={}", 5, path);
            VfsResponse::err(1)
        }
        VfsOp::Mkdir => {
            klog_warn!("vfs", "unimplemented opcode={} for path={}", 6, path);
            VfsResponse::err(1)
        }
        VfsOp::Rmdir => {
            klog_warn!("vfs", "unimplemented opcode={} for path={}", 7, path);
            VfsResponse::err(1)
        }
        VfsOp::Unlink => {
            klog_warn!("vfs", "unimplemented opcode={} for path={}", 8, path);
            VfsResponse::err(1)
        }
    }
}
