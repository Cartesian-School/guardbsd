// servers/vfs/src/vnode.rs
// Virtual node (vnode) abstraction
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VnodeType {
    File,
    Directory,
    Device,
    Link,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vnode {
    pub vnode_id: u64,
    pub vnode_type: VnodeType,
    pub size: u64,
    pub refcount: u32,
    pub flags: u32,
}

impl Vnode {
    pub const fn new(id: u64, vtype: VnodeType) -> Self {
        Self {
            vnode_id: id,
            vnode_type: vtype,
            size: 0,
            refcount: 1,
            flags: 0,
        }
    }

    pub fn incref(&mut self) {
        self.refcount = self.refcount.saturating_add(1);
    }

    pub fn decref(&mut self) -> u32 {
        self.refcount = self.refcount.saturating_sub(1);
        self.refcount
    }
}

pub struct VnodeTable {
    vnodes: [Option<Vnode>; 256],
    next_id: u64,
}

impl VnodeTable {
    pub const fn new() -> Self {
        Self {
            vnodes: [None; 256],
            next_id: 1,
        }
    }

    pub fn alloc(&mut self, vtype: VnodeType) -> Option<u64> {
        for slot in &mut self.vnodes {
            if slot.is_none() {
                let id = self.next_id;
                self.next_id += 1;
                *slot = Some(Vnode::new(id, vtype));
                return Some(id);
            }
        }
        None
    }

    pub fn get(&mut self, id: u64) -> Option<&mut Vnode> {
        self.vnodes
            .iter_mut()
            .find_map(|slot| slot.as_mut().filter(|v| v.vnode_id == id))
    }

    pub fn free(&mut self, id: u64) -> bool {
        for slot in &mut self.vnodes {
            if let Some(vnode) = slot {
                if vnode.vnode_id == id {
                    *slot = None;
                    return true;
                }
            }
        }
        false
    }
}
