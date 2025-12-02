// servers/netd/src/socket.rs
// Socket interface
// ============================================================================
// Copyright (c) 2025 Cartesian School - Siergej Sobolewski
// SPDX-License-Identifier: BSD-3-Clause

use crate::tcp::TcpConnection;
use crate::ip::IpAddr;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    Stream,  // TCP
    Dgram,   // UDP
    Raw,     // Raw IP
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
    Closed,
    Bound,
    Listening,
    Connected,
}

#[derive(Copy, Clone)]
pub struct Socket {
    pub sock_type: SocketType,
    pub state: SocketState,
    pub local_addr: IpAddr,
    pub local_port: u16,
    pub remote_addr: IpAddr,
    pub remote_port: u16,
    pub tcp_conn: TcpConnection,
}

impl Socket {
    pub const fn new(sock_type: SocketType) -> Self {
        Self {
            sock_type,
            state: SocketState::Closed,
            local_addr: IpAddr::new(0, 0, 0, 0),
            local_port: 0,
            remote_addr: IpAddr::new(0, 0, 0, 0),
            remote_port: 0,
            tcp_conn: TcpConnection::new(),
        }
    }

    pub fn bind(&mut self, addr: IpAddr, port: u16) -> Result<(), i64> {
        if self.state != SocketState::Closed {
            return Err(-22); // EINVAL
        }
        self.local_addr = addr;
        self.local_port = port;
        self.state = SocketState::Bound;
        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), i64> {
        if self.state != SocketState::Bound {
            return Err(-22); // EINVAL
        }
        if self.sock_type != SocketType::Stream {
            return Err(-95); // EOPNOTSUPP
        }
        self.tcp_conn.listen(self.local_port);
        self.state = SocketState::Listening;
        Ok(())
    }

    pub fn connect(&mut self, addr: IpAddr, port: u16) -> Result<(), i64> {
        self.remote_addr = addr;
        self.remote_port = port;
        if self.sock_type == SocketType::Stream {
            self.tcp_conn.connect(port);
        }
        self.state = SocketState::Connected;
        Ok(())
    }
}

pub struct SocketTable {
    sockets: [Option<Socket>; 64],
    count: usize,
}

impl SocketTable {
    pub const fn new() -> Self {
        Self {
            sockets: [None; 64],
            count: 0,
        }
    }

    pub fn create(&mut self, sock_type: SocketType) -> Option<usize> {
        for (i, slot) in self.sockets.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(Socket::new(sock_type));
                self.count += 1;
                return Some(i);
            }
        }
        None
    }

    pub fn get(&mut self, fd: usize) -> Option<&mut Socket> {
        if fd < 64 {
            self.sockets[fd].as_mut()
        } else {
            None
        }
    }

    pub fn close(&mut self, fd: usize) -> bool {
        if fd < 64 && self.sockets[fd].is_some() {
            self.sockets[fd] = None;
            self.count = self.count.saturating_sub(1);
            true
        } else {
            false
        }
    }
}
