// userland/devtest/src/main.rs
// Device Driver Framework Test
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]
#![no_main]

use gbsd::*;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_device()
}

fn test_device() -> ! {
    // Test 1: Register character device
    match dev_register(DEV_CHAR, 10, 0) {
        Ok(dev_id) => {
            // Test 2: Open device
            match dev_open(dev_id) {
                Ok(_) => {
                    // Test 3: Close device
                    let _ = dev_close(dev_id);
                    
                    // Test 4: Unregister device
                    let _ = dev_unregister(dev_id);
                    
                    exit(0); // Success
                }
                Err(_) => exit(2),
            }
        }
        Err(_) => exit(1),
    }
}
