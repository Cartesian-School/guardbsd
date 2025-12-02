// servers/ramfs/src/ops.rs
// RAM filesystem operations
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::node::*;

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
    
    if let Some(idx) = fs.find(0, name) {
        return idx as i64;
    }
    
    // Create if O_CREAT (0x200)
    if flags & 0x200 != 0 {
        if let Some(idx) = fs.create(0, name, NodeType::File) {
            return idx as i64;
        }
    }
    
    -2 // ENOENT
}

pub fn read(fs: &mut RamFs, fd: u32, buf: &mut [u8]) -> i64 {
    if let Some(node) = fs.get(fd) {
        if node.node_type != NodeType::File {
            return -21; // EISDIR
        }
        let len = node.size.min(buf.len());
        buf[..len].copy_from_slice(&node.data[..len]);
        return len as i64;
    }
    -9 // EBADF
}

pub fn write(fs: &mut RamFs, fd: u32, buf: &[u8]) -> i64 {
    if let Some(node) = fs.get(fd) {
        if node.node_type != NodeType::File {
            return -21; // EISDIR
        }
        let len = buf.len().min(4096);
        node.data[..len].copy_from_slice(&buf[..len]);
        node.size = len;
        return len as i64;
    }
    -9 // EBADF
}

pub fn mkdir(fs: &mut RamFs, path: &[u8]) -> i64 {
    let (_, name) = parse_path(path);
    
    if name.is_empty() {
        return -17; // EEXIST
    }
    
    if fs.find(0, name).is_some() {
        return -17; // EEXIST
    }
    
    if let Some(idx) = fs.create(0, name, NodeType::Directory) {
        return idx as i64;
    }
    
    -28 // ENOSPC
}

pub fn unlink(fs: &mut RamFs, path: &[u8]) -> i64 {
    let (_, name) = parse_path(path);
    
    if name.is_empty() {
        return -1; // EPERM
    }
    
    if let Some(idx) = fs.find(0, name) {
        if fs.delete(idx) {
            return 0;
        }
    }
    
    -2 // ENOENT
}
