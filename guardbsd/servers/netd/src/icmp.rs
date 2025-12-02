// servers/netd/src/icmp.rs
// ICMP protocol implementation
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

pub const ICMP_ECHO_REPLY: u8 = 0;
pub const ICMP_ECHO_REQUEST: u8 = 8;

pub struct IcmpMessage {
    pub msg_type: u8,
    pub code: u8,
    pub id: u16,
    pub seq: u16,
    pub data: [u8; 1472],
    pub len: usize,
}

impl IcmpMessage {
    pub const fn new() -> Self {
        Self {
            msg_type: 0,
            code: 0,
            id: 0,
            seq: 0,
            data: [0; 1472],
            len: 0,
        }
    }

    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 8 {
            return None;
        }

        let msg_type = buf[0];
        let code = buf[1];
        let id = u16::from_be_bytes([buf[4], buf[5]]);
        let seq = u16::from_be_bytes([buf[6], buf[7]]);
        
        let data_len = buf.len().saturating_sub(8).min(1472);

        let mut message = Self::new();
        message.msg_type = msg_type;
        message.code = code;
        message.id = id;
        message.seq = seq;
        message.len = data_len;

        if buf.len() >= 8 + data_len {
            message.data[..data_len].copy_from_slice(&buf[8..8 + data_len]);
        }

        Some(message)
    }
}
