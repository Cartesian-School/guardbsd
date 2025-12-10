#![no_std]

use core::cell::UnsafeCell;
use core::fmt;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// Recursion protection: per-CPU flag to prevent recursive logging
#[cfg(target_arch = "x86_64")]
static LOGGING_ACTIVE: AtomicBool = AtomicBool::new(false);

#[cfg(target_arch = "aarch64")]
static LOGGING_ACTIVE: AtomicBool = AtomicBool::new(false);

// Tunables
pub const LOG_RING_SIZE: usize = 256;
pub const LOG_MSG_MAX: usize = 192;
pub const LOG_SUBSYS_MAX: usize = 32;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl LogLevel {
    #[inline]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

pub type ThreadId = u64;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LogRecord {
    pub ts: u64,
    pub level: LogLevel,
    pub subsystem: &'static str,
    pub msg: [u8; LOG_MSG_MAX],
    pub len: u16,
    pub cpu_id: u8,
    pub tid: ThreadId,
}

impl LogRecord {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ts: 0,
            level: LogLevel::Info,
            subsystem: "",
            msg: [0; LOG_MSG_MAX],
            len: 0,
            cpu_id: 0,
            tid: 0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct UserLogRecord {
    pub ts: u64,
    pub level: LogLevel,
    pub cpu_id: u8,
    pub tid: ThreadId,
    pub msg_len: u16,
    pub subsystem_len: u8,
    pub msg: [u8; LOG_MSG_MAX],
    pub subsystem: [u8; LOG_SUBSYS_MAX],
}

impl UserLogRecord {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            ts: 0,
            level: LogLevel::Info,
            cpu_id: 0,
            tid: 0,
            msg_len: 0,
            subsystem_len: 0,
            msg: [0; LOG_MSG_MAX],
            subsystem: [0; LOG_SUBSYS_MAX],
        }
    }

    fn from_kernel(rec: &LogRecord) -> Self {
        let mut msg = [0u8; LOG_MSG_MAX];
        let copy_len = core::cmp::min(rec.len as usize, LOG_MSG_MAX);
        msg[..copy_len].copy_from_slice(&rec.msg[..copy_len]);

        let mut subsystem = [0u8; LOG_SUBSYS_MAX];
        let subsys_bytes = rec.subsystem.as_bytes();
        let subsys_len = core::cmp::min(subsys_bytes.len(), LOG_SUBSYS_MAX);
        subsystem[..subsys_len].copy_from_slice(&subsys_bytes[..subsys_len]);

        Self {
            ts: rec.ts,
            level: rec.level,
            cpu_id: rec.cpu_id,
            tid: rec.tid,
            msg_len: u16::try_from(copy_len).unwrap_or(0),
            subsystem_len: u8::try_from(subsys_len).unwrap_or(0),
            msg,
            subsystem,
        }
    }
}

#[derive(Copy, Clone)]
pub struct LoggerCallbacks {
    pub timestamp: fn() -> u64,
    pub cpu_id: fn() -> u8,
    pub thread_id: fn() -> ThreadId,
    pub early_print: fn(&str),
}

impl LoggerCallbacks {
    pub const fn new(
        timestamp: fn() -> u64,
        cpu_id: fn() -> u8,
        thread_id: fn() -> ThreadId,
        early_print: fn(&str),
    ) -> Self {
        Self {
            timestamp,
            cpu_id,
            thread_id,
            early_print,
        }
    }

    #[must_use]
    pub const fn default() -> Self {
        Self {
            timestamp: default_timestamp,
            cpu_id: default_cpu_id,
            thread_id: default_thread_id,
            early_print: default_early_print,
        }
    }
}

static FALLBACK_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn default_timestamp() -> u64 {
    FALLBACK_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[must_use]
pub fn default_cpu_id() -> u8 {
    0
}

#[must_use]
pub fn default_thread_id() -> ThreadId {
    0
}

pub fn default_early_print(msg: &str) {
    log_backend::write_bytes(msg.as_bytes());
}

// --------------------------------------------------------------------------
// Logger backends: serial + in-memory ring + VFS stub
// --------------------------------------------------------------------------

pub mod log_backend {
    use core::sync::atomic::{AtomicBool, Ordering};
    use super::SpinLock;

    const MEM_RING_SIZE: usize = 4096;

    static SERIAL_ENABLED: AtomicBool = AtomicBool::new(true);
    static MEM_ENABLED: AtomicBool = AtomicBool::new(true);
    static VFS_ENABLED: AtomicBool = AtomicBool::new(true); // stub for future file-backed logging
    type ExternalSinkFn = fn(&[u8]);
    static EXTERNAL_SINK_ENABLED: AtomicBool = AtomicBool::new(false);
    static mut EXTERNAL_SINK: Option<ExternalSinkFn> = None;

    struct MemRing {
        buf: [u8; MEM_RING_SIZE],
        head: usize,
        tail: usize,
        full: bool,
    }

    impl MemRing {
        const fn new() -> Self {
            Self {
                buf: [0; MEM_RING_SIZE],
                head: 0,
                tail: 0,
                full: false,
            }
        }

        fn len(&self) -> usize {
            if self.full {
                MEM_RING_SIZE
            } else if self.head >= self.tail {
                self.head - self.tail
            } else {
                MEM_RING_SIZE - self.tail + self.head
            }
        }

        fn push_bytes(&mut self, data: &[u8]) {
            for &b in data {
                self.buf[self.head] = b;
                self.head = (self.head + 1) % MEM_RING_SIZE;
                if self.full {
                    self.tail = self.head;
                } else if self.head == self.tail {
                    self.full = true;
                }
            }
        }

        fn copy_out(&self, out: &mut [u8]) -> usize {
            let available = self.len();
            let count = core::cmp::min(out.len(), available);
            for i in 0..count {
                let idx = (self.tail + i) % MEM_RING_SIZE;
                out[i] = self.buf[idx];
            }
            count
        }
    }

    static MEM_RING: SpinLock<MemRing> = SpinLock::new(MemRing::new());

    fn serial_write(data: &[u8]) {
        const COM1: u16 = 0x3F8;
        for &byte in data {
            unsafe {
                while (core::ptr::read_volatile((COM1 + 5) as *const u8) & 0x20) == 0 {}
                core::ptr::write_volatile(COM1 as *mut u8, byte);
            }
        }
    }

    #[inline]
    pub fn enable_serial(enable: bool) {
        SERIAL_ENABLED.store(enable, Ordering::Relaxed);
    }

    #[inline]
    pub fn enable_mem_ring(enable: bool) {
        MEM_ENABLED.store(enable, Ordering::Relaxed);
    }

    #[inline]
    pub fn enable_vfs_stub(enable: bool) {
        VFS_ENABLED.store(enable, Ordering::Relaxed);
    }

    /// Core write entry for all backends.
    pub fn write_bytes(data: &[u8]) {
        if data.is_empty() {
            return;
        }
        if SERIAL_ENABLED.load(Ordering::Relaxed) {
            serial_write(data);
        }
        if MEM_ENABLED.load(Ordering::Relaxed) {
            if let Some(mut ring) = MEM_RING.try_lock() {
                ring.push_bytes(data);
            }
        }
        if VFS_ENABLED.load(Ordering::Relaxed) {
            // TODO: persist to VFS file when VFS is available
        }
        if EXTERNAL_SINK_ENABLED.load(Ordering::Relaxed) {
            if let Some(sink) = unsafe { EXTERNAL_SINK } {
                sink(data);
            }
        }
    }

    /// Copy bytes from the in-memory ring buffer for diagnostics.
    #[must_use]
    pub fn copy_mem(out: &mut [u8]) -> usize {
        if let Some(ring) = MEM_RING.try_lock() {
            ring.copy_out(out)
        } else {
            0
        }
    }

    #[must_use]
    pub fn serial_enabled() -> bool {
        SERIAL_ENABLED.load(Ordering::Relaxed)
    }

    /// Register an external sink callback (non-blocking, best effort).
    pub fn set_external_sink(sink: Option<ExternalSinkFn>) {
        unsafe {
            EXTERNAL_SINK = sink;
        }
        EXTERNAL_SINK_ENABLED.store(sink.is_some(), Ordering::Relaxed);
    }
}

// --------------------------------------------------------------------------
// Public API
// --------------------------------------------------------------------------
// SAFETY MODEL:
// - ISR logging: try_lock() + serial fallback (never blocks)
// - Syscall read: try_lock() + return 0 records (never blocks)
// - Syscall ack: try_lock() + return E_AGAIN (never blocks)
// - Kernel internal: unchanged (may use blocking locks if not in syscall)
//
// INVARIANTS:
// - Syscalls never block on ring buffer locks
// - Syscalls are not allowed to run concurrently with ISR logging
// - Syscalls return appropriate errors when contention happens
// - Recursion guard prevents infinite logging loops
// --------------------------------------------------------------------------

pub fn init(callbacks: LoggerCallbacks) {
    LOGGER.init(callbacks);
}

pub fn log(level: LogLevel, subsystem: &'static str, args: fmt::Arguments) {
    LOGGER.record(level, subsystem, args);
}

/// Read records from the kernel log ring buffer.
/// Returns the number of records copied, or 0 if ring is busy.
pub fn read_records(out: &mut [UserLogRecord]) -> usize {
    LOGGER.read(out).unwrap_or_default()
}

/// Acknowledge records in the kernel log ring buffer.
///
/// # Errors
///
/// Returns `Err(())` if the ring buffer is busy.
#[allow(clippy::result_unit_err)]
pub fn ack_records(count: usize) -> Result<(), ()> {
    LOGGER.ack(count)
}

pub fn set_callbacks(callbacks: LoggerCallbacks) {
    LOGGER.set_callbacks(callbacks);
}

// --------------------------------------------------------------------------
// Macros
// --------------------------------------------------------------------------

#[macro_export]
macro_rules! klog_trace {
    ($subsystem:expr, $($arg:tt)*) => {
        $crate::log($crate::LogLevel::Trace, $subsystem, core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! klog_debug {
    ($subsystem:expr, $($arg:tt)*) => {
        $crate::log($crate::LogLevel::Debug, $subsystem, core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! klog_info {
    ($subsystem:expr, $($arg:tt)*) => {
        $crate::log($crate::LogLevel::Info, $subsystem, core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! klog_warn {
    ($subsystem:expr, $($arg:tt)*) => {
        $crate::log($crate::LogLevel::Warn, $subsystem, core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! klog_error {
    ($subsystem:expr, $($arg:tt)*) => {
        $crate::log($crate::LogLevel::Error, $subsystem, core::format_args!($($arg)*));
    };
}

// --------------------------------------------------------------------------
// Logger core
// --------------------------------------------------------------------------

static LOGGER: Logger = Logger::new();

struct Logger {
    ring: SpinLock<LogRing>,
    callbacks: CallbackCell,
    ready: AtomicBool,
}

impl Logger {
    const fn new() -> Self {
        Self {
            ring: SpinLock::new(LogRing::new()),
            callbacks: CallbackCell::new(LoggerCallbacks::default()),
            ready: AtomicBool::new(false),
        }
    }

    fn init(&self, callbacks: LoggerCallbacks) {
        self.set_callbacks(callbacks);
        self.ready.store(true, Ordering::Release);
    }

    fn set_callbacks(&self, callbacks: LoggerCallbacks) {
        self.callbacks.set(callbacks);
    }

    fn record(&self, level: LogLevel, subsystem: &'static str, args: fmt::Arguments) {
        // Recursion protection: prevent logging while already logging
        if LOGGING_ACTIVE
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // Recursion detected: use serial fallback only
            let mut msg_buf = MsgBuf::new();
            let _ = fmt::write(&mut msg_buf, args);
            let cb = self.callbacks.get();
            let mut early = EarlyBuf::new();
            let _ = early.write_prefix(level, subsystem);
            let _ = early.write_str(msg_buf.as_str());
            (cb.early_print)(early.as_str());
            return;
        }

        let mut msg_buf = MsgBuf::new();
        let _ = fmt::write(&mut msg_buf, args);

        let cb = self.callbacks.get();
        if self.ready.load(Ordering::Acquire) {
            let msg_len = u16::try_from(msg_buf.len).unwrap_or(0);
            let msg_arr = msg_buf.as_array();
            let record = LogRecord {
                ts: (cb.timestamp)(),
                level,
                subsystem,
                msg: msg_arr,
                len: msg_len,
                cpu_id: (cb.cpu_id)(),
                tid: (cb.thread_id)(),
            };

            // Deadlock prevention: use try_lock in ISR context
            // If try_lock fails, fall back to serial-only logging
            if let Some(mut ring) = self.ring.try_lock() {
                ring.push(record);
            } else {
                // ISR context or lock contention: serial fallback
                let mut early = EarlyBuf::new();
                let _ = early.write_prefix(level, subsystem);
                let _ = early.write_str(msg_buf.as_str());
                (cb.early_print)(early.as_str());
            }
        } else {
            // Early boot fallback: serial-only
            let mut early = EarlyBuf::new();
            let _ = early.write_prefix(level, subsystem);
            let _ = early.write_str(msg_buf.as_str());
            (cb.early_print)(early.as_str());
        }

        // Clear recursion flag
        LOGGING_ACTIVE.store(false, Ordering::Release);
    }

    /// Read records from the ring buffer.
    /// Returns Ok(count) on success, Err(()) if lock is busy (ISR contention).
    fn read(&self, out: &mut [UserLogRecord]) -> Result<usize, ()> {
        if out.is_empty() {
            return Ok(0);
        }

        // Non-blocking: return error if ring is busy (ISR logging in progress)
        if let Some(ring) = self.ring.try_lock() {
            Ok(ring.copy_out(out))
        } else {
            Err(())
        }
    }

    /// Acknowledge records (remove from ring buffer).
    /// Returns Ok(()) on success, Err(()) if lock is busy (ISR contention).
    fn ack(&self, count: usize) -> Result<(), ()> {
        if count == 0 {
            return Ok(());
        }

        // Non-blocking: return error if ring is busy (ISR logging in progress)
        if let Some(mut ring) = self.ring.try_lock() {
            ring.ack(count);
            Ok(())
        } else {
            Err(())
        }
    }
}

// --------------------------------------------------------------------------
// Ring buffer with minimal locking
// --------------------------------------------------------------------------

struct LogRing {
    buf: [LogRecord; LOG_RING_SIZE],
    head: usize,
    tail: usize,
    full: bool,
}

impl LogRing {
    const fn new() -> Self {
        Self {
            #[allow(clippy::large_stack_arrays)]
            buf: [LogRecord::empty(); LOG_RING_SIZE],
            head: 0,
            tail: 0,
            full: false,
        }
    }

    fn push(&mut self, record: LogRecord) {
        // Validate record before insertion
        if record.len > u16::try_from(LOG_MSG_MAX).unwrap_or(u16::MAX) {
            // Error: record too large - use serial fallback
            // DO NOT call klog_* here to avoid recursion
            serial_error_print(b"[KLOG-ERR] record too large");
            return;
        }

        self.buf[self.head] = record;
        self.head = (self.head + 1) % LOG_RING_SIZE;

        if self.full {
            self.tail = self.head;
        } else if self.head == self.tail {
            self.full = true;
        }
    }

    fn len(&self) -> usize {
        if self.full {
            LOG_RING_SIZE
        } else if self.head >= self.tail {
            self.head - self.tail
        } else {
            LOG_RING_SIZE - self.tail + self.head
        }
    }

    fn copy_out(&self, out: &mut [UserLogRecord]) -> usize {
        let available = self.len();
        let count = core::cmp::min(available, out.len());

        #[allow(clippy::needless_range_loop)]
        for i in 0..count {
            let idx = (self.tail + i) % LOG_RING_SIZE;
            out[i] = UserLogRecord::from_kernel(&self.buf[idx]);
        }
        count
    }

    fn ack(&mut self, count: usize) {
        let available = self.len();
        let to_drop = core::cmp::min(count, available);
        if to_drop == LOG_RING_SIZE {
            self.tail = self.head;
            self.full = false;
        } else {
            self.tail = (self.tail + to_drop) % LOG_RING_SIZE;
            self.full = false;
        }
    }
}

// --------------------------------------------------------------------------
// Minimal spin lock
// --------------------------------------------------------------------------

pub struct SpinLock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.lock.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }
        SpinLockGuard { lock: self }
    }

    /// Try to acquire the lock without blocking.
    /// Returns Some(guard) if successful, None if already locked.
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        if self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(SpinLockGuard { lock: self })
        } else {
            None
        }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.lock.store(false, Ordering::Release);
    }
}

impl<T> core::ops::Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> core::ops::DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

// --------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------

struct MsgBuf {
    buf: [u8; LOG_MSG_MAX],
    len: usize,
}

impl MsgBuf {
    fn new() -> Self {
        Self {
            buf: [0; LOG_MSG_MAX],
            len: 0,
        }
    }

    fn as_array(&self) -> [u8; LOG_MSG_MAX] {
        self.buf
    }

    fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }
}

impl fmt::Write for MsgBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.as_bytes() {
            if self.len < LOG_MSG_MAX {
                self.buf[self.len] = *b;
                self.len += 1;
            } else {
                break;
            }
        }
        Ok(())
    }
}

struct EarlyBuf {
    buf: [u8; LOG_MSG_MAX + LOG_SUBSYS_MAX + 16],
    len: usize,
}

impl EarlyBuf {
    fn new() -> Self {
        Self {
            buf: [0; LOG_MSG_MAX + LOG_SUBSYS_MAX + 16],
            len: 0,
        }
    }

    fn write_prefix(&mut self, level: LogLevel, subsystem: &'static str) -> fmt::Result {
        let _ = self.write_str("[");
        let _ = self.write_str(level.as_str());
        let _ = self.write_str("][");
        self.write_str(subsystem)?;
        self.write_str("] ")
    }

    fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.buf[..self.len]) }
    }
}

impl fmt::Write for EarlyBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.as_bytes() {
            if self.len < self.buf.len() {
                self.buf[self.len] = *b;
                self.len += 1;
            } else {
                break;
            }
        }
        Ok(())
    }
}

struct CallbackCell {
    value: UnsafeCell<LoggerCallbacks>,
}

unsafe impl Sync for CallbackCell {}

impl CallbackCell {
    const fn new(cb: LoggerCallbacks) -> Self {
        Self {
            value: UnsafeCell::new(cb),
        }
    }

    fn set(&self, cb: LoggerCallbacks) {
        unsafe {
            *self.value.get() = cb;
        }
    }

    fn get(&self) -> LoggerCallbacks {
        unsafe { *self.value.get() }
    }
}

// Error printing without logging (prevents recursion)
fn serial_error_print(msg: &[u8]) {
    // Direct COM1 write to guarantee visibility even if backends are misconfigured
    const COM1: u16 = 0x3F8;
    for &byte in msg {
        unsafe {
            while (core::ptr::read_volatile((COM1 + 5) as *const u8) & 0x20) == 0 {}
            core::ptr::write_volatile(COM1 as *mut u8, byte);
        }
    }
    unsafe {
        while (core::ptr::read_volatile((COM1 + 5) as *const u8) & 0x20) == 0 {}
        core::ptr::write_volatile(COM1 as *mut u8, b'\n');
    }
    // Also forward into unified logging path (may duplicate serial if enabled)
    log_backend::write_bytes(msg);
    log_backend::write_bytes(b"\n");
}
