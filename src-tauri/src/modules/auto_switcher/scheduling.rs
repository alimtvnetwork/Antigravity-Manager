use crate::models::config::AutoProfileSwitcherConfig;

use super::*;

/// Minimum Google quota check frequency bound (lowered to 10s so configured caution/critical ladders are respected)
pub const MIN_GOOGLE_QUOTA_CHECK_SECONDS: u32 = 10;

pub fn google_quota_interval_seconds(configured: u32) -> u32 {
    let secs = if configured == 300 { 120 } else { configured };
    secs.max(MIN_GOOGLE_QUOTA_CHECK_SECONDS)
}

pub fn determine_quota_stage(
    quota_percent: Option<f64>,
    cfg: &AutoProfileSwitcherConfig,
) -> &'static str {
    let Some(quota) = quota_percent else {
        return "normal";
    };

    let caution_threshold = if cfg.low_quota_threshold_percent > cfg.critical_threshold_percent {
        cfg.low_quota_threshold_percent
    } else {
        18.0
    };

    if quota <= cfg.critical_threshold_percent {
        "critical"
    } else if quota < caution_threshold {
        "caution"
    } else {
        "normal"
    }
}

/// Calculate dynamic polling interval based on credit quota ladder.
/// Configured caution (e.g. 60s) and critical (e.g. 40s) intervals are properly respected.
pub fn calculate_next_interval_seconds(
    quota_percent: Option<f64>,
    cfg: &AutoProfileSwitcherConfig,
) -> u32 {
    let Some(quota) = quota_percent else {
        return google_quota_interval_seconds(cfg.check_interval_seconds);
    };

    let caution_threshold = if cfg.low_quota_threshold_percent > cfg.critical_threshold_percent {
        cfg.low_quota_threshold_percent
    } else {
        18.0
    };

    if quota <= cfg.critical_threshold_percent {
        google_quota_interval_seconds(cfg.critical_interval_seconds)
    } else if quota < caution_threshold {
        google_quota_interval_seconds(cfg.caution_interval_seconds)
    } else {
        google_quota_interval_seconds(cfg.check_interval_seconds)
    }
}
