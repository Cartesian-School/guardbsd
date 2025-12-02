// servers/devd/src/device.rs
// Device abstraction
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Character,
    Block,
    Network,
}

#[derive(Clone, Copy)]
pub struct Device {
    pub id: u32,
    pub dev_type: DeviceType,
    pub major: u16,
    pub minor: u16,
    pub flags: u32,
}

impl Device {
    pub const fn new(id: u32, dev_type: DeviceType, major: u16, minor: u16) -> Self {
        Self {
            id,
            dev_type,
            major,
            minor,
            flags: 0,
        }
    }
}

pub struct DeviceTable {
    devices: [Option<Device>; 64],
    count: usize,
}

impl DeviceTable {
    pub const fn new() -> Self {
        Self {
            devices: [None; 64],
            count: 0,
        }
    }

    pub fn register(&mut self, dev_type: DeviceType, major: u16, minor: u16) -> Option<u32> {
        if self.count >= 64 {
            return None;
        }
        let id = self.count as u32;
        self.devices[self.count] = Some(Device::new(id, dev_type, major, minor));
        self.count += 1;
        Some(id)
    }

    pub fn get(&self, id: u32) -> Option<&Device> {
        if (id as usize) < self.count {
            self.devices[id as usize].as_ref()
        } else {
            None
        }
    }

    pub fn find(&self, major: u16, minor: u16) -> Option<u32> {
        for i in 0..self.count {
            if let Some(dev) = &self.devices[i] {
                if dev.major == major && dev.minor == minor {
                    return Some(dev.id);
                }
            }
        }
        None
    }

    pub fn unregister(&mut self, id: u32) -> bool {
        if (id as usize) < self.count {
            self.devices[id as usize] = None;
            true
        } else {
            false
        }
    }
}
