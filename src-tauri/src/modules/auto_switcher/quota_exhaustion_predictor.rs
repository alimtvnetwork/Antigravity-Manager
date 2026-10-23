//! Quota Exhaustion Velocity Predictor (`Imp-618`)
//!
//! Calculates token burn rate velocity and forecasts the time horizon until
//! quota exhaustion before HTTP 429 lockouts occur. Evaluates consumption
//! velocity across usage observations and triggers preemptive switching when
//! remaining runway falls below a configured threshold (e.g. <= 300s).

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Default switch threshold in seconds (5 minutes = 300 seconds).
pub const DEFAULT_SWITCH_THRESHOLD_SECONDS: u64 = 300;

/// Observation of quota/token consumption at a specific point in time.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct UsageObservation {
    /// Timestamp of the observation in Unix epoch seconds.
    pub timestamp_epoch_secs: u64,
    /// Number of tokens consumed in this observation.
    pub tokens_consumed: u64,
}

impl UsageObservation {
    /// Creates a new `UsageObservation` with timestamp and tokens consumed.
    pub fn new(timestamp_epoch_secs: u64, tokens_consumed: u64) -> Self {
        Self {
            timestamp_epoch_secs,
            tokens_consumed,
        }
    }

    /// Accessor for tokens consumed.
    pub fn tokens(&self) -> u64 {
        self.tokens_consumed
    }

    /// Accessor for observation timestamp.
    pub fn timestamp(&self) -> u64 {
        self.timestamp_epoch_secs
    }
}

/// Token burn rate velocity calculated across a series of observations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BurnRateVelocity {
    /// Calculated token burn rate per second.
    pub tokens_per_second: f64,
    /// Calculated token burn rate per minute.
    pub tokens_per_minute: f64,
    /// Number of observations evaluated.
    pub observation_count: usize,
    /// Time span of observations in seconds.
    pub window_seconds: u64,
    /// Total tokens consumed across observations.
    pub total_tokens: u64,
}

impl BurnRateVelocity {
    /// Returns a zero velocity instance.
    pub fn zero() -> Self {
        Self {
            tokens_per_second: 0.0,
            tokens_per_minute: 0.0,
            observation_count: 0,
            window_seconds: 0,
            total_tokens: 0,
        }
    }

    /// Returns true if burn rate velocity is zero or idle.
    pub fn is_zero(&self) -> bool {
        self.tokens_per_second <= 0.0
    }

    /// Alias for tokens_per_second.
    pub fn tokens_per_sec(&self) -> f64 {
        self.tokens_per_second
    }

    /// Alias for tokens_per_minute.
    pub fn tokens_per_min(&self) -> f64 {
        self.tokens_per_minute
    }

    /// Alias for observation_count.
    pub fn sample_count(&self) -> usize {
        self.observation_count
    }

    /// Alias for total_tokens.
    pub fn total_tokens_observed(&self) -> u64 {
        self.total_tokens
    }
}

/// Severity classification of impending quota exhaustion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ExhaustionSeverity {
    /// Ample runway remaining; burn rate is sustainable and safely above thresholds.
    Nominal,
    /// Consumption velocity is elevated; projected runway is within warning window (e.g. <= 2x threshold).
    Elevated,
    /// Quota exhaustion is imminent (runway <= threshold or zero tokens remaining); preemptive switch triggered.
    Critical,
}

impl ExhaustionSeverity {
    /// Returns true if severity is critical.
    pub fn is_critical(&self) -> bool {
        matches!(self, Self::Critical)
    }

    /// Returns true if severity is elevated.
    pub fn is_elevated(&self) -> bool {
        matches!(self, Self::Elevated)
    }

    /// Returns true if severity is nominal.
    pub fn is_nominal(&self) -> bool {
        matches!(self, Self::Nominal)
    }
}

/// Quota exhaustion forecast detailing projected runway and switch recommendation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExhaustionForecast {
    /// Calculated burn rate velocity.
    pub velocity: BurnRateVelocity,
    /// Remaining quota tokens on the target account.
    pub remaining_tokens: u64,
    /// Projected seconds until quota is exhausted (`None` if burn rate is zero).
    pub runway_seconds: Option<f64>,
    /// Risk severity level.
    pub severity: ExhaustionSeverity,
    /// Configured threshold in seconds that triggers a switch recommendation.
    pub switch_threshold_seconds: u64,
    /// Whether a preemptive auto-switch should be triggered.
    pub should_switch_preemptively: bool,
    /// Human-readable explanation of the forecast and recommendation.
    pub reason: String,
}

impl ExhaustionForecast {
    /// Returns true if exhaustion severity is critical.
    pub fn is_critical(&self) -> bool {
        self.severity == ExhaustionSeverity::Critical
    }

    /// Returns true if exhaustion severity is elevated.
    pub fn is_elevated(&self) -> bool {
        self.severity == ExhaustionSeverity::Elevated
    }

    /// Returns true if exhaustion severity is nominal.
    pub fn is_nominal(&self) -> bool {
        self.severity == ExhaustionSeverity::Nominal
    }

    /// Returns true if a preemptive switch is recommended.
    pub fn should_switch(&self) -> bool {
        self.should_switch_preemptively
    }

    /// Alias for `should_switch`.
    pub fn is_switch_recommended(&self) -> bool {
        self.should_switch_preemptively
    }

    /// Returns projected runway in minutes, if available.
    pub fn runway_minutes(&self) -> Option<f64> {
        self.runway_seconds
            .map(|s| (s / 60.0 * 10.0).round() / 10.0)
    }
}

/// Calculates token burn rate velocity across an observation slice.
///
/// Automatically sorts observations chronologically. Handles zero, single,
/// and multiple observation cases gracefully without panic or division by zero.
pub fn calculate_burn_rate_velocity(observations: &[UsageObservation]) -> BurnRateVelocity {
    if observations.is_empty() {
        return BurnRateVelocity::zero();
    }

    if observations.len() == 1 {
        let single = &observations[0];
        return BurnRateVelocity {
            tokens_per_second: 0.0,
            tokens_per_minute: 0.0,
            observation_count: 1,
            window_seconds: 0,
            total_tokens: single.tokens_consumed,
        };
    }

    let mut sorted = observations.to_vec();
    sorted.sort_by_key(|o| o.timestamp_epoch_secs);

    let oldest = &sorted[0];
    let newest = &sorted[sorted.len() - 1];

    let window_secs = newest
        .timestamp_epoch_secs
        .saturating_sub(oldest.timestamp_epoch_secs);
    let total_tokens: u64 = sorted.iter().map(|o| o.tokens_consumed).sum();

    // If multiple observations occur at the exact same second, bound window to minimum 1s to prevent division by zero
    let effective_secs = if window_secs == 0 {
        1.0
    } else {
        window_secs as f64
    };

    let tokens_per_second = (total_tokens as f64 / effective_secs).max(0.0);
    let tokens_per_minute = tokens_per_second * 60.0;

    BurnRateVelocity {
        tokens_per_second,
        tokens_per_minute,
        observation_count: sorted.len(),
        window_seconds: window_secs,
        total_tokens,
    }
}

/// Predicts quota exhaustion horizon and determines whether preemptive switching is warranted.
///
/// # Arguments
/// * `observations` - Historic usage observations
/// * `remaining_tokens` - Current remaining tokens for the candidate account
/// * `switch_threshold_seconds` - Runway threshold below which a preemptive switch is triggered (e.g. 300s)
pub fn predict_quota_exhaustion(
    observations: &[UsageObservation],
    remaining_tokens: u64,
    switch_threshold_seconds: u64,
) -> ExhaustionForecast {
    let velocity = calculate_burn_rate_velocity(observations);

    if remaining_tokens == 0 {
        warn!("Candidate account has zero remaining quota tokens; triggering immediate critical switch");
        return ExhaustionForecast {
            velocity,
            remaining_tokens: 0,
            runway_seconds: Some(0.0),
            severity: ExhaustionSeverity::Critical,
            switch_threshold_seconds,
            should_switch_preemptively: true,
            reason: "Remaining quota is zero; immediate exhaustion lockout imminent".to_string(),
        };
    }

    if velocity.is_zero() {
        debug!(
            remaining_tokens,
            "Token burn rate is zero; quota runway is effectively infinite"
        );
        return ExhaustionForecast {
            velocity,
            remaining_tokens,
            runway_seconds: None,
            severity: ExhaustionSeverity::Nominal,
            switch_threshold_seconds,
            should_switch_preemptively: false,
            reason: "Zero or negligible token consumption velocity; runway is effectively infinite"
                .to_string(),
        };
    }

    let runway_seconds = remaining_tokens as f64 / velocity.tokens_per_second;
    let rounded_runway = (runway_seconds * 10.0).round() / 10.0;

    let threshold_f64 = switch_threshold_seconds as f64;
    let elevated_threshold = threshold_f64 * 2.0;

    let (severity, should_switch, reason) = if runway_seconds <= threshold_f64 {
        warn!(
            remaining_tokens,
            runway_seconds = rounded_runway,
            threshold = switch_threshold_seconds,
            "Preemptive quota exhaustion switch triggered: runway <= threshold"
        );
        (
            ExhaustionSeverity::Critical,
            true,
            format!(
                "Projected runway {:.1}s is at or below switch threshold {}s; preemptive switch required",
                rounded_runway, switch_threshold_seconds
            ),
        )
    } else if runway_seconds <= elevated_threshold {
        debug!(
            remaining_tokens,
            runway_seconds = rounded_runway,
            "Quota consumption velocity elevated; within warning window"
        );
        (
            ExhaustionSeverity::Elevated,
            false,
            format!(
                "Projected runway {:.1}s is approaching threshold {}s (within {:.1}s window); elevated risk",
                rounded_runway, switch_threshold_seconds, elevated_threshold
            ),
        )
    } else {
        debug!(
            remaining_tokens,
            runway_seconds = rounded_runway,
            "Quota runway healthy; comfortably exceeds switch threshold"
        );
        (
            ExhaustionSeverity::Nominal,
            false,
            format!(
                "Projected runway {:.1}s safely exceeds switch threshold {}s; quota status is nominal",
                rounded_runway, switch_threshold_seconds
            ),
        )
    };

    ExhaustionForecast {
        velocity,
        remaining_tokens,
        runway_seconds: Some(rounded_runway),
        severity,
        switch_threshold_seconds,
        should_switch_preemptively: should_switch,
        reason,
    }
}

/// Helper calling `predict_quota_exhaustion` with the default threshold (300 seconds).
pub fn predict_quota_exhaustion_default(
    observations: &[UsageObservation],
    remaining_tokens: u64,
) -> ExhaustionForecast {
    predict_quota_exhaustion(
        observations,
        remaining_tokens,
        DEFAULT_SWITCH_THRESHOLD_SECONDS,
    )
}

/// OOP-style wrapper around the quota exhaustion functions.
pub struct QuotaExhaustionPredictor;

impl QuotaExhaustionPredictor {
    pub fn calculate_burn_rate_velocity(observations: &[UsageObservation]) -> BurnRateVelocity {
        calculate_burn_rate_velocity(observations)
    }

    pub fn predict_quota_exhaustion(
        observations: &[UsageObservation],
        remaining_tokens: u64,
        switch_threshold_seconds: u64,
    ) -> ExhaustionForecast {
        predict_quota_exhaustion(observations, remaining_tokens, switch_threshold_seconds)
    }

    pub fn predict_quota_exhaustion_default(
        observations: &[UsageObservation],
        remaining_tokens: u64,
    ) -> ExhaustionForecast {
        predict_quota_exhaustion_default(observations, remaining_tokens)
    }
}
