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
        DEVICE_TABLE.register(DeviceType::Character, 0, 0); // null
        DEVICE_TABLE.register(DeviceType::Character, 1, 0); // console
    }

    let port = match port_create() {
        Ok(p) => p,
        Err(_) => exit(1),
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
                    Some(id) => DevResponse::ok(id),
                    None => DevResponse::err(28), // ENOSPC
                }
            }
            2 => {
                // Unregister device
                if DEVICE_TABLE.unregister(req.dev_id) {
                    DevResponse::ok(0)
                } else {
                    DevResponse::err(19) // ENODEV
                }
            }
            3 => {
                // Open device
                if DEVICE_TABLE.get(req.dev_id).is_some() {
                    DevResponse::ok(req.dev_id)
                } else {
                    DevResponse::err(19) // ENODEV
                }
            }
            4 => {
                // Close device
                DevResponse::ok(0)
            }
            5 | 6 | 7 => {
                // Read/Write/Ioctl - stub for now
                DevResponse::ok(0)
            }
            _ => DevResponse::err(38), // ENOSYS
        }
    }
}
