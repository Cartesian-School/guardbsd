//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: shell
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Operacje I/O powłoki gsh.

use gbsd::*;

pub const STDIN: Fd = 0;
pub const STDOUT: Fd = 1;
pub const STDERR: Fd = 2;

pub fn write_str(fd: Fd, s: &[u8]) -> Result<()> {
    write(fd, s)?;
    Ok(())
}

pub fn write_byte(fd: Fd, b: u8) -> Result<()> {
    write(fd, &[b])?;
    Ok(())
}

pub fn read_line(fd: Fd, buf: &mut [u8]) -> Result<usize> {
    let mut pos = 0;
    while pos < buf.len() {
        let mut byte = [0u8; 1];
        match read(fd, &mut byte) {
            Ok(0) => break,
            Ok(_) => {
                if byte[0] == b'\n' {
                    break;
                }
                buf[pos] = byte[0];
                pos += 1;
            }
            Err(_) => break,
        }
    }
    Ok(pos)
}

pub fn print(s: &[u8]) -> Result<()> {
    write_str(STDOUT, s)
}

pub fn println(s: &[u8]) -> Result<()> {
    print(s)?;
    write_byte(STDOUT, b'\n')
}
