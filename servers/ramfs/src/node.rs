// servers/ramfs/src/node.rs
// RAM filesystem node structure
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    File,
    Directory,
}

#[derive(Copy, Clone)]
pub struct Node {
    pub name: [u8; 64],
    pub name_len: usize,
    pub node_type: NodeType,
    pub data: [u8; 4096],
    pub size: usize,
    pub parent: u32,
}

impl Node {
    pub const fn new() -> Self {
        Self {
            name: [0; 64],
            name_len: 0,
            node_type: NodeType::Directory,
            data: [0; 4096],
            size: 0,
            parent: 0,
        }
    }

    pub fn set_name(&mut self, name: &[u8]) {
        let len = name.len().min(64);
        self.name[..len].copy_from_slice(&name[..len]);
        self.name_len = len;
    }

    pub fn name_matches(&self, name: &[u8]) -> bool {
        self.name_len == name.len() && &self.name[..self.name_len] == name
    }
}

pub struct RamFs {
    nodes: [Node; 256],
    count: usize,
}

impl RamFs {
    pub const fn new() -> Self {
        const INIT: Node = Node::new();
        Self {
            nodes: [INIT; 256],
            count: 0,
        }
    }

    pub fn init(&mut self) {
        // Create root directory
        self.nodes[0].node_type = NodeType::Directory;
        self.nodes[0].set_name(b"/");
        self.count = 1;
    }

    pub fn create(&mut self, parent: u32, name: &[u8], ntype: NodeType) -> Option<u32> {
        if self.count >= 256 {
            return None;
        }
        let idx = self.count as u32;
        self.nodes[idx as usize].set_name(name);
        self.nodes[idx as usize].node_type = ntype;
        self.nodes[idx as usize].parent = parent;
        self.count += 1;
        Some(idx)
    }

    pub fn find(&self, parent: u32, name: &[u8]) -> Option<u32> {
        for i in 0..self.count {
            if self.nodes[i].parent == parent && self.nodes[i].name_matches(name) {
                return Some(i as u32);
            }
        }
        None
    }

    pub fn get(&mut self, idx: u32) -> Option<&mut Node> {
        if (idx as usize) < self.count {
            Some(&mut self.nodes[idx as usize])
        } else {
            None
        }
    }

    pub fn delete(&mut self, idx: u32) -> bool {
        if (idx as usize) >= self.count || idx == 0 {
            return false;
        }
        // Simple deletion: mark as unused by zeroing name_len
        self.nodes[idx as usize].name_len = 0;
        true
    }

    pub fn node_count(&self) -> usize {
        self.count
    }
}
