//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: devtest
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Testy frameworka sterowników urządzeń (userland).

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
