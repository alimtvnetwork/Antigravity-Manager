//! Database Connection Pool Concurrency Limiter.
//! Enforces maximum concurrent read connections (max = 8) and serialized single-writer discipline per split database.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Default configuration for split database pools: up to 8 readers, 1 exclusive writer.
pub const DEFAULT_MAX_READERS: usize = 8;
pub const DEFAULT_MAX_WRITERS: usize = 1;

/// Pool concurrency configuration.
#[derive(Debug, Clone, Copy)]
pub struct PoolConcurrencyConfig {
    pub max_readers: usize,
    pub max_writers: usize,
}

impl Default for PoolConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_readers: DEFAULT_MAX_READERS,
            max_writers: DEFAULT_MAX_WRITERS,
        }
    }
}

/// Thread-safe tracker governing read and write connection permits across split databases.
#[derive(Debug, Clone)]
pub struct PoolConcurrencyTracker {
    config: PoolConcurrencyConfig,
    active_readers: Arc<AtomicUsize>,
    is_writer_active: Arc<AtomicBool>,
}

impl PoolConcurrencyTracker {
    pub fn new(config: PoolConcurrencyConfig) -> Self {
        Self {
            config,
            active_readers: Arc::new(AtomicUsize::new(0)),
            is_writer_active: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Attempts to acquire a read permit. Returns guard if reader count < max_readers and no active writer.
    pub fn try_acquire_read(&self) -> Option<ReadPermitGuard> {
        if self.is_writer_active.load(Ordering::Acquire) {
            return None;
        }

        let mut current = self.active_readers.load(Ordering::Acquire);
        loop {
            if current >= self.config.max_readers {
                return None;
            }
            match self.active_readers.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return Some(ReadPermitGuard {
                        counter: Arc::clone(&self.active_readers),
                    });
                }
                Err(actual) => current = actual,
            }
        }
    }

    /// Attempts to acquire an exclusive writer permit. Returns guard if no active readers and no active writer.
    pub fn try_acquire_write(&self) -> Option<WritePermitGuard> {
        if self.active_readers.load(Ordering::Acquire) > 0 {
            return None;
        }

        if self
            .is_writer_active
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            Some(WritePermitGuard {
                writer_flag: Arc::clone(&self.is_writer_active),
            })
        } else {
            None
        }
    }

    /// Returns current active reader count.
    pub fn active_readers(&self) -> usize {
        self.active_readers.load(Ordering::Acquire)
    }

    /// Returns whether an exclusive writer is currently active.
    pub fn is_writer_active(&self) -> bool {
        self.is_writer_active.load(Ordering::Acquire)
    }
}

impl Default for PoolConcurrencyTracker {
    fn default() -> Self {
        Self::new(PoolConcurrencyConfig::default())
    }
}

/// RAII guard releasing reader slot on drop.
pub struct ReadPermitGuard {
    counter: Arc<AtomicUsize>,
}

impl Drop for ReadPermitGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Release);
    }
}

/// RAII guard releasing exclusive writer slot on drop.
pub struct WritePermitGuard {
    writer_flag: Arc<AtomicBool>,
}

impl Drop for WritePermitGuard {
    fn drop(&mut self) {
        self.writer_flag.store(false, Ordering::Release);
    }
}
