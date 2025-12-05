// servers/netsvc/src/dns.rs
// DNS client implementation
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#[derive(Copy, Clone)]
pub struct DnsQuery {
    pub name: [u8; 256],
    pub name_len: usize,
    pub qtype: u16,
}

impl DnsQuery {
    pub const fn new() -> Self {
        Self {
            name: [0; 256],
            name_len: 0,
            qtype: 1, // A record
        }
    }

    pub fn set_name(&mut self, name: &[u8]) {
        let len = name.len().min(256);
        self.name[..len].copy_from_slice(&name[..len]);
        self.name_len = len;
    }
}

#[derive(Copy, Clone)]
pub struct DnsResponse {
    pub addr: [u8; 4],
    pub ttl: u32,
}

impl DnsResponse {
    pub const fn new() -> Self {
        Self {
            addr: [0; 4],
            ttl: 0,
        }
    }
}

pub struct DnsCache {
    entries: [Option<(DnsQuery, DnsResponse)>; 16],
    count: usize,
}

impl DnsCache {
    pub const fn new() -> Self {
        Self {
            entries: [None; 16],
            count: 0,
        }
    }

    pub fn lookup(&self, name: &[u8]) -> Option<&DnsResponse> {
        for entry in &self.entries {
            if let Some((query, response)) = entry {
                if query.name_len == name.len() && &query.name[..query.name_len] == name {
                    return Some(response);
                }
            }
        }
        None
    }

    pub fn insert(&mut self, query: DnsQuery, response: DnsResponse) {
        if self.count < 16 {
            self.entries[self.count] = Some((query, response));
            self.count += 1;
        }
    }
}
