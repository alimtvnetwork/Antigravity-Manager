use crate::models::config::AutoProfileSwitcherConfig;

use super::*;

/// Minimum Google quota check frequency bound (strict >= 30s floor)
pub const MIN_GOOGLE_QUOTA_CHECK_SECONDS: u32 = 30;

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

/// Dynamic adaptive interval ladder scaling automatically based on lowest observed quota:
/// - > 50%: 60 seconds (healthy status)
/// - <= 20% and > 10%: 30 seconds (caution status)
/// - <= 10%: 15 seconds (critical threshold emergency polling)
/// - None: 45 seconds (default balanced baseline)
pub fn calculate_adaptive_polling_interval(lowest_quota_percent: Option<f64>) -> std::time::Duration {
    match lowest_quota_percent {
        Some(q) if q > 50.0 => std::time::Duration::from_secs(60),
        Some(q) if q > 20.0 => std::time::Duration::from_secs(45),
        Some(q) if q > 10.0 => std::time::Duration::from_secs(30),
        Some(_) => std::time::Duration::from_secs(15),
        None => std::time::Duration::from_secs(45),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_google_quota_check_seconds_floor() {
        assert_eq!(MIN_GOOGLE_QUOTA_CHECK_SECONDS, 30);
        assert!(google_quota_interval_seconds(10) >= 30);
        assert_eq!(google_quota_interval_seconds(10), 30);
        assert_eq!(google_quota_interval_seconds(45), 45);
    }
}
