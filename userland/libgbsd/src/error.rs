// userland/libgbsd/src/error.rs
// Error handling for GuardBSD system library
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Ok = 0,
    Invalid,
    NoMemory,
    PortInvalid,
    PortFull,
    NoRights,
    CapInvalid,
    Again,
    Permission,
    Alignment,
    NoSys,
}

impl Error {
    #[must_use]
    pub fn from_code(code: i32) -> Self {
        match code as i64 {
            0 => Error::Ok,
            2 => Error::NoMemory,
            3 => Error::PortInvalid,
            4 => Error::PortFull,
            5 => Error::NoRights,
            6 => Error::CapInvalid,
            7 => Error::Alignment,
            11 => Error::Again,      // EAGAIN
            13 => Error::Permission, // EACCES
            38 => Error::NoSys,      // ENOSYS
            _ => Error::Invalid,
        }
    }

    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, Error::Ok)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
