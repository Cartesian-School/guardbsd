// servers/netd/src/udp.rs
// UDP protocol implementation
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

pub struct UdpDatagram {
    pub src_port: u16,
    pub dst_port: u16,
    pub data: [u8; 1472],
    pub len: usize,
}

impl UdpDatagram {
    pub const fn new() -> Self {
        Self {
            src_port: 0,
            dst_port: 0,
            data: [0; 1472],
            len: 0,
        }
    }

    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 8 {
            return None;
        }

        let src_port = u16::from_be_bytes([buf[0], buf[1]]);
        let dst_port = u16::from_be_bytes([buf[2], buf[3]]);
        let length = u16::from_be_bytes([buf[4], buf[5]]) as usize;
        
        let data_len = length.saturating_sub(8).min(1472);

        let mut datagram = Self::new();
        datagram.src_port = src_port;
        datagram.dst_port = dst_port;
        datagram.len = data_len;

        if buf.len() >= 8 + data_len {
            datagram.data[..data_len].copy_from_slice(&buf[8..8 + data_len]);
        }

        Some(datagram)
    }
}
