// servers/netd/src/tcp.rs
// TCP protocol implementation
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TcpState {
    Closed,
    Listen,
    SynSent,
    Established,
    FinWait,
}

#[derive(Copy, Clone)]
pub struct TcpConnection {
    pub state: TcpState,
    pub local_port: u16,
    pub remote_port: u16,
    pub seq: u32,
    pub ack: u32,
}

impl TcpConnection {
    pub const fn new() -> Self {
        Self {
            state: TcpState::Closed,
            local_port: 0,
            remote_port: 0,
            seq: 0,
            ack: 0,
        }
    }

    pub fn listen(&mut self, port: u16) {
        self.local_port = port;
        self.state = TcpState::Listen;
    }

    pub fn connect(&mut self, port: u16) {
        self.remote_port = port;
        self.state = TcpState::SynSent;
    }
}

pub struct TcpSegment {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq: u32,
    pub ack: u32,
    pub flags: u8,
    pub data: [u8; 1460],
    pub len: usize,
}

impl TcpSegment {
    pub const fn new() -> Self {
        Self {
            src_port: 0,
            dst_port: 0,
            seq: 0,
            ack: 0,
            flags: 0,
            data: [0; 1460],
            len: 0,
        }
    }

    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 20 {
            return None;
        }

        let src_port = u16::from_be_bytes([buf[0], buf[1]]);
        let dst_port = u16::from_be_bytes([buf[2], buf[3]]);
        let seq = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let ack = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let flags = buf[13];

        let header_len = ((buf[12] >> 4) * 4) as usize;
        let data_len = buf.len().saturating_sub(header_len).min(1460);

        let mut segment = Self::new();
        segment.src_port = src_port;
        segment.dst_port = dst_port;
        segment.seq = seq;
        segment.ack = ack;
        segment.flags = flags;
        segment.len = data_len;

        if buf.len() >= header_len + data_len {
            segment.data[..data_len].copy_from_slice(&buf[header_len..header_len + data_len]);
        }

        Some(segment)
    }
}
