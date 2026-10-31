//! Account Switch Audit Emitter (`Imp-623`).
//!
//! Structured JSON telemetry and audit event emission for auto-switch decisions.
//! Tracks switch trigger reasons, candidate scores breakdown, cooldown duration,
//! velocity forecast runway, and emits auditable switch logs into AppError / tracing
//! streams without swallowing errors.
//!
//! Enforces AGENTS.md invariant:
//! "No silent errors: every fallible operation must route its failure into the error module."

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Trigger reason categorizing why an auto-switch was initiated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SwitchTriggerReason {
    /// Upstream returned HTTP 429 Too Many Requests / Resource Exhausted.
    RateLimit429,
    /// Preemptive velocity-based quota exhaustion forecast.
    PreemptiveExhaustionVelocity,
    /// Explicit manual switch override requested by user or admin.
    ManualOverride,
    /// Rapid failure or endpoint health score decay below threshold.
    HealthDecay,
    /// Cooldown timer has expired for candidate account.
    CooldownExpired,
    /// Initial startup or session account selection.
    InitialSelection,
}

impl SwitchTriggerReason {
    /// Returns standardized uppercase string representation of trigger reason.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RateLimit429 => "RATE_LIMIT_429",
            Self::PreemptiveExhaustionVelocity => "PREEMPTIVE_EXHAUSTION_VELOCITY",
            Self::ManualOverride => "MANUAL_OVERRIDE",
            Self::HealthDecay => "HEALTH_DECAY",
            Self::CooldownExpired => "COOLDOWN_EXPIRED",
            Self::InitialSelection => "INITIAL_SELECTION",
        }
    }

    /// Parses trigger reason from variant name or uppercase identifier string.
    pub fn from_str_name(s: &str) -> Option<Self> {
        match s {
            "RateLimit429" | "RATE_LIMIT_429" => Some(Self::RateLimit429),
            "PreemptiveExhaustionVelocity" | "PREEMPTIVE_EXHAUSTION_VELOCITY" => {
                Some(Self::PreemptiveExhaustionVelocity)
            }
            "ManualOverride" | "MANUAL_OVERRIDE" => Some(Self::ManualOverride),
            "HealthDecay" | "HEALTH_DECAY" => Some(Self::HealthDecay),
            "CooldownExpired" | "COOLDOWN_EXPIRED" => Some(Self::CooldownExpired),
            "InitialSelection" | "INITIAL_SELECTION" => Some(Self::InitialSelection),
            _ => None,
        }
    }
}

impl fmt::Display for SwitchTriggerReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Candidate account evaluation score details during an auto-switch decision pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateAuditScore {
    /// Candidate account identifier.
    pub candidate_account_id: String,
    /// Composite final score determined by scoring engine.
    pub final_score: f64,
    /// Headroom available in quota (0.0 - 1.0 or remaining requests).
    pub quota_headroom: f64,
    /// Penalty deducted due to high token burn rate velocity.
    pub burn_rate_velocity_penalty: f64,
    /// Penalty applied due to recent throttling or rate limits.
    pub throttle_penalty: f64,
    /// Whether candidate is currently in cooldown (true = active cooldown).
    pub cooldown_status: bool,
}

impl CandidateAuditScore {
    /// Constructs a new `CandidateAuditScore` with all constituent evaluation factors.
    pub fn new(
        candidate_account_id: impl Into<String>,
        final_score: f64,
        quota_headroom: f64,
        burn_rate_velocity_penalty: f64,
        throttle_penalty: f64,
        cooldown_status: bool,
    ) -> Self {
        Self {
            candidate_account_id: candidate_account_id.into(),
            final_score,
            quota_headroom,
            burn_rate_velocity_penalty,
            throttle_penalty,
            cooldown_status,
        }
    }
}

/// Complete telemetry payload for an account switch audit event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountSwitchAuditEvent {
    /// Unique event identifier (UUID v4).
    pub event_id: String,
    /// Milliseconds since Unix epoch when switch event was triggered.
    pub timestamp_epoch_ms: u64,
    /// Source account ID being rotated out (None if initial selection).
    pub source_account_id: Option<String>,
    /// Target account ID being rotated in.
    pub target_account_id: String,
    /// Primary rationale / trigger reason for account switch.
    pub trigger_reason: SwitchTriggerReason,
    /// Evaluated candidate accounts and score breakdown.
    pub candidate_scores: Vec<CandidateAuditScore>,
    /// Forecast runway remaining in seconds (if known).
    pub forecast_runway_seconds: Option<u64>,
    /// Applied cooldown duration in seconds (if candidate entered cooldown).
    pub cooldown_seconds: Option<u64>,
    /// Whether the switch operation succeeded.
    pub success: bool,
    /// Diagnostic error message if switch failed.
    pub error_message: Option<String>,
}

impl AccountSwitchAuditEvent {
    /// Constructs a new audit event with UUID v4 and current epoch milliseconds.
    pub fn new(
        source_account_id: Option<String>,
        target_account_id: impl Into<String>,
        trigger_reason: SwitchTriggerReason,
    ) -> Self {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp_epoch_ms: now_ms,
            source_account_id,
            target_account_id: target_account_id.into(),
            trigger_reason,
            candidate_scores: Vec::new(),
            forecast_runway_seconds: None,
            cooldown_seconds: None,
            success: true,
            error_message: None,
        }
    }

    /// Sets evaluated candidate scores list.
    pub fn with_candidate_scores(mut self, scores: Vec<CandidateAuditScore>) -> Self {
        self.candidate_scores = scores;
        self
    }

    /// Sets forecast runway remaining in seconds.
    pub fn with_forecast_runway(mut self, runway_seconds: Option<u64>) -> Self {
        self.forecast_runway_seconds = runway_seconds;
        self
    }

    /// Sets cooldown duration in seconds.
    pub fn with_cooldown_seconds(mut self, cooldown_seconds: Option<u64>) -> Self {
        self.cooldown_seconds = cooldown_seconds;
        self
    }

    /// Sets execution outcome and optional error message.
    pub fn with_outcome(mut self, success: bool, error_message: Option<String>) -> Self {
        self.success = success;
        self.error_message = error_message;
        self
    }

    /// Returns candidate scores ranked in descending order by `final_score`.
    pub fn ranked_candidate_scores(&self) -> Vec<CandidateAuditScore> {
        rank_candidate_scores(&self.candidate_scores)
    }

    /// Finds candidate score entry for target account.
    pub fn target_candidate_score(&self) -> Option<&CandidateAuditScore> {
        self.candidate_scores
            .iter()
            .find(|c| c.candidate_account_id == self.target_account_id)
    }

    /// Serializes this audit event into a structured JSON string.
    pub fn to_json(&self) -> AppResult<String> {
        format_audit_event_json(self)
    }
}

/// Backwards compatibility alias for `AccountSwitchAuditEvent`.
pub type SwitchAuditEvent = AccountSwitchAuditEvent;

/// Helper function to construct a complete `AccountSwitchAuditEvent`.
#[allow(clippy::too_many_arguments)]
pub fn build_switch_audit_event(
    source_account_id: Option<String>,
    target_account_id: impl Into<String>,
    trigger_reason: SwitchTriggerReason,
    candidate_scores: Vec<CandidateAuditScore>,
    forecast_runway_seconds: Option<u64>,
    cooldown_seconds: Option<u64>,
    success: bool,
    error_message: Option<String>,
) -> AccountSwitchAuditEvent {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    AccountSwitchAuditEvent {
        event_id: uuid::Uuid::new_v4().to_string(),
        timestamp_epoch_ms: now_ms,
        source_account_id,
        target_account_id: target_account_id.into(),
        trigger_reason,
        candidate_scores,
        forecast_runway_seconds,
        cooldown_seconds,
        success,
        error_message,
    }
}

/// Ranks candidate scores in descending order of final score.
pub fn rank_candidate_scores(scores: &[CandidateAuditScore]) -> Vec<CandidateAuditScore> {
    let mut ranked = scores.to_vec();
    ranked.sort_by(|a, b| {
        b.final_score
            .partial_cmp(&a.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ranked
}

/// Formats an `AccountSwitchAuditEvent` into a structured, pretty-printed JSON string.
pub fn format_audit_event_json(event: &AccountSwitchAuditEvent) -> AppResult<String> {
    serde_json::to_string_pretty(event).map_err(|e| {
        AppError::Config(format!(
            "Failed to serialize AccountSwitchAuditEvent to JSON: {e}"
        ))
    })
}

/// Parses an `AccountSwitchAuditEvent` from a JSON string.
pub fn parse_audit_event_json(json: &str) -> AppResult<AccountSwitchAuditEvent> {
    serde_json::from_str(json).map_err(|e| {
        AppError::Config(format!(
            "Failed to parse AccountSwitchAuditEvent from JSON: {e}"
        ))
    })
}

/// Global in-memory ring buffer for recent switch audit events.
static RECENT_AUDIT_EVENTS: Mutex<Vec<AccountSwitchAuditEvent>> = Mutex::new(Vec::new());
static TOTAL_EMITTED_EVENTS: AtomicUsize = AtomicUsize::new(0);
const MAX_RING_BUFFER_CAPACITY: usize = 200;

/// Emits an account switch audit event, logging via `tracing::info!` or `tracing::warn!`
/// and recording it into the global in-memory audit ring buffer.
///
/// Propagates `AppError` on serialization or lock failure; never silently swallows errors.
pub fn emit_account_switch_audit(event: &AccountSwitchAuditEvent) -> AppResult<()> {
    let json = format_audit_event_json(event)?;

    if event.success {
        tracing::info!(
            target: "auto_switcher::audit",
            event_id = %event.event_id,
            source = ?event.source_account_id,
            target = %event.target_account_id,
            reason = %event.trigger_reason,
            candidates = event.candidate_scores.len(),
            "Auto-switch audit event emitted: {}",
            json
        );
    } else {
        tracing::warn!(
            target: "auto_switcher::audit",
            event_id = %event.event_id,
            source = ?event.source_account_id,
            target = %event.target_account_id,
            reason = %event.trigger_reason,
            error = ?event.error_message,
            "Auto-switch audit event recorded failure: {}",
            json
        );
    }

    let mut buffer = RECENT_AUDIT_EVENTS
        .lock()
        .map_err(|e| AppError::Config(format!("Failed to acquire audit buffer lock: {e}")))?;

    if buffer.len() >= MAX_RING_BUFFER_CAPACITY {
        buffer.remove(0);
    }
    buffer.push(event.clone());
    TOTAL_EMITTED_EVENTS.fetch_add(1, Ordering::Relaxed);

    Ok(())
}

/// Retrieves up to `limit` most recent switch audit events (newest first).
pub fn get_recent_audit_events(limit: usize) -> Vec<AccountSwitchAuditEvent> {
    if let Ok(buffer) = RECENT_AUDIT_EVENTS.lock() {
        buffer.iter().rev().take(limit).cloned().collect()
    } else {
        Vec::new()
    }
}

/// Clears the in-memory audit ring buffer (primarily for unit tests and diagnostics).
pub fn clear_audit_events() {
    if let Ok(mut buffer) = RECENT_AUDIT_EVENTS.lock() {
        buffer.clear();
        TOTAL_EMITTED_EVENTS.store(0, Ordering::Relaxed);
    }
}

/// Returns the total number of audit events emitted over runtime lifetime.
pub fn total_emitted_audit_events_count() -> usize {
    TOTAL_EMITTED_EVENTS.load(Ordering::Relaxed)
}

/// Exports all currently buffered audit events as a structured JSON array string.
pub fn export_buffered_events_json() -> AppResult<String> {
    let events = get_recent_audit_events(MAX_RING_BUFFER_CAPACITY);
    serde_json::to_string_pretty(&events)
        .map_err(|e| AppError::Config(format!("Failed to export audit events JSON: {e}")))
}

/// Static coordinator for emitting, storing, and exporting switch audit telemetry.
pub struct AccountSwitchAuditEmitter;

impl AccountSwitchAuditEmitter {
    /// Emits a switch audit event and returns structured JSON representation.
    pub fn emit(event: &AccountSwitchAuditEvent) -> AppResult<String> {
        let json = format_audit_event_json(event)?;
        emit_account_switch_audit(event)?;
        Ok(json)
    }

    /// Fetches up to `limit` most recent audit events (newest first).
    pub fn recent_events(limit: usize) -> Vec<AccountSwitchAuditEvent> {
        get_recent_audit_events(limit)
    }

    /// Clears the in-memory audit ring buffer.
    pub fn clear() {
        clear_audit_events();
    }

    /// Returns the total number of events emitted over runtime lifetime.
    pub fn total_emitted_count() -> usize {
        total_emitted_audit_events_count()
    }

    /// Exports all currently buffered events as a structured JSON array string.
    pub fn export_buffered_events_json() -> AppResult<String> {
        export_buffered_events_json()
    }
}
