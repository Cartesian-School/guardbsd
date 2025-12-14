//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School. Developed by Siergej Sobolewski.
//! License: BSD-3-Clause
//!
//! Infrastruktura IPC w boot stubie (proste porty i kolejki wiadomości).

use core::sync::atomic::{AtomicBool, Ordering};

// Message structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Message {
    pub sender_pid: usize,
    pub receiver_pid: usize,
    pub msg_type: u32,
    pub data: [u32; 4], // Simple data payload
}

// Port structure
#[repr(C)]
pub struct Port {
    pub port_id: usize,
    pub owner_pid: usize,
    pub messages: [Option<Message>; 16], // Simple ring buffer
    pub read_idx: usize,
    pub write_idx: usize,
    pub is_open: AtomicBool,
}

impl Port {
    pub const fn new(port_id: usize, owner_pid: usize) -> Self {
        Port {
            port_id,
            owner_pid,
            messages: [None; 16],
            read_idx: 0,
            write_idx: 0,
            is_open: AtomicBool::new(true),
        }
    }

    pub fn send(&mut self, msg: Message) -> bool {
        if !self.is_open.load(Ordering::Acquire) {
            return false;
        }

        let next_write = (self.write_idx + 1) % 16;
        if next_write == self.read_idx {
            // Buffer full
            return false;
        }

        self.messages[self.write_idx] = Some(msg);
        self.write_idx = next_write;
        true
    }

    pub fn receive(&mut self) -> Option<Message> {
        if self.read_idx == self.write_idx || !self.is_open.load(Ordering::Acquire) {
            return None;
        }

        let msg = self.messages[self.read_idx];
        self.messages[self.read_idx] = None;
        self.read_idx = (self.read_idx + 1) % 16;
        msg
    }

    pub fn close(&mut self) {
        self.is_open.store(false, Ordering::Release);
    }
}

// IPC Manager
pub struct IpcManager {
    ports: [core::mem::MaybeUninit<Option<Port>>; 64],
    next_port_id: usize,
}

impl IpcManager {
    pub fn new() -> Self {
        let mut manager = IpcManager {
            ports: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            next_port_id: 1,
        };

        // Initialize all ports to None
        for port in &mut manager.ports {
            unsafe {
                port.write(None);
            }
        }

        manager
    }

    pub fn create_port(&mut self, owner_pid: usize) -> Option<usize> {
        if self.next_port_id >= 64 {
            return None;
        }

        let port_id = self.next_port_id;
        self.next_port_id += 1;

        unsafe {
            self.ports[port_id].write(Some(Port::new(port_id, owner_pid)));
        }
        Some(port_id)
    }

    pub fn get_port(&self, port_id: usize) -> Option<&Port> {
        if port_id < 64 {
            unsafe { self.ports[port_id].assume_init_ref().as_ref() }
        } else {
            None
        }
    }

    pub fn get_port_mut(&mut self, port_id: usize) -> Option<&mut Port> {
        if port_id < 64 {
            unsafe { self.ports[port_id].assume_init_mut().as_mut() }
        } else {
            None
        }
    }

    pub fn send_message(&mut self, port_id: usize, msg: Message) -> bool {
        if let Some(port) = self.get_port_mut(port_id) {
            port.send(msg)
        } else {
            false
        }
    }

    pub fn receive_message(&mut self, port_id: usize) -> Option<Message> {
        if let Some(port) = self.get_port_mut(port_id) {
            port.receive()
        } else {
            None
        }
    }

    pub fn close_port(&mut self, port_id: usize) {
        if let Some(port) = self.get_port_mut(port_id) {
            port.close();
        }
    }
}

// Global IPC manager (initialized at runtime)
pub static mut IPC_MANAGER: core::mem::MaybeUninit<IpcManager> = core::mem::MaybeUninit::uninit();

pub fn init_ipc() {
    unsafe {
        IPC_MANAGER.write(IpcManager::new());
    }
}

// IPC Syscall handlers
pub fn ipc_create_port(owner_pid: usize) -> isize {
    unsafe {
        if let Some(port_id) = IPC_MANAGER.assume_init_mut().create_port(owner_pid) {
            port_id as isize
        } else {
            -1 // No available ports
        }
    }
}

pub fn ipc_send(
    port_id: usize,
    sender_pid: usize,
    receiver_pid: usize,
    msg_type: u32,
    data: [u32; 4],
) -> isize {
    let msg = Message {
        sender_pid,
        receiver_pid,
        msg_type,
        data,
    };

    unsafe {
        if IPC_MANAGER.assume_init_mut().send_message(port_id, msg) {
            0
        } else {
            -1 // Send failed
        }
    }
}

pub fn ipc_receive(port_id: usize) -> Option<Message> {
    unsafe { IPC_MANAGER.assume_init_mut().receive_message(port_id) }
}

pub fn ipc_close_port(port_id: usize) -> isize {
    unsafe {
        IPC_MANAGER.assume_init_mut().close_port(port_id);
        0
    }
}

// Simple stub receive returning error
pub fn ipc_recv(_port_id: usize, _buf: *mut u8, _len: usize) -> isize {
    -1
}

// Simple byte-oriented send helper for boot stub VFS calls
pub fn ipc_send_simple(_port_id: usize, _buf: *const u8, _len: usize) -> isize {
    0
}

// Microkernel communication channels
pub struct MicrokernelChannels {
    pub space_port: usize,
    pub time_port: usize,
    pub ipc_port: usize,
}

impl MicrokernelChannels {
    pub fn new() -> Option<Self> {
        unsafe {
            let manager = IPC_MANAGER.assume_init_mut();
            let space_port = manager.create_port(0)?;
            let time_port = manager.create_port(0)?;
            let ipc_port = manager.create_port(0)?;

            Some(MicrokernelChannels {
                space_port,
                time_port,
                ipc_port,
            })
        }
    }

    pub fn send_to_space(&mut self, mut msg: Message) -> bool {
        msg.receiver_pid = 1; // µK-Space PID
        unsafe {
            IPC_MANAGER
                .assume_init_mut()
                .send_message(self.space_port, msg)
        }
    }

    pub fn send_to_time(&mut self, mut msg: Message) -> bool {
        msg.receiver_pid = 2; // µK-Time PID
        unsafe {
            IPC_MANAGER
                .assume_init_mut()
                .send_message(self.time_port, msg)
        }
    }

    pub fn send_to_ipc(&mut self, mut msg: Message) -> bool {
        msg.receiver_pid = 3; // µK-IPC PID
        unsafe {
            IPC_MANAGER
                .assume_init_mut()
                .send_message(self.ipc_port, msg)
        }
    }
}

// Global microkernel communication channels
pub static mut MICROKERNEL_CHANNELS: Option<MicrokernelChannels> = None;

pub fn init_microkernel_channels() -> bool {
    unsafe {
        MICROKERNEL_CHANNELS = MicrokernelChannels::new();
        MICROKERNEL_CHANNELS.is_some()
    }
}

// Server communication channels
pub struct ServerChannels {
    pub init_port: usize,
    pub vfs_port: usize,
    pub ramfs_port: usize,
    pub devd_port: usize,
}

impl ServerChannels {
    pub fn new() -> Option<Self> {
        unsafe {
            let manager = IPC_MANAGER.assume_init_mut();
            let init_port = manager.create_port(0)?;
            let vfs_port = manager.create_port(0)?;
            let ramfs_port = manager.create_port(0)?;
            let devd_port = manager.create_port(0)?;

            Some(ServerChannels {
                init_port,
                vfs_port,
                ramfs_port,
                devd_port,
            })
        }
    }

    pub fn send_to_init(&mut self, mut msg: Message) -> bool {
        if let Some((_, pid)) = lookup_service("init") {
            msg.receiver_pid = pid;
        } else {
            return false;
        }
        unsafe {
            IPC_MANAGER
                .assume_init_mut()
                .send_message(self.init_port, msg)
        }
    }

    pub fn send_to_vfs(&mut self, mut msg: Message) -> bool {
        msg.receiver_pid = 5; // VFS server PID
        unsafe {
            IPC_MANAGER
                .assume_init_mut()
                .send_message(self.vfs_port, msg)
        }
    }

    pub fn send_to_ramfs(&mut self, mut msg: Message) -> bool {
        msg.receiver_pid = 6; // RAMFS server PID
        unsafe {
            IPC_MANAGER
                .assume_init_mut()
                .send_message(self.ramfs_port, msg)
        }
    }

    pub fn send_to_devd(&mut self, mut msg: Message) -> bool {
        msg.receiver_pid = 7; // DEVD server PID
        unsafe {
            IPC_MANAGER
                .assume_init_mut()
                .send_message(self.devd_port, msg)
        }
    }
}

// Global server communication channels
pub static mut SERVER_CHANNELS: Option<ServerChannels> = None;

pub fn init_server_channels() -> bool {
    unsafe {
        SERVER_CHANNELS = ServerChannels::new();
        SERVER_CHANNELS.is_some()
    }
}

// Service registry for inter-server coordination
pub struct ServiceRegistry {
    services: [Option<ServiceInfo>; 16],
}

#[derive(Clone, Copy)]
pub struct ServiceInfo {
    pub name: [u8; 32],
    pub port: usize,
    pub pid: usize,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        ServiceRegistry {
            services: [None; 16],
        }
    }

    pub fn register(&mut self, name: &str, port: usize, pid: usize) -> bool {
        let mut name_bytes = [0u8; 32];
        let name_data = name.as_bytes();
        let copy_len = name_data.len().min(31);
        name_bytes[..copy_len].copy_from_slice(&name_data[..copy_len]);

        for i in 0..16 {
            if self.services[i].is_none() {
                self.services[i] = Some(ServiceInfo {
                    name: name_bytes,
                    port,
                    pid,
                });
                return true;
            }
        }
        false
    }

    pub fn lookup(&self, name: &str) -> Option<&ServiceInfo> {
        for service in &self.services {
            if let Some(svc) = service {
                // Compare names
                let svc_name = core::str::from_utf8(&svc.name).unwrap_or("");
                if svc_name.trim_end_matches('\0') == name {
                    return Some(svc);
                }
            }
        }
        None
    }
}

// Global service registry
pub static mut SERVICE_REGISTRY: ServiceRegistry = ServiceRegistry {
    services: [None; 16],
};

pub fn register_service(name: &str, port: usize, pid: usize) -> bool {
    unsafe { SERVICE_REGISTRY.register(name, port, pid) }
}

pub fn lookup_service(name: &str) -> Option<(usize, usize)> {
    unsafe { SERVICE_REGISTRY.lookup(name).map(|svc| (svc.port, svc.pid)) }
}
