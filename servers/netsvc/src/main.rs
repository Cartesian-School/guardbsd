// servers/netsvc/src/main.rs
// GuardBSD Network Services Server
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use gbsd::*;

mod dns;
mod dhcp;
mod http;

use dns::DnsCache;
use dhcp::DhcpClient;

static mut DNS_CACHE: DnsCache = DnsCache::new();
static mut DHCP_CLIENT: DhcpClient = DhcpClient::new();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    netsvc_main()
}

fn netsvc_main() -> ! {
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

fn handle_request(op: u32, data: &[u8], resp_data: &mut [u8]) -> i64 {
    unsafe {
        match op {
            1 => {
                // DNS lookup
                if data.len() >= 4 {
                    let name_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                    if data.len() >= 4 + name_len {
                        let name = &data[4..4 + name_len];
                        
                        if let Some(response) = DNS_CACHE.lookup(name) {
                            resp_data[0..4].copy_from_slice(&response.addr);
                            resp_data[4..8].copy_from_slice(&response.ttl.to_le_bytes());
                            return 8;
                        }
                        
                        -2 // ENOENT
                    } else {
                        -22 // EINVAL
                    }
                } else {
                    -22 // EINVAL
                }
            }
            2 => {
                // DHCP discover
                DHCP_CLIENT.discover();
                0
            }
            3 => {
                // DHCP request
                DHCP_CLIENT.request();
                0
            }
            4 => {
                // Get DHCP lease
                if DHCP_CLIENT.state == dhcp::DhcpState::Bound {
                    resp_data[0..4].copy_from_slice(&DHCP_CLIENT.lease.ip_addr);
                    resp_data[4..8].copy_from_slice(&DHCP_CLIENT.lease.netmask);
                    resp_data[8..12].copy_from_slice(&DHCP_CLIENT.lease.gateway);
                    resp_data[12..16].copy_from_slice(&DHCP_CLIENT.lease.dns_server);
                    16
                } else {
                    -11 // EAGAIN
                }
            }
            5 => {
                // HTTP parse request
                if let Some(request) = http::HttpRequest::parse(data) {
                    resp_data[0] = request.method as u8;
                    resp_data[1..5].copy_from_slice(&(request.path_len as u32).to_le_bytes());
                    let len = request.path_len.min(resp_data.len() - 5);
                    resp_data[5..5 + len].copy_from_slice(&request.path[..len]);
                    (5 + len) as i64
                } else {
                    -22 // EINVAL
                }
            }
            _ => -38, // ENOSYS
        }
    }
}
