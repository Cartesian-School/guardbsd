// servers/netd/src/main.rs
// GuardBSD Network Stack Server
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use gbsd::*;

mod ip;
mod tcp;
mod udp;
mod icmp;
mod socket;

use socket::{SocketTable, SocketType};
use ip::IpAddr;

static mut SOCKET_TABLE: SocketTable = SocketTable::new();

const PROTO_TCP: u8 = 6;
const PROTO_UDP: u8 = 17;
const PROTO_ICMP: u8 = 1;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    netd_main()
}

fn netd_main() -> ! {
    let port = match port_create() {
        Ok(p) => p,
        Err(_) => exit(1),
    };

    let mut req_buf = [0u8; 2048];
    let mut resp_buf = [0u8; 2048];

    loop {
        if port_receive(port, req_buf.as_mut_ptr() as u64).is_ok() {
            if req_buf.len() >= 4 {
                let op = u32::from_le_bytes([req_buf[0], req_buf[1], req_buf[2], req_buf[3]]);
                let result = handle_request(op, &req_buf[4..], &mut resp_buf[8..]);
                
                resp_buf[0..8].copy_from_slice(&result.to_le_bytes());
                let _ = port_send(port, resp_buf.as_ptr() as u64);
            }
        }

        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("pause", options(nomem, nostack));
        }

        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("yield", options(nomem, nostack));
        }
    }
}

fn handle_request(op: u32, data: &[u8], _resp_data: &mut [u8]) -> i64 {
    unsafe {
        match op {
            1 => {
                // Socket create
                if data.len() >= 4 {
                    let sock_type_val = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    let sock_type = match sock_type_val {
                        1 => SocketType::Stream,
                        2 => SocketType::Dgram,
                        3 => SocketType::Raw,
                        _ => return -22, // EINVAL
                    };
                    
                    match SOCKET_TABLE.create(sock_type) {
                        Some(fd) => fd as i64,
                        None => -24, // EMFILE
                    }
                } else {
                    -22 // EINVAL
                }
            }
            2 => {
                // Bind
                if data.len() >= 10 {
                    let fd = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                    let addr = IpAddr::from_bytes(&data[4..8]);
                    let port = u16::from_be_bytes([data[8], data[9]]);
                    
                    if let Some(sock) = SOCKET_TABLE.get(fd) {
                        match sock.bind(addr, port) {
                            Ok(_) => 0,
                            Err(e) => e,
                        }
                    } else {
                        -9 // EBADF
                    }
                } else {
                    -22 // EINVAL
                }
            }
            3 => {
                // Listen
                if data.len() >= 4 {
                    let fd = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                    
                    if let Some(sock) = SOCKET_TABLE.get(fd) {
                        match sock.listen() {
                            Ok(_) => 0,
                            Err(e) => e,
                        }
                    } else {
                        -9 // EBADF
                    }
                } else {
                    -22 // EINVAL
                }
            }
            4 => {
                // Connect
                if data.len() >= 10 {
                    let fd = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                    let addr = IpAddr::from_bytes(&data[4..8]);
                    let port = u16::from_be_bytes([data[8], data[9]]);
                    
                    if let Some(sock) = SOCKET_TABLE.get(fd) {
                        match sock.connect(addr, port) {
                            Ok(_) => 0,
                            Err(e) => e,
                        }
                    } else {
                        -9 // EBADF
                    }
                } else {
                    -22 // EINVAL
                }
            }
            5 => {
                // Close
                if data.len() >= 4 {
                    let fd = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                    
                    if SOCKET_TABLE.close(fd) {
                        0
                    } else {
                        -9 // EBADF
                    }
                } else {
                    -22 // EINVAL
                }
            }
            _ => -38, // ENOSYS
        }
    }
}
