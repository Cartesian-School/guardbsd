//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: devd
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Operacje na urządzeniach obsługiwane przez devd.

#[repr(u32)]
pub enum DevOp {
    Register = 1,
    Unregister = 2,
    Open = 3,
    Close = 4,
    Read = 5,
    Write = 6,
    Ioctl = 7,
}

pub struct DevRequest {
    pub op: u32,
    pub dev_id: u32,
    pub major: u16,
    pub minor: u16,
    pub flags: u32,
}

impl DevRequest {
    pub fn from_bytes(data: &[u8]) -> Self {
        if data.len() >= 16 {
            Self {
                op: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
                dev_id: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
                major: u16::from_le_bytes([data[8], data[9]]),
                minor: u16::from_le_bytes([data[10], data[11]]),
                flags: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            }
        } else {
            Self {
                op: 0,
                dev_id: 0,
                major: 0,
                minor: 0,
                flags: 0,
            }
        }
    }
}

pub struct DevResponse {
    pub result: i64,
}

impl DevResponse {
    pub const fn ok(value: u32) -> Self {
        Self {
            result: value as i64,
        }
    }

    pub const fn err(code: i64) -> Self {
        Self { result: -code }
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        self.result.to_le_bytes()
    }
}
