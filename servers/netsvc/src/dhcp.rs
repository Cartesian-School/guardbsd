// servers/netsvc/src/dhcp.rs
// DHCP client implementation
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DhcpState {
    Init,
    Selecting,
    Requesting,
    Bound,
}

pub struct DhcpLease {
    pub ip_addr: [u8; 4],
    pub netmask: [u8; 4],
    pub gateway: [u8; 4],
    pub dns_server: [u8; 4],
    pub lease_time: u32,
}

impl DhcpLease {
    pub const fn new() -> Self {
        Self {
            ip_addr: [0; 4],
            netmask: [0; 4],
            gateway: [0; 4],
            dns_server: [0; 4],
            lease_time: 0,
        }
    }
}

pub struct DhcpClient {
    pub state: DhcpState,
    pub lease: DhcpLease,
    pub xid: u32,
}

impl DhcpClient {
    pub const fn new() -> Self {
        Self {
            state: DhcpState::Init,
            lease: DhcpLease::new(),
            xid: 0,
        }
    }

    pub fn discover(&mut self) {
        self.state = DhcpState::Selecting;
        self.xid = self.xid.wrapping_add(1);
    }

    pub fn request(&mut self) {
        self.state = DhcpState::Requesting;
    }

    pub fn bind(&mut self, lease: DhcpLease) {
        self.lease = lease;
        self.state = DhcpState::Bound;
    }
}
