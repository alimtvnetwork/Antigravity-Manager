//! Streaming Timeout Sentinel
//! Detects upstream SSE / chunk stream stalls exceeding inactivity timeouts (e.g. 30s)
//! and provides graceful cancellation hooks to prevent lingering sockets.

use std::time::{Duration, Instant};

/// Configuration for streaming inactivity detection.
#[derive(Debug, Clone, Copy)]
pub struct StreamingTimeoutConfig {
    /// Maximum allowed duration of silence between stream chunks before aborting.
    pub max_inactivity_duration: Duration,
    /// Absolute total generation timeout regardless of chunk activity.
    pub total_stream_timeout: Duration,
}

impl Default for StreamingTimeoutConfig {
    fn default() -> Self {
        Self {
            max_inactivity_duration: Duration::from_secs(30),
            total_stream_timeout: Duration::from_secs(600),
        }
    }
}

/// State tracker for an in-flight upstream response stream.
#[derive(Debug)]
pub struct StreamingTimeoutSentinel {
    config: StreamingTimeoutConfig,
    stream_started_at: Instant,
    last_chunk_at: Instant,
    chunks_received: u64,
    bytes_received: u64,
    is_aborted: bool,
}

impl StreamingTimeoutSentinel {
    /// Creates a new sentinel with default or custom configuration.
    pub fn new(config: Option<StreamingTimeoutConfig>) -> Self {
        let now = Instant::now();
        Self {
            config: config.unwrap_or_default(),
            stream_started_at: now,
            last_chunk_at: now,
            chunks_received: 0,
            bytes_received: 0,
            is_aborted: false,
        }
    }

    /// Records receipt of a stream chunk, resetting the inactivity timer.
    pub fn record_chunk(&mut self, chunk_len: usize) {
        self.last_chunk_at = Instant::now();
        self.chunks_received += 1;
        self.bytes_received += chunk_len as u64;
    }

    /// Checks whether the stream has stalled or exceeded its total lifetime.
    pub fn check_liveness(&mut self) -> Result<(), StreamingTimeoutError> {
        if self.is_aborted {
            return Err(StreamingTimeoutError::AlreadyAborted);
        }

        let now = Instant::now();
        let silence_duration = now.duration_since(self.last_chunk_at);
        let total_elapsed = now.duration_since(self.stream_started_at);

        if silence_duration > self.config.max_inactivity_duration {
            self.is_aborted = true;
            return Err(StreamingTimeoutError::InactivityTimeout {
                silence_secs: silence_duration.as_secs(),
                chunks_received: self.chunks_received,
            });
        }

        if total_elapsed > self.config.total_stream_timeout {
            self.is_aborted = true;
            return Err(StreamingTimeoutError::TotalDurationTimeout {
                total_secs: total_elapsed.as_secs(),
                bytes_received: self.bytes_received,
            });
        }

        Ok(())
    }

    /// Returns the total duration since streaming began.
    pub fn elapsed(&self) -> Duration {
        Instant::now().duration_since(self.stream_started_at)
    }

    /// Total chunks observed so far.
    pub fn chunks_received(&self) -> u64 {
        self.chunks_received
    }

    /// Total bytes observed so far.
    pub fn bytes_received(&self) -> u64 {
        self.bytes_received
    }
}

/// Errors raised when streaming timeouts or stalls occur.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum StreamingTimeoutError {
    #[error("Streaming inactivity timeout: no chunk received for {silence_secs}s after {chunks_received} chunks")]
    InactivityTimeout {
        silence_secs: u64,
        chunks_received: u64,
    },
    #[error("Total streaming duration limit reached: {total_secs}s elapsed with {bytes_received} bytes received")]
    TotalDurationTimeout {
        total_secs: u64,
        bytes_received: u64,
    },
    #[error("Stream was already aborted due to a previous timeout violation")]
    AlreadyAborted,
}
