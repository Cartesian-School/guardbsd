//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: vfstest
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Test integracyjny VFS/RAMFS w userlandzie.

#![no_std]
#![no_main]

use gbsd::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_vfs();
}

fn test_vfs() -> ! {
    // Test 1: Create file
    let path = b"/test.txt\0";
    match open(path, O_CREAT | O_WRONLY) {
        Ok(fd) => {
            // Test 2: Write to file
            let data = b"Hello GuardBSD!";
            let _ = write(fd, data);
            let _ = close(fd);
        }
        Err(_) => exit(1),
    }

    // Test 3: Read file
    match open(path, O_RDONLY) {
        Ok(fd) => {
            let mut buf = [0u8; 64];
            match read(fd, &mut buf) {
                Ok(n) if n > 0 => {
                    let _ = close(fd);
                    exit(0); // Success
                }
                _ => {
                    let _ = close(fd);
                    exit(2);
                }
            }
        }
        Err(_) => exit(3),
    }
}
