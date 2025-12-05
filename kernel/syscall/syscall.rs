// Syscall Interface - Full Implementation
// BSD 3-Clause License

#![no_std]

// Import canonical syscall numbers from shared module
include!("../../shared/syscall_numbers.rs");

// Import process syscall implementations
use crate::syscalls::process;

pub fn syscall_handler(syscall_num: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    match syscall_num {
        SYS_EXIT => {
            // sys_exit never returns, so we call it directly
            process::sys_exit(arg1 as i32);
        },
        SYS_GETPID => process::sys_getpid(),
        SYS_FORK => process::sys_fork(),
        SYS_EXEC => process::sys_exec(arg1 as *const u8, arg2 as *const *const u8),
        SYS_WAIT => process::sys_wait(arg1 as *mut i32),
        SYS_WRITE => sys_write(arg1, arg2 as *const u8, arg3),
        SYS_READ => sys_read(arg1, arg2 as *mut u8, arg3),
        SYS_OPEN => sys_open(arg1 as *const u8, arg2),
        SYS_CLOSE => sys_close(arg1),
        SYS_STAT => sys_stat(arg1 as *const u8, arg2 as *mut u8),
        SYS_MKDIR => sys_mkdir(arg1 as *const u8, arg2),
        SYS_UNLINK => sys_unlink(arg1 as *const u8),
        SYS_RENAME => sys_rename(arg1 as *const u8, arg2 as *const u8),
        SYS_SYNC => sys_sync(arg1),
        SYS_CHDIR => sys_chdir(arg1 as *const u8),
        SYS_GETCWD => sys_getcwd(arg1 as *mut u8, arg2),
        SYS_MOUNT => sys_mount(arg1 as *const u8, arg2 as *const u8, arg3 as *const u8),
        SYS_UMOUNT => sys_umount(arg1 as *const u8),
        _ => -1,
    }
}

fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    if fd == 1 || fd == 2 {
        // stdout/stderr - write to serial
        unsafe {
            let slice = core::slice::from_raw_parts(buf, len);
            for &byte in slice {
                serial_putc(byte);
            }
        }
        len as isize
    } else {
        -1
    }
}

// File descriptor management
const MAX_FDS: usize = 256;
static mut OPEN_FDS: [Option<FileDescriptor>; MAX_FDS] = [None; MAX_FDS];
static mut NEXT_FD: usize = 3; // 0,1,2 reserved for stdin/stdout/stderr

#[derive(Clone, Copy)]
struct FileDescriptor {
    inode: u64,
    offset: u64,
    flags: u32,
}

fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    unsafe {
        if fd >= MAX_FDS || OPEN_FDS[fd].is_none() {
            return -9; // EBADF
        }

        let fd_info = OPEN_FDS[fd].unwrap();
        let read_result = vfs_read(fd_info.inode, buf, len, fd_info.offset);

        if read_result > 0 {
            OPEN_FDS[fd].as_mut().unwrap().offset += read_result as u64;
        }

        read_result
    }
}

const COM1: u16 = 0x3F8;

unsafe fn serial_putc(c: u8) {
    while (inb(COM1 + 5) & 0x20) == 0 {}
    outb(COM1, c);
}

unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val);
}

unsafe fn inb(port: u16) -> u8 {
    let ret: u8;
    core::arch::asm!("in al, dx", out("al") ret, in("dx") port);
    ret
}

// VFS Interface Functions (will be implemented by filesystem servers)
fn vfs_open(path: *const u8, flags: usize) -> isize {
    // For now, directly call RAMFS operations
    // In the future, this will use IPC to VFS server

    // Simple path handling - assume all paths go to RAMFS for now
    unsafe {
        // Convert path to string
        let mut path_buf = [0u8; 256];
        let mut i = 0;
        while i < 255 {
            let byte = *path.add(i);
            if byte == 0 {
                break;
            }
            path_buf[i] = byte;
            i += 1;
        }

        // Call RAMFS open operation
        ramfs_open(path_buf.as_ptr(), i, flags)
    }
}

fn vfs_close(fd: usize) -> isize {
    unsafe {
        if fd >= MAX_FDS || OPEN_FDS[fd].is_none() {
            return -9; // EBADF
        }
        OPEN_FDS[fd] = None;
        0
    }
}

fn vfs_read(inode: u64, buf: *mut u8, len: usize, offset: u64) -> isize {
    ramfs_read(inode, buf, len, offset)
}

fn vfs_write(inode: u64, buf: *const u8, len: usize, offset: u64) -> isize {
    ramfs_write(inode, buf, len, offset)
}

fn vfs_stat(path: *const u8, stat_buf: *mut u8) -> isize {
    // Simple stat implementation
    unsafe {
        // Find the inode for this path
        let path_slice = core::slice::from_raw_parts(path, 256);
        let path_len = path_slice.iter().position(|&c| c == 0).unwrap_or(256);

        // For now, assume root directory
        if path_len >= 1 && path_slice[0] == b'/' {
            // Fill in stat structure
            let stat_ptr = stat_buf as *mut u64;
            *stat_ptr.add(0) = 1; // st_dev
            *stat_ptr.add(1) = 0; // st_ino
            *stat_ptr.add(2) = 0o755 | (4 << 12); // st_mode (IFDIR)
            *stat_ptr.add(3) = 1; // st_nlink
            *stat_ptr.add(4) = 0; // st_uid
            *stat_ptr.add(5) = 0; // st_gid
            *stat_ptr.add(6) = 0; // st_rdev
            *stat_ptr.add(7) = 4096; // st_size
            *stat_ptr.add(8) = 4096; // st_blksize
            *stat_ptr.add(9) = 1; // st_blocks
            // Timestamps (atime, mtime, ctime)
            *stat_ptr.add(10) = 0;
            *stat_ptr.add(11) = 0;
            *stat_ptr.add(12) = 0;
            return 0;
        }
        -2 // ENOENT
    }
}

fn vfs_mkdir(path: *const u8, mode: usize) -> isize {
    // Simple mkdir - create directory node
    unsafe {
        if RAMFS_NODE_COUNT < 256 {
            let node_idx = RAMFS_NODE_COUNT;
            let path_slice = core::slice::from_raw_parts(path, 256);
            let path_len = path_slice.iter().position(|&c| c == 0).unwrap_or(256);

            RAMFS_NODES[node_idx].set_name(&path_slice[..path_len]);
            RAMFS_NODES[node_idx].node_type = NodeType::Directory;
            RAMFS_NODES[node_idx].parent = 0; // Root
            RAMFS_NODE_COUNT += 1;
            return 0;
        }
        -28 // ENOSPC
    }
}

fn vfs_unlink(path: *const u8) -> isize {
    // Simple unlink - mark node as unused (not implemented)
    -38 // ENOSYS
}

fn vfs_rename(old_path: *const u8, new_path: *const u8) -> isize {
    // Simple rename - not implemented
    -38 // ENOSYS
}

fn vfs_sync(fd: usize) -> isize {
    0 // No-op for now
}

fn vfs_chdir(path: *const u8) -> isize {
    -38 // ENOSYS
}

fn vfs_getcwd(buf: *mut u8, size: usize) -> isize {
    // Return root directory for now
    unsafe {
        if size < 2 {
            return -34; // ERANGE
        }
        *buf = b'/';
        *(buf.add(1)) = 0;
        2
    }
}

fn vfs_mount(source: *const u8, target: *const u8, fstype: *const u8) -> isize {
    -38 // ENOSYS
}

fn vfs_umount(target: *const u8) -> isize {
    -38 // ENOSYS
}

// Simple in-kernel RAMFS implementation
// This is temporary - will be replaced with proper IPC

static mut RAMFS_NODES: [RamFsNode; 256] = [RamFsNode::new(); 256];
static mut RAMFS_NODE_COUNT: usize = 1; // Root directory

#[derive(Clone, Copy)]
struct RamFsNode {
    name: [u8; 64],
    name_len: usize,
    node_type: NodeType,
    data: [u8; 4096],
    size: usize,
    parent: usize,
}

#[derive(Clone, Copy, PartialEq)]
enum NodeType {
    File,
    Directory,
}

impl RamFsNode {
    const fn new() -> Self {
        RamFsNode {
            name: [0; 64],
            name_len: 0,
            node_type: NodeType::File,
            data: [0; 4096],
            size: 0,
            parent: 0,
        }
    }

    fn set_name(&mut self, name: &[u8]) {
        let len = name.len().min(64);
        self.name[..len].copy_from_slice(&name[..len]);
        self.name_len = len;
    }

    fn name_matches(&self, name: &[u8]) -> bool {
        self.name_len == name.len() && &self.name[..self.name_len] == name
    }
}

pub fn init_ramfs() {
    unsafe {
        // Initialize root directory
        RAMFS_NODES[0].set_name(b"/");
        RAMFS_NODES[0].node_type = NodeType::Directory;
        RAMFS_NODE_COUNT = 1;
    }
}

fn ramfs_open(path: *const u8, path_len: usize, flags: usize) -> isize {
    unsafe {
        if RAMFS_NODE_COUNT == 0 {
            init_ramfs();
        }

        // Simple implementation - create file if it doesn't exist
        if RAMFS_NODE_COUNT < 256 {
            let node_idx = RAMFS_NODE_COUNT;
            RAMFS_NODES[node_idx].set_name(core::slice::from_raw_parts(path, path_len));
            RAMFS_NODES[node_idx].node_type = NodeType::File;
            RAMFS_NODES[node_idx].parent = 0; // Root
            RAMFS_NODE_COUNT += 1;
            return node_idx as isize; // Return inode number
        }
        -28 // ENOSPC
    }
}

fn ramfs_read(inode: u64, buf: *mut u8, len: usize, offset: u64) -> isize {
    unsafe {
        if (inode as usize) >= RAMFS_NODE_COUNT {
            return -2; // ENOENT
        }

        let node = &RAMFS_NODES[inode as usize];
        let start = offset as usize;
        let end = (offset as usize + len).min(node.size);

        if start >= node.size {
            return 0; // EOF
        }

        let copy_len = end - start;
        core::ptr::copy_nonoverlapping(
            node.data.as_ptr().add(start),
            buf,
            copy_len
        );

        copy_len as isize
    }
}

fn ramfs_write(inode: u64, buf: *const u8, len: usize, offset: u64) -> isize {
    unsafe {
        if (inode as usize) >= RAMFS_NODE_COUNT {
            return -2; // ENOENT
        }

        let node = &mut RAMFS_NODES[inode as usize];
        let start = offset as usize;
        let end = start + len;

        if end > 4096 {
            return -27; // EFBIG - file too big
        }

        core::ptr::copy_nonoverlapping(
            buf,
            node.data.as_mut_ptr().add(start),
            len
        );

        if end > node.size {
            node.size = end;
        }

        len as isize
    }
}

fn sys_open(path: *const u8, flags: usize) -> isize {
    let result = vfs_open(path, flags);
    if result < 0 {
        return result; // Error from VFS
    }

    // Allocate file descriptor
    unsafe {
        for i in 3..MAX_FDS { // Skip stdin/stdout/stderr
            if OPEN_FDS[i].is_none() {
                OPEN_FDS[i] = Some(FileDescriptor {
                    inode: result as u64,
                    offset: 0,
                    flags: flags as u32,
                });
                return i as isize;
            }
        }
        -24 // EMFILE - too many open files
    }
}

fn sys_close(fd: usize) -> isize {
    vfs_close(fd)
}

fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    unsafe {
        if fd >= MAX_FDS || OPEN_FDS[fd].is_none() {
            return -9; // EBADF
        }

        let fd_info = OPEN_FDS[fd].unwrap();
        let write_result = vfs_write(fd_info.inode, buf, len, fd_info.offset);

        if write_result > 0 {
            OPEN_FDS[fd].as_mut().unwrap().offset += write_result as u64;
        }

        write_result
    }
}

fn sys_stat(path: *const u8, stat_buf: *mut u8) -> isize {
    vfs_stat(path, stat_buf)
}

fn sys_mkdir(path: *const u8, mode: usize) -> isize {
    vfs_mkdir(path, mode)
}

fn sys_unlink(path: *const u8) -> isize {
    vfs_unlink(path)
}

fn sys_rename(old_path: *const u8, new_path: *const u8) -> isize {
    vfs_rename(old_path, new_path)
}

fn sys_sync(fd: usize) -> isize {
    vfs_sync(fd)
}

fn sys_chdir(path: *const u8) -> isize {
    vfs_chdir(path)
}

fn sys_getcwd(buf: *mut u8, size: usize) -> isize {
    vfs_getcwd(buf, size)
}

fn sys_mount(source: *const u8, target: *const u8, fstype: *const u8) -> isize {
    vfs_mount(source, target, fstype)
}

fn sys_umount(target: *const u8) -> isize {
    vfs_umount(target)
}
