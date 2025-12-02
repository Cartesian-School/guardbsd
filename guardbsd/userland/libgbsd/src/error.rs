// userland/libgbsd/src/error.rs
// Error handling for GuardBSD system library
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Ok = 0,
    Invalid = 1,
    NoMemory = 2,
    PortInvalid = 3,
    PortFull = 4,
    NoRights = 5,
    CapInvalid = 6,
    Again = 7,
    Permission = 8,
}

impl Error {
    pub fn from_code(code: u64) -> Self {
        match code {
            0 => Error::Ok,
            1 => Error::Invalid,
            2 => Error::NoMemory,
            3 => Error::PortInvalid,
            4 => Error::PortFull,
            5 => Error::NoRights,
            6 => Error::CapInvalid,
            7 => Error::Again,
            8 => Error::Permission,
            _ => Error::Invalid,
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, Error::Ok)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
