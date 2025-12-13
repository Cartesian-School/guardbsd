//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: guardfs
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! GuardFS — natywny system plików GuardBSD (biblioteka no_std).

#![no_std]

// GuardFS is a no_std library that can be used in both kernel and userland contexts
// For now, we don't use alloc in the library itself to keep it simple

pub mod bitmap;
pub mod compress;
pub mod dir;
pub mod inode;
pub mod journal;
pub mod ops;
pub mod snapshot;
pub mod superblock;

pub use bitmap::*;
pub use compress::*;
pub use dir::*;
pub use inode::*;
pub use journal::*;
pub use ops::*;
pub use snapshot::*;
pub use superblock::*;
