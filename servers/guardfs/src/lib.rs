// servers/guardfs/src/lib.rs
// GuardFS - Native GuardBSD Filesystem
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#![no_std]

// GuardFS is a no_std library that can be used in both kernel and userland contexts
// For now, we don't use alloc in the library itself to keep it simple

pub mod superblock;
pub mod inode;
pub mod bitmap;
pub mod journal;
pub mod snapshot;
pub mod compress;
pub mod dir;
pub mod ops;

pub use superblock::*;
pub use inode::*;
pub use bitmap::*;
pub use journal::*;
pub use snapshot::*;
pub use compress::*;
pub use dir::*;
pub use ops::*;

