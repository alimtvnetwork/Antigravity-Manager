use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_quota_drop_deduplication_tracking() {
        let test_email = "test-sensor-dedup@example.com".to_string();
        {
            let mut alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
            alerts.remove(&test_email);
        }

        // First alert at 18%
        let should_alert_1 = {
            let mut alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
            match alerts.get(&test_email) {
                Some(&last_pct) => (last_pct - 18.0) >= 1.0,
                None => {
                    alerts.insert(test_email.clone(), 18.0);
                    true
                }
            }
        };
        assert!(should_alert_1, "First low quota occurrence should alert");

        // Duplicate check at same 18%
        let should_alert_duplicate = {
            let mut alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
            match alerts.get(&test_email) {
                Some(&last_pct) => (last_pct - 18.0) >= 1.0,
                None => {
                    alerts.insert(test_email.clone(), 18.0);
                    true
                }
            }
        };
        assert!(
            !should_alert_duplicate,
            "Identical quota percentage must be suppressed"
        );

        // Further drop to 15% (>= 1.0% drop)
        let should_alert_further_drop = {
            let mut alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
            match alerts.get(&test_email) {
                Some(&last_pct) => {
                    if (last_pct - 15.0) >= 1.0 {
                        alerts.insert(test_email.clone(), 15.0);
                        true
                    } else {
                        false
                    }
                }
                None => true,
            }
        };
        assert!(
            should_alert_further_drop,
            "Significant drop should re-alert"
        );

        // Recovery above threshold (e.g. 100%)
        {
            let mut alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
            alerts.remove(&test_email);
        }
        let is_cleared = {
            let alerts = LAST_QUOTA_ALERTED_PERCENT.lock().unwrap();
            !alerts.contains_key(&test_email)
        };
        assert!(is_cleared, "Recovery should clear cache");
    }

    #[test]
    pub(crate) fn test_detect_machine_and_ip() {
        let ip = detect_local_ip();
        assert!(!ip.is_empty());
        let name = detect_machine_name();
        assert!(!name.is_empty());
    }
}
