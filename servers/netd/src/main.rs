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
    // Initialize network stack with basic MTU
    const DEFAULT_MTU: u16 = 1500;

    klog_info!("netd", "netd online (mtu={}, pid={})", DEFAULT_MTU, getpid());

    let port = match port_create() {
        Ok(p) => {
            klog_info!("netd", "network server started (port={})", p);
            p
        }
        Err(_) => {
            klog_error!("netd", "failed to create network server port");
            exit(1);
        }
    };

    let mut req_buf = [0u8; 2048];
    let mut resp_buf = [0u8; 2048];

    loop {
        if port_receive(port, req_buf.as_mut_ptr(), req_buf.len()).is_ok() {
            if req_buf.len() >= 4 {
                let op = u32::from_le_bytes([req_buf[0], req_buf[1], req_buf[2], req_buf[3]]);
                let result = handle_request(op, &req_buf[4..], &mut resp_buf[8..]);
                
                resp_buf[0..8].copy_from_slice(&result.to_le_bytes());
                let _ = port_send(port, resp_buf.as_ptr(), resp_buf.len());
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
                    let (sock_type, proto_str) = match sock_type_val {
                        1 => (SocketType::Stream, "tcp"),
                        2 => (SocketType::Dgram, "udp"),
                        3 => (SocketType::Raw, "raw"),
                        _ => return -22, // EINVAL
                    };

                    match SOCKET_TABLE.create(sock_type) {
                        Some(fd) => {
                            klog_info!("netd", "socket {} created (proto={})", fd, proto_str);
                            fd as i64
                        }
                        None => {
                            klog_error!("netd", "socket creation failed - table full");
                            -24 // EMFILE
                        }
                    }
                } else {
                    klog_warn!("netd", "socket create - invalid data length");
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
                            Ok(_) => {
                                klog_info!("netd", "socket {} bound to {}:{}", fd, addr, port);
                                0
                            }
                            Err(e) => {
                                klog_error!("netd", "socket {} bind failed: {}", fd, e);
                                e
                            }
                        }
                    } else {
                        klog_warn!("netd", "socket {} bind - invalid descriptor", fd);
                        -9 // EBADF
                    }
                } else {
                    klog_warn!("netd", "socket bind - invalid data length");
                    -22 // EINVAL
                }
            }
            3 => {
                // Listen
                if data.len() >= 4 {
                    let fd = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

                    if let Some(sock) = SOCKET_TABLE.get(fd) {
                        match sock.listen() {
                            Ok(_) => {
                                klog_info!("netd", "socket {} listening", fd);
                                0
                            }
                            Err(e) => {
                                klog_error!("netd", "socket {} listen failed: {}", fd, e);
                                e
                            }
                        }
                    } else {
                        klog_warn!("netd", "socket {} listen - invalid descriptor", fd);
                        -9 // EBADF
                    }
                } else {
                    klog_warn!("netd", "socket listen - invalid data length");
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
                            Ok(_) => {
                                klog_info!("netd", "socket {} connecting to {}:{}", fd, addr, port);
                                0
                            }
                            Err(e) => {
                                klog_error!("netd", "socket {} connect failed: {}", fd, e);
                                e
                            }
                        }
                    } else {
                        klog_warn!("netd", "socket {} connect - invalid descriptor", fd);
                        -9 // EBADF
                    }
                } else {
                    klog_warn!("netd", "socket connect - invalid data length");
                    -22 // EINVAL
                }
            }
            5 => {
                // Close
                if data.len() >= 4 {
                    let fd = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

                    if SOCKET_TABLE.close(fd) {
                        klog_info!("netd", "socket {} closed", fd);
                        0
                    } else {
                        klog_warn!("netd", "socket {} close - invalid descriptor", fd);
                        -9 // EBADF
                    }
                } else {
                    klog_warn!("netd", "socket close - invalid data length");
                    -22 // EINVAL
                }
            }
            _ => {
                klog_warn!("netd", "unknown operation {}", op);
                -38 // ENOSYS
            }
        }
    }
}
