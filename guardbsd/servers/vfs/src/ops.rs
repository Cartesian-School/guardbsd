// servers/vfs/src/ops.rs
// VFS operations
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::vnode::*;

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
