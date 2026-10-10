use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSwitcherStatus {
    pub is_running: bool,
    pub active_instance_id: String,
    pub active_account_email: Option<String>,
    pub current_quota_percent: Option<f64>,
    pub last_check_timestamp: i64,
    pub last_switch_timestamp: Option<i64>,
    pub last_switch_reason: Option<String>,
    #[serde(default)]
    pub monitored_instance_count: usize,
    #[serde(default)]
    pub monitored_instances: Vec<InstanceQuotaSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InstanceQuotaSummary {
    pub instance_id: String,
    pub instance_name: String,
    pub bound_email: Option<String>,
    pub quota_percent: Option<f64>,
    pub reset_time_iso: Option<String>,
    pub seconds_until_reset: Option<i64>,
    pub is_running: bool,
    pub is_depleted_before_finish: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuotaPeriodStatus {
    pub quota_percent: f64,
    pub reset_time_iso: Option<String>,
    pub reset_timestamp: Option<i64>,
    pub seconds_until_reset: Option<i64>,
    pub is_period_finished: bool,
    pub is_depleted_before_finish: bool,
    #[serde(default)]
    pub is_low_quota: bool,
    #[serde(default)]
    pub below_threshold: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecoverySnapshot {
    pub instance_id: String,
    pub account_id: String,
    pub timestamp: i64,
    pub reason: String,
    pub is_recovered: bool,
}

pub(crate) struct AutoSwitcherRuntimeState {
    pub is_running: bool,
    pub last_check_timestamp: i64,
    pub last_switch_timestamp: Option<i64>,
    pub last_switch_reason: Option<String>,
    pub next_check_timestamp: i64,
    pub check_interval_seconds: u32,
    pub current_stage: String, // "normal" | "caution" | "critical"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSwitcherDaemonStatus {
    pub is_daemon_running: bool,
    pub last_evaluated_at: i64,
    pub next_check_timestamp: i64,
    pub next_check_in_seconds: i64,
    pub check_interval_seconds: u32,
    pub current_stage: String,
    pub active_account_email: Option<String>,
    pub current_quota_percent: f64,
    pub monitored_instance_count: usize,
}

pub(crate) static RUNTIME_STATE: Lazy<Mutex<AutoSwitcherRuntimeState>> = Lazy::new(|| {
    Mutex::new(AutoSwitcherRuntimeState {
        is_running: false,
        last_check_timestamp: 0,
        last_switch_timestamp: None,
        last_switch_reason: None,
        next_check_timestamp: 0,
        check_interval_seconds: 120,
        current_stage: "normal".to_string(),
    })
});
