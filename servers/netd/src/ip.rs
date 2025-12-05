// servers/netd/src/ip.rs
// IP layer implementation
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use core::fmt;

#[derive(Clone, Copy)]
pub struct IpAddr {
    pub octets: [u8; 4],
}

impl IpAddr {
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self {
            octets: [a, b, c, d],
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            octets: [bytes[0], bytes[1], bytes[2], bytes[3]],
        }
    }

    pub fn to_u32(&self) -> u32 {
        u32::from_be_bytes(self.octets)
    }
}

impl fmt::Display for IpAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.octets[0], self.octets[1], self.octets[2], self.octets[3]
        )
    }
}

pub struct IpPacket {
    pub src: IpAddr,
    pub dst: IpAddr,
    pub protocol: u8,
    pub data: [u8; 1500],
    pub len: usize,
}

impl IpPacket {
    pub const fn new() -> Self {
        Self {
            src: IpAddr::new(0, 0, 0, 0),
            dst: IpAddr::new(0, 0, 0, 0),
            protocol: 0,
            data: [0; 1500],
            len: 0,
        }
    }

    pub fn parse(buf: &[u8]) -> Option<Self> {
        if buf.len() < 20 {
            return None;
        }

        let version = buf[0] >> 4;
        if version != 4 {
            return None;
        }

        let protocol = buf[9];
        let src = IpAddr::from_bytes(&buf[12..16]);
        let dst = IpAddr::from_bytes(&buf[16..20]);

        let total_len = u16::from_be_bytes([buf[2], buf[3]]) as usize;
        let header_len = ((buf[0] & 0x0F) * 4) as usize;
        let data_len = total_len.saturating_sub(header_len).min(1500);

        let mut packet = Self::new();
        packet.src = src;
        packet.dst = dst;
        packet.protocol = protocol;
        packet.len = data_len;

        if buf.len() >= header_len + data_len {
            packet.data[..data_len].copy_from_slice(&buf[header_len..header_len + data_len]);
        }

        Some(packet)
    }
}
