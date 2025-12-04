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
    pub fn from_code(code: u64) -> Self {
        match code {
            0 => Error::Ok,
            0xFFFF_FFFF_0000_0001 => Error::Invalid,
            0xFFFF_FFFF_0000_0002 => Error::NoMemory,
            0xFFFF_FFFF_0000_0003 => Error::PortInvalid,
            0xFFFF_FFFF_0000_0004 => Error::PortFull,
            0xFFFF_FFFF_0000_0005 => Error::NoRights,
            0xFFFF_FFFF_0000_0006 => Error::CapInvalid,
            0xFFFF_FFFF_0000_0007 => Error::Alignment,
            0xFFFF_FFFF_0000_0100 => Error::Again,
            0xFFFF_FFFF_0000_0200 => Error::Permission,
            0xFFFF_FFFF_FFFF_FFDA => Error::NoSys, // -38 ENOSYS
            _ => Error::Invalid,
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, Error::Ok)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
