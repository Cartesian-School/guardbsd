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
use http::HttpMethod;

static mut DNS_CACHE: DnsCache = DnsCache::new();
static mut DHCP_CLIENT: DhcpClient = DhcpClient::new();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    netsvc_main()
}

fn netsvc_main() -> ! {
    let pid = getpid().unwrap_or(0);
    klog_info!("netsvc", "network services server starting (pid={})", pid);

    let port = match port_create() {
        Ok(p) => {
            klog_info!("netsvc", "network services server started (port={})", p);
            p
        }
        Err(_) => {
            klog_error!("netsvc", "failed to create network services port");
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

fn handle_request(op: u32, data: &[u8], resp_data: &mut [u8]) -> i64 {
    unsafe {
        match op {
            1 => {
                // DNS lookup
                if data.len() >= 4 {
                    let name_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
                    if data.len() >= 4 + name_len {
                        let name = &data[4..4 + name_len];
                        let hostname = core::str::from_utf8(name).unwrap_or("<invalid>");

                        if let Some(response) = DNS_CACHE.lookup(name) {
                            klog_info!("netsvc", "DNS query from {} for '{}'", "127.0.0.1", hostname);
                            resp_data[0..4].copy_from_slice(&response.addr);
                            resp_data[4..8].copy_from_slice(&response.ttl.to_le_bytes());
                            return 8;
                        }

                        klog_warn!("netsvc", "DNS query from {} for '{}' - not found", "127.0.0.1", hostname);
                        -2 // ENOENT
                    } else {
                        klog_warn!("netsvc", "DNS query - invalid name length");
                        -22 // EINVAL
                    }
                } else {
                    klog_warn!("netsvc", "DNS query - invalid data length");
                    -22 // EINVAL
                }
            }
            2 => {
                // DHCP discover
                DHCP_CLIENT.discover();
                klog_info!("netsvc", "DHCP discover sent");
                0
            }
            3 => {
                // DHCP request
                DHCP_CLIENT.request();
                klog_info!("netsvc", "DHCP request sent");
                0
            }
            4 => {
                // Get DHCP lease
                if DHCP_CLIENT.state == dhcp::DhcpState::Bound {
                    let ip_addr = u32::from_be_bytes(DHCP_CLIENT.lease.ip_addr);
                    klog_info!("netsvc", "DHCP lease obtained: {}.{}.{}.{}",
                               (ip_addr >> 24) & 0xff,
                               (ip_addr >> 16) & 0xff,
                               (ip_addr >> 8) & 0xff,
                               ip_addr & 0xff);
                    resp_data[0..4].copy_from_slice(&DHCP_CLIENT.lease.ip_addr);
                    resp_data[4..8].copy_from_slice(&DHCP_CLIENT.lease.netmask);
                    resp_data[8..12].copy_from_slice(&DHCP_CLIENT.lease.gateway);
                    resp_data[12..16].copy_from_slice(&DHCP_CLIENT.lease.dns_server);
                    16
                } else {
                    klog_warn!("netsvc", "DHCP lease not available");
                    -11 // EAGAIN
                }
            }
            5 => {
                // HTTP parse request
                if let Some(request) = http::HttpRequest::parse(data) {
                    let method_str = match request.method {
                        HttpMethod::Get => "GET",
                        HttpMethod::Post => "POST",
                        HttpMethod::Head => "HEAD",
                    };
                    let path_str = core::str::from_utf8(&request.path[..request.path_len]).unwrap_or("<invalid>");
                    klog_info!("netsvc", "HTTP request {} {}", method_str, path_str);

                    resp_data[0] = request.method as u8;
                    resp_data[1..5].copy_from_slice(&(request.path_len as u32).to_le_bytes());
                    let len = request.path_len.min(resp_data.len() - 5);
                    resp_data[5..5 + len].copy_from_slice(&request.path[..len]);
                    (5 + len) as i64
                } else {
                    klog_warn!("netsvc", "HTTP 400 - bad request");
                    -22 // EINVAL
                }
            }
            _ => {
                klog_warn!("netsvc", "unknown operation {}", op);
                -38 // ENOSYS
            }
        }
    }
}
