//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: libgbsd
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Obsługa błędów w bibliotece systemowej GuardBSD.

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
    NotFound,
    Unsupported,
}

impl Error {
    #[must_use]
    pub fn from_code(code: i32) -> Self {
        match code as i64 {
            0 => Error::Ok,
            2 => Error::NotFound,     // ENOENT
            11 => Error::Again,       // EAGAIN
            12 => Error::NoMemory,    // ENOMEM
            13 => Error::Permission,  // EACCES
            38 => Error::NoSys,       // ENOSYS
            95 => Error::Unsupported, // EOPNOTSUPP
            _ => Error::Invalid,
        }
    }

    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, Error::Ok)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
