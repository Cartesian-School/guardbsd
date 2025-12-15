//! kernel/boot_stub/src/ipc.rs
//! Project: GuardBSD Winter Saga version 1.0.0
//! Package: boot_stub
//! Copyright © 2025 Cartesian School.
//! License: BSD-3-Clause
//!
//! IPC infrastructure for boot stub (simple ports + message ring).
//!
//! This version adds an IRQ-safe spinlock:
//! - On x86_64: lock() disables interrupts (CLI) and restores previous IF on unlock.
//! - On other arch: interrupts are not touched, but mutual exclusion still holds.
//!
//! Rationale:
//! - If IRQ handlers can call IPC (directly or indirectly), we must prevent races
//!   with normal code paths. Disabling IRQs around lock acquisition prevents
//!   deadlocks from same-CPU interrupt reentrancy.

#![allow(dead_code)]

use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

const MAX_PORTS: usize = 64;
const PORT_QUEUE_LEN: usize = 16;

// Minimal errno subset (negative Linux-style values).
const EINVAL: isize = -22;
const EAGAIN: isize = -11;
const ENOSYS: isize = -38;
const EPIPE: isize = -32;

// ============================================================================
// IRQ-safe SpinLock (x86_64)
// ============================================================================
//
// IMPORTANT:
// - Do NOT lie to the compiler with `nomem`/`preserves_flags` here.
//   `pushfq/pop` touches stack memory, and `cli/sti` modify flags.
//

#[inline(always)]
#[cfg(target_arch = "x86_64")]
unsafe fn irq_save_disable() -> u64 {
    let rflags: u64;
    // Reads current RFLAGS into a GP register (touches stack).
    core::arch::asm!(
        "pushfq",
        "pop {}",
        out(reg) rflags,
        // no options: this uses stack memory and affects compiler assumptions
    );
    // Disable interrupts (modifies IF flag).
    core::arch::asm!("cli");
    rflags
}

#[inline(always)]
#[cfg(target_arch = "x86_64")]
unsafe fn irq_restore(saved_rflags: u64) {
    // IF is bit 9 in RFLAGS.
    const IF_MASK: u64 = 1 << 9;
    if (saved_rflags & IF_MASK) != 0 {
        core::arch::asm!("sti");
    }
}

#[inline(always)]
#[cfg(not(target_arch = "x86_64"))]
unsafe fn irq_save_disable() -> u64 {
    0
}

#[inline(always)]
#[cfg(not(target_arch = "x86_64"))]
unsafe fn irq_restore(_saved_rflags: u64) {}

pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for SpinLock<T> {}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
    saved_rflags: u64,
}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    /// Acquire the lock.
    ///
    /// On x86_64 this disables IRQs before spinning to avoid same-CPU
    /// interrupt reentrancy deadlocks.
    #[inline(always)]
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        let saved = unsafe { irq_save_disable() };

        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            spin_loop();
        }

        SpinLockGuard {
            lock: self,
            saved_rflags: saved,
        }
    }
}

impl<T> core::ops::Deref for SpinLockGuard<'_, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> core::ops::DerefMut for SpinLockGuard<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {
        // Release lock first, then restore IRQ state.
        // This prevents an interrupt from re-entering and seeing the lock still held.
        self.lock.locked.store(false, Ordering::Release);
        unsafe { irq_restore(self.saved_rflags) };
    }
}

// ============================================================================
// IPC core types
// ============================================================================

// Message structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Message {
    pub sender_pid: usize,
    pub receiver_pid: usize,
    pub msg_type: u32,
    pub data: [u32; 4], // 16-byte payload
}

// Port structure
#[repr(C)]
pub struct Port {
    pub port_id: usize,
    pub owner_pid: usize,
    pub messages: [Option<Message>; PORT_QUEUE_LEN], // ring buffer
    pub read_idx: usize,
    pub write_idx: usize,
    pub is_open: AtomicBool,
}

impl Port {
    pub const fn new(port_id: usize, owner_pid: usize) -> Self {
        Port {
            port_id,
            owner_pid,
            messages: [None; PORT_QUEUE_LEN],
            read_idx: 0,
            write_idx: 0,
            is_open: AtomicBool::new(true),
        }
    }

    #[inline(always)]
    pub fn send(&mut self, msg: Message) -> bool {
        if !self.is_open.load(Ordering::Acquire) {
            return false;
        }
        let next_write = (self.write_idx + 1) % PORT_QUEUE_LEN;
        if next_write == self.read_idx {
            // queue full
            return false;
        }
        self.messages[self.write_idx] = Some(msg);
        self.write_idx = next_write;
        true
    }

    #[inline(always)]
    pub fn receive(&mut self) -> Option<Message> {
        if !self.is_open.load(Ordering::Acquire) {
            return None;
        }
        if self.read_idx == self.write_idx {
            return None;
        }
        let msg = self.messages[self.read_idx];
        self.messages[self.read_idx] = None;
        self.read_idx = (self.read_idx + 1) % PORT_QUEUE_LEN;
        msg
    }

    #[inline(always)]
    pub fn close(&mut self) {
        self.is_open.store(false, Ordering::Release);
    }
}

// IPC Manager (protected by SpinLock globally)
pub struct IpcManager {
    ports: [MaybeUninit<Port>; MAX_PORTS],
    used: [bool; MAX_PORTS],
    next_port_id: usize,
}

impl IpcManager {
    /// Const constructor: no unsafe required.
    pub const fn new() -> Self {
        // `MaybeUninit<T>` is fine to repeat in const arrays.
        Self {
            ports: [MaybeUninit::<Port>::uninit(); MAX_PORTS],
            used: [false; MAX_PORTS],
            next_port_id: 1, // port 0 reserved
        }
    }

    #[inline(always)]
    fn is_valid_id(port_id: usize) -> bool {
        port_id < MAX_PORTS
    }

    pub fn create_port(&mut self, owner_pid: usize) -> Option<usize> {
        let mut start = self.next_port_id;
        if start == 0 {
            start = 1;
        }
        if start >= MAX_PORTS {
            start = 1;
        }

        // One full scan
        for _ in 0..MAX_PORTS {
            let id = start;
            start += 1;
            if start >= MAX_PORTS {
                start = 1;
            }

            if !self.used[id] {
                self.used[id] = true;
                self.ports[id].write(Port::new(id, owner_pid));
                self.next_port_id = start;
                return Some(id);
            }
        }

        None
    }

    pub fn get_port(&self, port_id: usize) -> Option<&Port> {
        if !Self::is_valid_id(port_id) || !self.used[port_id] {
            return None;
        }
        Some(unsafe { self.ports[port_id].assume_init_ref() })
    }

    pub fn get_port_mut(&mut self, port_id: usize) -> Option<&mut Port> {
        if !Self::is_valid_id(port_id) || !self.used[port_id] {
            return None;
        }
        Some(unsafe { self.ports[port_id].assume_init_mut() })
    }

    pub fn send_message(&mut self, port_id: usize, msg: Message) -> bool {
        self.get_port_mut(port_id).map(|p| p.send(msg)).unwrap_or(false)
    }

    pub fn receive_message(&mut self, port_id: usize) -> Option<Message> {
        self.get_port_mut(port_id).and_then(|p| p.receive())
    }

    pub fn close_port(&mut self, port_id: usize) {
        if let Some(p) = self.get_port_mut(port_id) {
            p.close();
        }
    }
}

// ============================================================================
// Global IPC state (protected)
// ============================================================================

static IPC_READY: AtomicBool = AtomicBool::new(false);

// IPC_MANAGER is protected by SpinLock, so access is race-free even with IRQs enabled.
static IPC_MANAGER_LOCK: SpinLock<MaybeUninit<IpcManager>> = SpinLock::new(MaybeUninit::uninit());

/// Idempotent initialization (race-safe).
pub fn init_ipc() {
    if IPC_READY.load(Ordering::Acquire) {
        return;
    }

    let mut guard = IPC_MANAGER_LOCK.lock();

    // Second check under lock: prevents double-init under concurrency.
    if IPC_READY.load(Ordering::Relaxed) {
        return;
    }

    guard.write(IpcManager::new());

    // Publish after the manager is written.
    IPC_READY.store(true, Ordering::Release);
}

#[inline(always)]
fn ipc_ready() -> bool {
    IPC_READY.load(Ordering::Acquire)
}

#[inline(always)]
fn with_ipc_manager_mut<R>(f: impl FnOnce(&mut IpcManager) -> R) -> Option<R> {
    if !ipc_ready() {
        return None;
    }
    let mut guard = IPC_MANAGER_LOCK.lock();
    // Safety: init_ipc writes IpcManager before IPC_READY becomes true.
    let mgr = unsafe { guard.assume_init_mut() };
    Some(f(mgr))
}

#[inline(always)]
fn with_ipc_manager<R>(f: impl FnOnce(&IpcManager) -> R) -> Option<R> {
    if !ipc_ready() {
        return None;
    }
    let guard = IPC_MANAGER_LOCK.lock();
    // Safety: init_ipc writes IpcManager before IPC_READY becomes true.
    let mgr = unsafe { guard.assume_init_ref() };
    Some(f(mgr))
}

// ============================================================================
// IPC API (boot stub)
// ============================================================================

pub fn ipc_create_port(owner_pid: usize) -> isize {
    match with_ipc_manager_mut(|mgr| mgr.create_port(owner_pid)) {
        Some(Some(id)) => id as isize,
        Some(None) => EAGAIN,
        None => ENOSYS,
    }
}

pub fn ipc_send(
    port_id: usize,
    sender_pid: usize,
    receiver_pid: usize,
    msg_type: u32,
    data: [u32; 4],
) -> isize {
    if !ipc_ready() {
        return ENOSYS;
    }

    let msg = Message {
        sender_pid,
        receiver_pid,
        msg_type,
        data,
    };

    // TODO: refine error codes:
    // - if port closed => EPIPE
    // - if queue full  => EAGAIN
    match with_ipc_manager_mut(|mgr| mgr.send_message(port_id, msg)) {
        Some(true) => 0,
        Some(false) => EPIPE,
        None => ENOSYS,
    }
}

pub fn ipc_receive(port_id: usize) -> Option<Message> {
    with_ipc_manager_mut(|mgr| mgr.receive_message(port_id)).and_then(|x| x)
}

pub fn ipc_close_port(port_id: usize) -> isize {
    if !ipc_ready() {
        return ENOSYS;
    }
    let _ = with_ipc_manager_mut(|mgr| mgr.close_port(port_id));
    0
}

/// Minimal receive helper for byte-oriented protocols.
/// Copies up to 16 bytes (Message.data) into `buf`.
pub fn ipc_recv(port_id: usize, buf: *mut u8, len: usize) -> isize {
    if !ipc_ready() {
        return ENOSYS;
    }
    if buf.is_null() || len == 0 {
        return EINVAL;
    }

    let msg = match ipc_receive(port_id) {
        Some(m) => m,
        None => return EAGAIN,
    };

    let src_bytes = unsafe {
        core::slice::from_raw_parts((&msg.data as *const [u32; 4]) as *const u8, 16)
    };
    let copy_len = core::cmp::min(len, 16);

    unsafe {
        core::ptr::copy_nonoverlapping(src_bytes.as_ptr(), buf, copy_len);
    }

    copy_len as isize
}

/// Minimal send helper for byte-oriented protocols.
/// Packs up to 16 bytes from `buf` into Message.data and sends it.
pub fn ipc_send_simple(port_id: usize, buf: *const u8, len: usize) -> isize {
    if !ipc_ready() {
        return ENOSYS;
    }
    if buf.is_null() || len == 0 {
        return EINVAL;
    }

    let mut data = [0u32; 4];
    let copy_len = core::cmp::min(len, 16);

    unsafe {
        core::ptr::copy_nonoverlapping(buf, data.as_mut_ptr() as *mut u8, copy_len);
    }

    // Boot stub: sender_pid=0, receiver_pid not enforced here.
    ipc_send(port_id, 0, 0, 0, data)
}

// ============================================================================
// Microkernel and server channels
// ============================================================================

pub struct MicrokernelChannels {
    pub space_port: usize,
    pub time_port: usize,
    pub ipc_port: usize,
}

impl MicrokernelChannels {
    pub fn new() -> Option<Self> {
        let (space_port, time_port, ipc_port) =
            with_ipc_manager_mut(|mgr| {
                let s = mgr.create_port(0)?;
                let t = mgr.create_port(0)?;
                let i = mgr.create_port(0)?;
                Some((s, t, i))
            })
            .and_then(|x| x)?;

        Some(MicrokernelChannels {
            space_port,
            time_port,
            ipc_port,
        })
    }

    pub fn send_to_space(&mut self, mut msg: Message) -> bool {
        msg.receiver_pid = 1; // µK-Space PID
        with_ipc_manager_mut(|mgr| mgr.send_message(self.space_port, msg)).unwrap_or(false)
    }

    pub fn send_to_time(&mut self, mut msg: Message) -> bool {
        msg.receiver_pid = 2; // µK-Time PID
        with_ipc_manager_mut(|mgr| mgr.send_message(self.time_port, msg)).unwrap_or(false)
    }

    pub fn send_to_ipc(&mut self, mut msg: Message) -> bool {
        msg.receiver_pid = 3; // µK-IPC PID
        with_ipc_manager_mut(|mgr| mgr.send_message(self.ipc_port, msg)).unwrap_or(false)
    }
}

pub static mut MICROKERNEL_CHANNELS: Option<MicrokernelChannels> = None;

pub fn init_microkernel_channels() -> bool {
    unsafe {
        MICROKERNEL_CHANNELS = MicrokernelChannels::new();
        MICROKERNEL_CHANNELS.is_some()
    }
}

pub struct ServerChannels {
    pub init_port: usize,
    pub vfs_port: usize,
    pub ramfs_port: usize,
    pub devd_port: usize,
}

impl ServerChannels {
    pub fn new() -> Option<Self> {
        let (init_port, vfs_port, ramfs_port, devd_port) =
            with_ipc_manager_mut(|mgr| {
                let a = mgr.create_port(0)?;
                let b = mgr.create_port(0)?;
                let c = mgr.create_port(0)?;
                let d = mgr.create_port(0)?;
                Some((a, b, c, d))
            })
            .and_then(|x| x)?;

        Some(ServerChannels {
            init_port,
            vfs_port,
            ramfs_port,
            devd_port,
        })
    }

    pub fn send_to_init(&mut self, mut msg: Message) -> bool {
        // Lock ordering recommendation:
        // - If you ever need both locks, keep order: SERVICE_REGISTRY_LOCK -> IPC_MANAGER_LOCK.
        // This function follows that order (registry lookup first, IPC send second).
        if let Some((_, pid)) = lookup_service("init") {
            msg.receiver_pid = pid;
        } else {
            return false;
        }
        with_ipc_manager_mut(|mgr| mgr.send_message(self.init_port, msg)).unwrap_or(false)
    }

    pub fn send_to_vfs(&mut self, mut msg: Message) -> bool {
        msg.receiver_pid = 5; // VFS server PID
        with_ipc_manager_mut(|mgr| mgr.send_message(self.vfs_port, msg)).unwrap_or(false)
    }

    pub fn send_to_ramfs(&mut self, mut msg: Message) -> bool {
        msg.receiver_pid = 6; // RAMFS server PID
        with_ipc_manager_mut(|mgr| mgr.send_message(self.ramfs_port, msg)).unwrap_or(false)
    }

    pub fn send_to_devd(&mut self, mut msg: Message) -> bool {
        msg.receiver_pid = 7; // DEVD server PID
        with_ipc_manager_mut(|mgr| mgr.send_message(self.devd_port, msg)).unwrap_or(false)
    }
}

pub static mut SERVER_CHANNELS: Option<ServerChannels> = None;

pub fn init_server_channels() -> bool {
    unsafe {
        SERVER_CHANNELS = ServerChannels::new();
        SERVER_CHANNELS.is_some()
    }
}

// ============================================================================
// Service registry (protected by SpinLock as well)
// ============================================================================

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
    pub const fn new() -> Self {
        ServiceRegistry { services: [None; 16] }
    }

    pub fn register(&mut self, name: &str, port: usize, pid: usize) -> bool {
        let mut name_bytes = [0u8; 32];
        let src = name.as_bytes();
        let n = core::cmp::min(src.len(), 31);
        name_bytes[..n].copy_from_slice(&src[..n]);
        name_bytes[n] = 0;

        for slot in self.services.iter_mut() {
            if slot.is_none() {
                *slot = Some(ServiceInfo {
                    name: name_bytes,
                    port,
                    pid,
                });
                return true;
            }
        }
        false
    }

    pub fn lookup(&self, name: &str) -> Option<ServiceInfo> {
        let src = name.as_bytes();
        if src.is_empty() || src.len() >= 32 {
            return None;
        }

        for slot in self.services.iter() {
            let svc = match slot.as_ref() {
                Some(s) => s,
                None => continue,
            };

            let mut i = 0usize;
            while i < 32 && svc.name[i] != 0 {
                if i >= src.len() || svc.name[i] != src[i] {
                    break;
                }
                i += 1;
            }

            if i == src.len() && i < 32 && svc.name[i] == 0 {
                return Some(*svc);
            }
        }

        None
    }
}

static SERVICE_REGISTRY_LOCK: SpinLock<ServiceRegistry> = SpinLock::new(ServiceRegistry::new());

pub fn register_service(name: &str, port: usize, pid: usize) -> bool {
    let mut reg = SERVICE_REGISTRY_LOCK.lock();
    reg.register(name, port, pid)
}

pub fn lookup_service(name: &str) -> Option<(usize, usize)> {
    let reg = SERVICE_REGISTRY_LOCK.lock();
    reg.lookup(name).map(|svc| (svc.port, svc.pid))
}
