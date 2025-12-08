// servers/ramfs/src/ops.rs
// RAM filesystem operations
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::node::*;
use gbsd::*;

pub fn parse_path(path: &[u8]) -> (&[u8], &[u8]) {
    let mut end = 0;
    while end < path.len() && path[end] != 0 {
        end += 1;
    }
    let path = &path[..end];

    if path.is_empty() || path[0] != b'/' {
        return (b"", b"");
    }

    if path.len() == 1 {
        return (b"/", b"");
    }

    let mut last_slash = 0;
    for i in 1..path.len() {
        if path[i] == b'/' {
            last_slash = i;
        }
    }

    if last_slash == 0 {
        (&path[..1], &path[1..])
    } else {
        (&path[..last_slash], &path[last_slash + 1..])
    }
}

pub fn open(fs: &mut RamFs, path: &[u8], flags: u32) -> i64 {
    let (_, name) = parse_path(path);

    if name.is_empty() {
        return 0; // Root directory
    }

    // Extract filename string safely
    let name_str = core::str::from_utf8(name).unwrap_or("<invalid>");

    if let Some(idx) = fs.find(0, name) {
        return idx as i64;
    }

    // Create if O_CREAT (0x200)
    if flags & 0x200 != 0 {
        if let Some(idx) = fs.create(0, name, NodeType::File) {
            klog_info!("ramfs", "create file '{}'", name_str);
            return idx as i64;
        } else {
            klog_error!("ramfs", "ramfs out of space (requested=1)");
            return -28; // ENOSPC
        }
    }

    klog_warn!("ramfs", "file '{}' not found", name_str);
    -2 // ENOENT
}

pub fn read(fs: &mut RamFs, fd: u32, buf: &mut [u8]) -> i64 {
    if let Some(node) = fs.get(fd) {
        match node.node_type {
            NodeType::File => {
                let len = node.size.min(buf.len());
                buf[..len].copy_from_slice(&node.data[..len]);
                klog_info!("ramfs", "read {} bytes from fd={}", len, fd);
                len as i64
            }
            NodeType::Device => {
                // Device nodes should be handled by VFS → devd
                // RAMFS just returns the dev_id as metadata
                klog_warn!("ramfs", "device node read should go through VFS");
                -22 // EINVAL
            }
            NodeType::Directory => {
                -21 // EISDIR
            }
        }
    } else {
        klog_warn!("ramfs", "invalid file descriptor {}", fd);
        -9 // EBADF
    }
}

pub fn write(fs: &mut RamFs, fd: u32, buf: &[u8]) -> i64 {
    if let Some(node) = fs.get(fd) {
        match node.node_type {
            NodeType::File => {
                let len = buf.len().min(4096);
                node.data[..len].copy_from_slice(&buf[..len]);
                node.size = len;
                klog_info!("ramfs", "write {} bytes to fd={}", len, fd);
                len as i64
            }
            NodeType::Device => {
                // Device nodes should be handled by VFS → devd
                klog_warn!("ramfs", "device node write should go through VFS");
                -22 // EINVAL
            }
            NodeType::Directory => {
                -21 // EISDIR
            }
        }
    } else {
        klog_warn!("ramfs", "invalid file descriptor {}", fd);
        -9 // EBADF
    }
}

pub fn mkdir(fs: &mut RamFs, path: &[u8]) -> i64 {
    let (_, name) = parse_path(path);

    if name.is_empty() {
        return -17; // EEXIST
    }

    let name_str = core::str::from_utf8(name).unwrap_or("<invalid>");

    if fs.find(0, name).is_some() {
        return -17; // EEXIST
    }

    if let Some(idx) = fs.create(0, name, NodeType::Directory) {
        klog_info!("ramfs", "create directory '{}'", name_str);
        return idx as i64;
    }

    klog_error!("ramfs", "ramfs out of space (requested=1)");
    -28 // ENOSPC
}

pub fn unlink(fs: &mut RamFs, path: &[u8]) -> i64 {
    let (_, name) = parse_path(path);

    if name.is_empty() {
        return -1; // EPERM
    }

    let name_str = core::str::from_utf8(name).unwrap_or("<invalid>");

    if let Some(idx) = fs.find(0, name) {
        if fs.delete(idx) {
            klog_info!("ramfs", "delete file '{}'", name_str);
            return 0;
        }
    }

    klog_warn!("ramfs", "file '{}' not found", name_str);
    -2 // ENOENT
}

pub fn mknod(fs: &mut RamFs, path: &[u8], dev_id: u32) -> i64 {
    let (_, name) = parse_path(path);

    if name.is_empty() {
        return -22; // EINVAL
    }

    let name_str = core::str::from_utf8(name).unwrap_or("<invalid>");

    if fs.find(0, name).is_some() {
        return -17; // EEXIST
    }

    if let Some(idx) = fs.create_device(0, name, dev_id) {
        klog_info!("ramfs", "create device node '{}' dev_id={}", name_str, dev_id);
        return idx as i64;
    }

    klog_error!("ramfs", "ramfs out of space (requested=1)");
    -28 // ENOSPC
}
