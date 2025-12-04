// servers/devd/src/main.rs
// GuardBSD Device Driver Framework Server
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use gbsd::*;

mod device;
mod ops;

use device::{DeviceTable, DeviceType};
use ops::{DevRequest, DevResponse};

static mut DEVICE_TABLE: DeviceTable = DeviceTable::new();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    devd_main()
}

fn devd_main() -> ! {
    unsafe {
        // Register standard devices
        let null_id = DEVICE_TABLE.register(DeviceType::Character, 0, 0); // null
        let console_id = DEVICE_TABLE.register(DeviceType::Character, 1, 0); // console
        let device_count = if null_id.is_some() && console_id.is_some() { 2 } else { 0 };

        let pid = getpid().unwrap_or(0);
        klog_info!("devd", "device server online (devices={}, pid={})", device_count, pid);
    }

    let port = match port_create() {
        Ok(p) => {
            klog_info!("devd", "device server started (port={})", p);
            p
        }
        Err(_) => {
            klog_error!("devd", "failed to create device server port");
            exit(1);
        }
    };

    let mut req_buf = [0u8; 256];
    let mut resp_buf = [0u8; 8];

    loop {
        if port_receive(port, req_buf.as_mut_ptr(), req_buf.len()).is_ok() {
            let req = DevRequest::from_bytes(&req_buf);
            let resp = handle_request(&req);
            resp_buf.copy_from_slice(&resp.to_bytes());
            let _ = port_send(port, resp_buf.as_ptr(), resp_buf.len());
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

fn handle_request(req: &DevRequest) -> DevResponse {
    unsafe {
        match req.op {
            1 => {
                // Register device
                let dev_type = match req.flags & 0x3 {
                    0 => DeviceType::Character,
                    1 => DeviceType::Block,
                    2 => DeviceType::Network,
                    _ => return DevResponse::err(22), // EINVAL
                };
                match DEVICE_TABLE.register(dev_type, req.major, req.minor) {
                    Some(id) => {
                        let type_str = match dev_type {
                            DeviceType::Character => "char",
                            DeviceType::Block => "block",
                            DeviceType::Network => "net",
                        };
                        klog_info!("devd", "register device id={} name='{}/{}' type={}",
                                 id, req.major, req.minor, type_str);
                        DevResponse::ok(id)
                    }
                    None => {
                        klog_error!("devd", "failed to initialize device major={} minor={}", req.major, req.minor);
                        DevResponse::err(28) // ENOSPC
                    }
                }
            }
            2 => {
                // Unregister device
                if DEVICE_TABLE.unregister(req.dev_id) {
                    klog_info!("devd", "unregister device id={}", req.dev_id);
                    DevResponse::ok(0)
                } else {
                    klog_warn!("devd", "device id={} not found for unregister", req.dev_id);
                    DevResponse::err(19) // ENODEV
                }
            }
            3 => {
                // Open device
                if DEVICE_TABLE.get(req.dev_id).is_some() {
                    klog_info!("devd", "open device id={}", req.dev_id);
                    DevResponse::ok(req.dev_id)
                } else {
                    klog_warn!("devd", "device id={} not found for open", req.dev_id);
                    DevResponse::err(19) // ENODEV
                }
            }
            4 => {
                // Close device
                klog_info!("devd", "close device id={}", req.dev_id);
                DevResponse::ok(0)
            }
            5 => {
                // Read device
                klog_info!("devd", "read device id={}", req.dev_id);
                DevResponse::ok(0)
            }
            6 => {
                // Write device
                klog_info!("devd", "write device id={}", req.dev_id);
                DevResponse::ok(0)
            }
            7 => {
                // Ioctl device
                klog_info!("devd", "ioctl device id={}", req.dev_id);
                DevResponse::ok(0)
            }
            _ => {
                klog_warn!("devd", "unknown operation {}", req.op);
                DevResponse::err(38) // ENOSYS
            }
        }
    }
}
