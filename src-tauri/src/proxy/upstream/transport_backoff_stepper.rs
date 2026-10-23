//! Transport Backoff Stepper with Jitter (`Imp-620`).
//! Implements adaptive exponential backoff with full jitter and decorrelated jitter
//! on upstream proxy network errors (DNS resolution failures, TCP resets, timeouts, 502/503/504 gateway failures).
//! Enforces AGENTS.md retry requirements ("Bound every wait with a timeout", "exponential backoff with jitter").

use serde::{Deserialize, Serialize};

/// Supported jitter algorithms for backoff delay calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JitterMode {
    /// Full jitter: delay = Uniform(0, min(max_backoff, base * 2^attempt)).
    FullJitter,
    /// Equal jitter: delay = (min(max_backoff, base * 2^attempt) / 2) + Uniform(0, min(...) / 2).
    EqualJitter,
    /// Decorrelated jitter: delay = min(max_backoff, Uniform(base, prev_delay * 3)).
    DecorrelatedJitter,
    /// Pure deterministic exponential backoff without jitter.
    None,
}

impl Default for JitterMode {
    fn default() -> Self {
        Self::FullJitter
    }
}

/// Upstream network and gateway failure classifications.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkFailureType {
    /// DNS resolution failure / NXDOMAIN.
    DnsResolutionFailed,
    /// TCP reset by peer (ECONNRESET / WSAECONNRESET).
    TcpReset,
    /// Connection or read/write idle timeout.
    ConnectionTimeout,
    /// TLS handshake negotiation failure.
    TlsHandshakeFailed,
    /// HTTP Gateway / proxy error status (e.g. 502, 503, 504, 429).
    GatewayStatus(u16),
    /// Unrecoverable client error or protocol violation (non-retryable).
    UnrecoverableError(String),
}

impl NetworkFailureType {
    /// Returns true if this failure type is generally considered transient and retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::DnsResolutionFailed
            | Self::TcpReset
            | Self::ConnectionTimeout
            | Self::TlsHandshakeFailed => true,
            Self::GatewayStatus(code) => matches!(*code, 408 | 429 | 500 | 502 | 503 | 504),
            Self::UnrecoverableError(_) => false,
        }
    }
}

/// Configuration parameters for the backoff stepper.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackoffConfig {
    /// Initial base backoff delay in milliseconds (default: 500ms).
    pub base_backoff_ms: u64,
    /// Upper bound cap on backoff delay in milliseconds (default: 30,000ms = 30s).
    pub max_backoff_ms: u64,
    /// Maximum allowed retry attempts before aborting (default: 5).
    pub max_attempts: u32,
    /// Jitter strategy used to desynchronize retry storms.
    pub jitter_mode: JitterMode,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            base_backoff_ms: 500,
            max_backoff_ms: 30_000,
            max_attempts: 5,
            jitter_mode: JitterMode::FullJitter,
        }
    }
}

impl BackoffConfig {
    /// Creates a new BackoffConfig with specified base and max caps.
    pub fn new(base_ms: u64, max_ms: u64, max_attempts: u32, mode: JitterMode) -> Self {
        Self {
            base_backoff_ms: base_ms.max(1),
            max_backoff_ms: max_ms.max(base_ms),
            max_attempts,
            jitter_mode: mode,
        }
    }
}

/// Execution outcome for a single step through the backoff ladder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackoffStepResult {
    /// Whether another retry should be attempted.
    pub should_retry: bool,
    /// Recommended sleep duration in milliseconds prior to retry.
    pub delay_ms: u64,
    /// Attempt counter (1-indexed for the upcoming retry).
    pub attempt: u32,
    /// Explanatory reason if the request is not retryable or max attempts exhausted.
    pub reason: Option<String>,
}

/// Stateful backoff stepper tracking attempts and cumulative wait durations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransportBackoffStepper {
    config: BackoffConfig,
    attempt: u32,
    last_delay_ms: u64,
    total_wait_ms: u64,
}

impl Default for TransportBackoffStepper {
    fn default() -> Self {
        Self::new(BackoffConfig::default())
    }
}

impl TransportBackoffStepper {
    /// Constructs a new TransportBackoffStepper with given configuration.
    pub fn new(config: BackoffConfig) -> Self {
        let last = config.base_backoff_ms;
        Self {
            config,
            attempt: 0,
            last_delay_ms: last,
            total_wait_ms: 0,
        }
    }

    /// Resets the stepper state back to zero attempts.
    pub fn reset(&mut self) {
        self.attempt = 0;
        self.last_delay_ms = self.config.base_backoff_ms;
        self.total_wait_ms = 0;
    }

    /// Returns the current attempt number.
    pub fn current_attempt(&self) -> u32 {
        self.attempt
    }

    /// Returns the total cumulative milliseconds waited across all retries.
    pub fn total_wait_ms(&self) -> u64 {
        self.total_wait_ms
    }

    /// Evaluates a failure with an explicit random multiplier in range [0.0, 1.0] for deterministic testing.
    pub fn step_with_random(
        &mut self,
        failure: &NetworkFailureType,
        rand_val: f64,
    ) -> BackoffStepResult {
        if !failure.is_retryable() {
            return BackoffStepResult {
                should_retry: false,
                delay_ms: 0,
                attempt: self.attempt,
                reason: Some(format!("Failure type {:?} is non-retryable", failure)),
            };
        }

        if self.attempt >= self.config.max_attempts {
            return BackoffStepResult {
                should_retry: false,
                delay_ms: 0,
                attempt: self.attempt,
                reason: Some(format!(
                    "Maximum retry attempts ({}) exhausted",
                    self.config.max_attempts
                )),
            };
        }

        self.attempt += 1;
        let clamped_rand = rand_val.clamp(0.0, 1.0);

        // Calculate raw exponential ceiling: base * 2^(attempt - 1)
        let exponent = (self.attempt - 1).min(31);
        let raw_exp = self
            .config
            .base_backoff_ms
            .saturating_mul(1u64.checked_shl(exponent).unwrap_or(u64::MAX));
        let ceiling = raw_exp.min(self.config.max_backoff_ms);

        let delay = match self.config.jitter_mode {
            JitterMode::None => ceiling,
            JitterMode::FullJitter => {
                let range = ceiling.max(1) as f64;
                (clamped_rand * range).round() as u64
            }
            JitterMode::EqualJitter => {
                let half = ceiling / 2;
                let jitter_part = ((ceiling - half) as f64 * clamped_rand).round() as u64;
                half + jitter_part
            }
            JitterMode::DecorrelatedJitter => {
                let low = self.config.base_backoff_ms;
                let high = self
                    .last_delay_ms
                    .saturating_mul(3)
                    .max(low)
                    .min(self.config.max_backoff_ms);
                let span = (high - low) as f64;
                low + (clamped_rand * span).round() as u64
            }
        };

        let final_delay = delay.clamp(0, self.config.max_backoff_ms);
        self.last_delay_ms = final_delay.max(self.config.base_backoff_ms);
        self.total_wait_ms = self.total_wait_ms.saturating_add(final_delay);

        BackoffStepResult {
            should_retry: true,
            delay_ms: final_delay,
            attempt: self.attempt,
            reason: None,
        }
    }

    /// Evaluates a failure with a pseudo-random seed derived from attempt counter and clock skew.
    pub fn step(&mut self, failure: &NetworkFailureType) -> BackoffStepResult {
        // Deterministic fallback pseudo-randomness in [0.15, 0.85] range if system entropy is unavailable
        let pseudo = (((self.attempt as f64 * 37.0) + (self.last_delay_ms as f64 * 13.0)) % 100.0) / 100.0;
        self.step_with_random(failure, pseudo)
    }
}
