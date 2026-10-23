//! Caller IDE Vitality Heartbeat & Eviction Watchdog (`Imp-661`).
//!
//! Enforces caller IDE self-preservation invariant per AGENTS.md:
//! "Caller IDE Self-Preservation & Transient Test Sandboxing: Automated tests,
//! CLI commands, and auto-switcher daemons MUST detect the caller's active IDE
//! process (`detect_caller_instance_id()`) to prevent terminating the developer's
//! working IDE session. Commands targeting the caller instance (`agm instances stop`,
//! `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided."
//!
//! Tracks caller IDE process vitality, verifies monotonic heartbeats, and guarantees
//! that auto-switcher daemons and background eviction routines never evict, terminate,
//! or starve active developer working sessions.

use serde::{Deserialize, Serialize};

/// Default heartbeat time-to-live in milliseconds (30 seconds).
pub const DEFAULT_HEARTBEAT_TTL_MS: u64 = 30_000;

/// Minimum allowable heartbeat time-to-live in milliseconds (1 second).
pub const MIN_HEARTBEAT_TTL_MS: u64 = 1_000;

/// Maximum allowable heartbeat time-to-live in milliseconds (5 minutes).
pub const MAX_HEARTBEAT_TTL_MS: u64 = 300_000;

/// Safe retrieval of current epoch timestamp in milliseconds with clock-skew error recording.
pub fn get_system_time_epoch_ms() -> u64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(err) => {
            crate::error::record_ignored(
                Err::<(), _>(&err),
                "caller_ide_heartbeat_watchdog::get_system_time_epoch_ms clock skew",
            );
            // Justification: System clock jump backwards safely falls back to epoch 0 to preserve continuity
            0
        }
    }
}

/// Eviction decision type representing the outcome of an eviction immunity evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CallerEvictionDecisionType {
    ProtectedActiveCaller,
    ProtectedFreshHeartbeat,
    EvictionPermittedForcedOverride,
    EvictionPermittedDifferentTarget,
    EvictionPermittedCallerDead,
    EvictionPermittedHeartbeatStale,
    EvictionPermittedNoCallerTracked,
}

impl CallerEvictionDecisionType {
    /// Returns true if the decision grants caller eviction immunity.
    pub fn is_protected(&self) -> bool {
        matches!(
            self,
            Self::ProtectedActiveCaller | Self::ProtectedFreshHeartbeat
        )
    }

    /// Formats the decision type into a machine-readable string key.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProtectedActiveCaller => "protected_active_caller",
            Self::ProtectedFreshHeartbeat => "protected_fresh_heartbeat",
            Self::EvictionPermittedForcedOverride => "eviction_permitted_forced_override",
            Self::EvictionPermittedDifferentTarget => "eviction_permitted_different_target",
            Self::EvictionPermittedCallerDead => "eviction_permitted_caller_dead",
            Self::EvictionPermittedHeartbeatStale => "eviction_permitted_heartbeat_stale",
            Self::EvictionPermittedNoCallerTracked => "eviction_permitted_no_caller_tracked",
        }
    }
}

/// Watchdog vitality health state of the tracked caller IDE session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WatchdogVitalityStateType {
    HealthyActive,
    StaleHeartbeat,
    DeadProcess,
    Uninitialized,
}

impl WatchdogVitalityStateType {
    /// Returns true if the session is currently considered active and healthy.
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::HealthyActive)
    }

    /// Formats the state into a canonical string key.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HealthyActive => "healthy_active",
            Self::StaleHeartbeat => "stale_heartbeat",
            Self::DeadProcess => "dead_process",
            Self::Uninitialized => "uninitialized",
        }
    }
}

/// Structured explanation type describing the underlying protection reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvictionProtectionReasonType {
    ActiveCallerWithLivePid,
    ForcedOverrideBypass,
    TargetNotCaller,
    CallerPidNotAlive,
    HeartbeatExpired,
    NoCallerConfigured,
}

impl EvictionProtectionReasonType {
    /// Formats the reason into human-readable descriptive text.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ActiveCallerWithLivePid => {
                "Active caller IDE session verified with live PID and fresh heartbeat"
            }
            Self::ForcedOverrideBypass => "Forced eviction override requested via flag",
            Self::TargetNotCaller => "Target PID does not match caller IDE session PID",
            Self::CallerPidNotAlive => "Caller IDE process is no longer active in OS process table",
            Self::HeartbeatExpired => "Caller IDE heartbeat has expired and is stale",
            Self::NoCallerConfigured => "No caller IDE session is currently registered in watchdog",
        }
    }
}

/// Structured caller IDE heartbeat and vitality status report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallerIdeHeartbeatReport {
    pub caller_pid: Option<u32>,
    pub is_alive: bool,
    pub is_eviction_immune: bool,
    pub last_heartbeat_age_ms: u64,
    pub protection_reason: String,
    pub vitality_state: WatchdogVitalityStateType,
    pub decision_type: CallerEvictionDecisionType,
}

impl CallerIdeHeartbeatReport {
    /// Constructs a new CallerIdeHeartbeatReport with all audit fields.
    pub fn new(
        caller_pid: Option<u32>,
        is_alive: bool,
        is_eviction_immune: bool,
        last_heartbeat_age_ms: u64,
        protection_reason: String,
        vitality_state: WatchdogVitalityStateType,
        decision_type: CallerEvictionDecisionType,
    ) -> Self {
        Self {
            caller_pid,
            is_alive,
            is_eviction_immune,
            last_heartbeat_age_ms,
            protection_reason,
            vitality_state,
            decision_type,
        }
    }
}

/// Watchdog controller enforcing caller IDE self-preservation and eviction immunity.
#[derive(Debug, Clone)]
pub struct CallerIdeHeartbeatWatchdog {
    caller_pid: Option<u32>,
    last_heartbeat_timestamp_ms: u64,
    heartbeat_ttl_ms: u64,
    is_protection_enabled: bool,
}

impl Default for CallerIdeHeartbeatWatchdog {
    fn default() -> Self {
        Self::new()
    }
}

impl CallerIdeHeartbeatWatchdog {
    /// Instantiates an empty watchdog without a registered caller PID.
    pub fn new() -> Self {
        Self {
            caller_pid: None,
            last_heartbeat_timestamp_ms: 0,
            heartbeat_ttl_ms: DEFAULT_HEARTBEAT_TTL_MS,
            is_protection_enabled: true,
        }
    }

    /// Instantiates a watchdog initialized with the specified caller PID.
    pub fn with_caller(caller_pid: u32) -> Self {
        let now_ms = get_system_time_epoch_ms();
        Self {
            caller_pid: Some(caller_pid),
            last_heartbeat_timestamp_ms: now_ms,
            heartbeat_ttl_ms: DEFAULT_HEARTBEAT_TTL_MS,
            is_protection_enabled: true,
        }
    }

    /// Instantiates a watchdog initialized with a caller PID and custom TTL.
    pub fn with_caller_and_ttl(caller_pid: u32, ttl_ms: u64) -> Self {
        let bounded_ttl = ttl_ms.clamp(MIN_HEARTBEAT_TTL_MS, MAX_HEARTBEAT_TTL_MS);
        let now_ms = get_system_time_epoch_ms();
        Self {
            caller_pid: Some(caller_pid),
            last_heartbeat_timestamp_ms: now_ms,
            heartbeat_ttl_ms: bounded_ttl,
            is_protection_enabled: true,
        }
    }

    /// Registers or updates the caller PID and records a fresh heartbeat tick.
    pub fn register_caller(&mut self, caller_pid: u32) {
        self.caller_pid = Some(caller_pid);
        self.last_heartbeat_timestamp_ms = get_system_time_epoch_ms();
    }

    /// Unregisters the caller PID, resetting vitality tracking to uninitialized.
    pub fn unregister_caller(&mut self) {
        self.caller_pid = None;
        self.last_heartbeat_timestamp_ms = 0;
    }

    /// Records an active heartbeat tick at the current epoch timestamp.
    pub fn record_heartbeat(&mut self) {
        self.last_heartbeat_timestamp_ms = get_system_time_epoch_ms();
    }

    /// Records an active heartbeat tick at an explicit timestamp (for testing).
    pub fn record_heartbeat_at(&mut self, timestamp_ms: u64) {
        self.last_heartbeat_timestamp_ms = timestamp_ms;
    }

    /// Returns the currently registered caller PID, if any.
    pub fn caller_pid(&self) -> Option<u32> {
        self.caller_pid
    }

    /// Returns the configured heartbeat time-to-live in milliseconds.
    pub fn heartbeat_ttl_ms(&self) -> u64 {
        self.heartbeat_ttl_ms
    }

    /// Returns true if eviction protection enforcement is active.
    pub fn is_protection_enabled(&self) -> bool {
        self.is_protection_enabled
    }

    /// Enables or disables eviction protection enforcement.
    pub fn set_protection_enabled(&mut self, is_enabled: bool) {
        self.is_protection_enabled = is_enabled;
    }

    /// Calculates the age of the last recorded heartbeat in milliseconds.
    pub fn last_heartbeat_age_ms(&self, now_ms: u64) -> u64 {
        if self.last_heartbeat_timestamp_ms == 0 {
            return u64::MAX;
        }
        now_ms.saturating_sub(self.last_heartbeat_timestamp_ms)
    }

    /// Checks if the last heartbeat was recorded within the configured TTL.
    pub fn is_heartbeat_fresh(&self, now_ms: u64) -> bool {
        if self.last_heartbeat_timestamp_ms == 0 {
            return false;
        }
        self.last_heartbeat_age_ms(now_ms) <= self.heartbeat_ttl_ms
    }

    /// Queries OS process vitality for the registered caller PID.
    pub fn is_caller_pid_alive(&self) -> bool {
        match self.caller_pid {
            Some(pid) => {
                crate::modules::instance::process_vitality_probe::ProcessVitalityProbe::is_pid_alive(
                    pid,
                )
            }
            None => false,
        }
    }

    /// Evaluates if target instance PID must be protected from eviction or termination.
    pub fn should_protect_from_eviction(&self, target_pid: u32, is_forced: bool) -> bool {
        let now_ms = get_system_time_epoch_ms();
        self.should_protect_from_eviction_with_probe(
            target_pid,
            is_forced,
            now_ms,
            crate::modules::instance::process_vitality_probe::ProcessVitalityProbe::is_pid_alive,
        )
    }

    /// Evaluates eviction protection with custom timestamp and vitality probe.
    pub fn should_protect_from_eviction_with_probe<F>(
        &self,
        target_pid: u32,
        is_forced: bool,
        now_ms: u64,
        vitality_probe: F,
    ) -> bool
    where
        F: Fn(u32) -> bool,
    {
        let decision = self.evaluate_eviction_decision_with_probe(
            target_pid,
            is_forced,
            now_ms,
            vitality_probe,
        );
        decision.is_protected()
    }

    /// Evaluates eviction decision using system time and real OS vitality probe.
    pub fn evaluate_eviction_decision(
        &self,
        target_pid: u32,
        is_forced: bool,
    ) -> CallerEvictionDecisionType {
        let now_ms = get_system_time_epoch_ms();
        self.evaluate_eviction_decision_with_probe(
            target_pid,
            is_forced,
            now_ms,
            crate::modules::instance::process_vitality_probe::ProcessVitalityProbe::is_pid_alive,
        )
    }

    /// Evaluates eviction decision with custom probe and timestamp.
    pub fn evaluate_eviction_decision_with_probe<F>(
        &self,
        target_pid: u32,
        is_forced: bool,
        now_ms: u64,
        vitality_probe: F,
    ) -> CallerEvictionDecisionType
    where
        F: Fn(u32) -> bool,
    {
        if let Some(early_verdict) = self.check_eviction_prerequisites(is_forced) {
            return early_verdict;
        }
        let caller_pid = self.caller_pid.unwrap_or(0);
        if let Some(vitality_verdict) =
            self.check_target_and_vitality(target_pid, caller_pid, &vitality_probe)
        {
            return vitality_verdict;
        }
        self.resolve_heartbeat_verdict(now_ms)
    }

    /// Checks early eviction prerequisites such as forced flags and registered PID.
    fn check_eviction_prerequisites(&self, is_forced: bool) -> Option<CallerEvictionDecisionType> {
        if is_forced || !self.is_protection_enabled {
            return Some(CallerEvictionDecisionType::EvictionPermittedForcedOverride);
        }
        if self.caller_pid.is_none() {
            return Some(CallerEvictionDecisionType::EvictionPermittedNoCallerTracked);
        }
        None
    }

    /// Verifies target PID alignment and caller OS process table presence.
    fn check_target_and_vitality<F>(
        &self,
        target_pid: u32,
        caller_pid: u32,
        vitality_probe: &F,
    ) -> Option<CallerEvictionDecisionType>
    where
        F: Fn(u32) -> bool,
    {
        if target_pid != caller_pid {
            return Some(CallerEvictionDecisionType::EvictionPermittedDifferentTarget);
        }
        if !vitality_probe(caller_pid) {
            return Some(CallerEvictionDecisionType::EvictionPermittedCallerDead);
        }
        None
    }

    /// Resolves verdict based on whether heartbeat timestamp is within TTL window.
    fn resolve_heartbeat_verdict(&self, now_ms: u64) -> CallerEvictionDecisionType {
        if self.is_heartbeat_fresh(now_ms) {
            CallerEvictionDecisionType::ProtectedActiveCaller
        } else {
            CallerEvictionDecisionType::EvictionPermittedHeartbeatStale
        }
    }

    /// Generates structured heartbeat report using system clock and real OS probe.
    pub fn generate_report(&self) -> CallerIdeHeartbeatReport {
        let now_ms = get_system_time_epoch_ms();
        self.generate_report_with_probe(
            now_ms,
            crate::modules::instance::process_vitality_probe::ProcessVitalityProbe::is_pid_alive,
        )
    }

    /// Generates structured heartbeat report at a specific timestamp.
    pub fn generate_report_at(&self, now_ms: u64) -> CallerIdeHeartbeatReport {
        self.generate_report_with_probe(
            now_ms,
            crate::modules::instance::process_vitality_probe::ProcessVitalityProbe::is_pid_alive,
        )
    }

    /// Generates structured heartbeat report with injected vitality probe.
    pub fn generate_report_with_probe<F>(
        &self,
        now_ms: u64,
        vitality_probe: F,
    ) -> CallerIdeHeartbeatReport
    where
        F: Fn(u32) -> bool,
    {
        let (is_alive, vitality_state) = self.determine_vitality_state(now_ms, &vitality_probe);
        let target_pid = self.caller_pid.unwrap_or(0);
        let decision =
            self.evaluate_eviction_decision_with_probe(target_pid, false, now_ms, &vitality_probe);
        let is_eviction_immune = decision.is_protected();
        let age_ms = self.last_heartbeat_age_ms(now_ms);
        let reason = Self::resolve_protection_reason(decision, vitality_state);
        CallerIdeHeartbeatReport::new(
            self.caller_pid,
            is_alive,
            is_eviction_immune,
            age_ms,
            reason,
            vitality_state,
            decision,
        )
    }

    /// Determines OS process vitality and heartbeat state classification.
    fn determine_vitality_state<F>(
        &self,
        now_ms: u64,
        vitality_probe: &F,
    ) -> (bool, WatchdogVitalityStateType)
    where
        F: Fn(u32) -> bool,
    {
        let caller_pid = match self.caller_pid {
            Some(pid) => pid,
            None => return (false, WatchdogVitalityStateType::Uninitialized),
        };
        if !vitality_probe(caller_pid) {
            return (false, WatchdogVitalityStateType::DeadProcess);
        }
        if self.is_heartbeat_fresh(now_ms) {
            (true, WatchdogVitalityStateType::HealthyActive)
        } else {
            (true, WatchdogVitalityStateType::StaleHeartbeat)
        }
    }

    /// Maps decision type and vitality state to human-readable explanation text.
    fn resolve_protection_reason(
        decision: CallerEvictionDecisionType,
        vitality_state: WatchdogVitalityStateType,
    ) -> String {
        match decision {
            CallerEvictionDecisionType::ProtectedActiveCaller
            | CallerEvictionDecisionType::ProtectedFreshHeartbeat => {
                EvictionProtectionReasonType::ActiveCallerWithLivePid
                    .as_str()
                    .to_string()
            }
            CallerEvictionDecisionType::EvictionPermittedForcedOverride => {
                EvictionProtectionReasonType::ForcedOverrideBypass
                    .as_str()
                    .to_string()
            }
            CallerEvictionDecisionType::EvictionPermittedDifferentTarget => {
                EvictionProtectionReasonType::TargetNotCaller
                    .as_str()
                    .to_string()
            }
            CallerEvictionDecisionType::EvictionPermittedCallerDead => {
                EvictionProtectionReasonType::CallerPidNotAlive
                    .as_str()
                    .to_string()
            }
            CallerEvictionDecisionType::EvictionPermittedHeartbeatStale => {
                EvictionProtectionReasonType::HeartbeatExpired
                    .as_str()
                    .to_string()
            }
            CallerEvictionDecisionType::EvictionPermittedNoCallerTracked => {
                if vitality_state == WatchdogVitalityStateType::Uninitialized {
                    EvictionProtectionReasonType::NoCallerConfigured
                        .as_str()
                        .to_string()
                } else {
                    EvictionProtectionReasonType::TargetNotCaller
                        .as_str()
                        .to_string()
                }
            }
        }
    }
}
