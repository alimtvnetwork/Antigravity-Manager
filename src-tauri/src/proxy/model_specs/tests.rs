use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_gemini_version_checks() {
        assert!(is_gemini_under_v3("gemini-2.5-flash"));
        assert!(is_gemini_under_v3("gemini-2.5-flash-lite"));
        assert!(is_gemini_under_v3("gemini-2.5-pro"));
        assert!(is_gemini_under_v3("gemini-2.0-flash"));
        assert!(is_gemini_under_v3("gemini-1.5-pro"));
        assert!(!is_gemini_under_v3("gemini-3.7-flash-high"));
        assert!(!is_gemini_under_v3("gemini-3-flash"));
        assert!(!is_gemini_under_v3("gemini-3-pro"));
        assert!(!is_gemini_under_v3("gemini-3.1-pro"));
        assert!(!is_gemini_under_v3("gemini-3.8-flash"));
        assert!(!is_gemini_under_v3("gemini-pro-agent"));
        assert!(!is_gemini_under_v3("claude-3-7-sonnet"));

        assert!(!is_gemini_v3_or_above("gemini-2.5-flash"));
        assert!(!is_gemini_v3_or_above("gemini-2.0-flash"));
        assert!(is_gemini_v3_or_above("gemini-3.7-flash-high"));
        assert!(is_gemini_v3_or_above("gemini-3.7-flash"));
        assert!(is_gemini_v3_or_above("gemini-3-flash"));
        assert!(is_gemini_v3_or_above("gemini-3-pro"));
        assert!(is_gemini_v3_or_above("gemini-3.1-pro"));
        assert!(is_gemini_v3_or_above("gemini-3.8-flash"));
        assert!(is_gemini_v3_or_above("gemini-pro-agent"));
        assert!(!is_gemini_v3_or_above("claude-3-7-sonnet"));
    }

    #[test]
    fn test_gemini_thinking_budget() {
        // Explicit -high suffix yields 10000 budget
        assert_eq!(get_thinking_budget("gemini-3.7-flash-high", None), 10000);
        assert_eq!(get_thinking_budget("gemini-3.8-flash-high", None), 10000);
        assert_eq!(get_thinking_budget("gemini-3.9-flash-high", None), 10000);

        // Explicit -medium suffix yields 4000 budget
        assert_eq!(get_thinking_budget("gemini-3.7-flash-medium", None), 4000);
        assert_eq!(get_thinking_budget("gemini-3.8-flash-medium", None), 4000);

        // Explicit -low suffix yields 1000 budget
        assert_eq!(get_thinking_budget("gemini-3.7-flash-low", None), 1000);
        assert_eq!(get_thinking_budget("gemini-3.8-flash-low", None), 1000);

        // Bare Gemini >= 3.0 models default to medium budget (4000)
        assert_eq!(get_thinking_budget("gemini-3.7-flash", None), 4000);
        assert_eq!(get_thinking_budget("gemini-3.8-flash", None), 4000);
        assert_eq!(get_thinking_budget("gemini-3.9-flash", None), 4000);

        // Pro family
        assert_eq!(get_thinking_budget("gemini-3.1-pro-high", None), 10001);
        assert_eq!(get_thinking_budget("gemini-pro-agent", None), 10001);
        assert_eq!(get_thinking_budget("gemini-3.1-pro-low", None), 1001);
    }

    #[test]
    fn test_resolve_authoritative_thinking_budget() {
        // 1. Explicit models: enforce dictionary budget, ignore client effort
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.7-flash-high", Some("low"), None, None),
            10000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget(
                "gemini-3.7-flash-high",
                Some("none"),
                None,
                None
            ),
            10000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro-low", Some("high"), None, None),
            1001
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-pro-agent", Some("low"), None, None),
            10001
        );

        // 2. Bare Flash models: adopt client effort
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("high"), None, None),
            10000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("max"), None, None),
            10000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("xhigh"), None, None),
            10000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("low"), None, None),
            1000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("extra-low"), None, None),
            1000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("medium"), None, None),
            4000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("default"), None, None),
            4000
        );

        // Bare Flash models: client omitted/disabled enforces -medium fallback (4000)
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", None, None, None),
            4000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("none"), None, None),
            4000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("disabled"), None, None),
            4000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("0"), None, None),
            4000
        );

        // 3. Bare Pro models: adopt client effort
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro", Some("high"), None, None),
            10001
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro", Some("low"), None, None),
            1001
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro", Some("medium"), None, None),
            10001
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro", None, None, None),
            10001
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro", Some("none"), None, None),
            10001
        );

        // 4. Gemini < 3 non-thinking models: return 0
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-2.5-flash", Some("high"), None, None),
            0
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-1.5-pro", None, None, None),
            0
        );

        // 5. Bare models ignore client_budget, authoritatively governed by effort tier or server fallback
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("high"), Some(1024), None),
            10000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", None, Some(1024), None),
            4000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", None, Some(32000), None),
            4000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro", None, Some(1000), None),
            10001
        );
    }

    #[test]
    fn test_bare_gemini_v36_or_above_flash() {
        assert!(is_bare_gemini_v36_or_above_flash("gemini-3.6-flash"));
        assert!(is_bare_gemini_v36_or_above_flash("gemini-3.7-flash"));
        assert!(is_bare_gemini_v36_or_above_flash("gemini-3.8-flash"));
        assert!(is_bare_gemini_v36_or_above_flash("gemini-3.9-flash"));
        assert!(is_bare_gemini_v36_or_above_flash("gemini-4.0-flash"));
        assert!(is_bare_gemini_v36_or_above_flash("GEMINI-3.7-FLASH"));

        // 3.5 and below does not match
        assert!(!is_bare_gemini_v36_or_above_flash("gemini-3.5-flash"));
        assert!(!is_bare_gemini_v36_or_above_flash("gemini-3-flash"));
        assert!(!is_bare_gemini_v36_or_above_flash("gemini-2.5-flash"));

        // Already having explicit suffix or variant marker does not match
        assert!(!is_bare_gemini_v36_or_above_flash("gemini-3.7-flash-high"));
        assert!(!is_bare_gemini_v36_or_above_flash(
            "gemini-3.7-flash-medium"
        ));
        assert!(!is_bare_gemini_v36_or_above_flash("gemini-3.7-flash-low"));
        assert!(!is_bare_gemini_v36_or_above_flash(
            "gemini-3.7-flash-tiered"
        ));
        assert!(!is_bare_gemini_v36_or_above_flash(
            "gemini-3.8-flash-tiered"
        ));
        assert!(!is_bare_gemini_v36_or_above_flash("gemini-3.7-pro"));
    }

    #[test]
    fn test_tiered_flash_budget_and_client_effort() {
        let mut tb = crate::proxy::config::ThinkingBudgetConfig::default();
        tb.flash_mode = crate::proxy::config::ThinkingBudgetMode::Custom;
        tb.flash_low = 1024;
        tb.flash_medium = 4096;
        tb.flash_high = 16384;
        tb.flash_tiered = -1;

        // 1. No effort parameter, and flash_tiered is -1: returns None (adaptive)
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-tiered", None, None, &tb, None),
            None
        );

        // 2. Ignore client numeric budget; no effort still returns None
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-tiered", None, Some(9999), &tb, None),
            None
        );

        // 3. Client effort low / medium / high mapped to flash_low, flash_medium, flash_high
        assert_eq!(
            resolve_custom_budget(
                "gemini-3.7-flash-tiered",
                Some("low"),
                Some(5000),
                &tb,
                None
            ),
            Some(1024)
        );
        assert_eq!(
            resolve_custom_budget(
                "gemini-3.7-flash-tiered",
                Some("extra-low"),
                None,
                &tb,
                None
            ),
            Some(1024)
        );
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-tiered", Some("medium"), None, &tb, None),
            Some(4096)
        );
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-tiered", Some("high"), None, &tb, None),
            Some(16384)
        );
        assert_eq!(
            resolve_custom_budget(
                "gemini-3.7-flash-tiered",
                Some("max"),
                Some(2048),
                &tb,
                None
            ),
            Some(16384)
        );

        // 4. Custom positive flash_tiered without client effort returns configured positive value
        tb.flash_tiered = 8192;
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-tiered", None, None, &tb, None),
            Some(8192)
        );

        // 5. Default mode without client effort returns None (adaptive)
        tb.flash_mode = crate::proxy::config::ThinkingBudgetMode::Default;
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-tiered", None, None, &tb, None),
            None
        );

        // 6. Named non-tiered model: locked to tier indicated by suffix, unaffected by client effort
        tb.flash_mode = crate::proxy::config::ThinkingBudgetMode::Custom;
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-high", Some("low"), None, &tb, None),
            Some(16384) // Preserves high tier setting (16384), never downgraded by client low effort
        );
    }

    #[test]
    fn test_resolve_gemini_3x_flash_tiered_wildcard() {
        // x > 8 matches and routes to 3.x-flash-tiered
        assert_eq!(
            resolve_gemini_3x_flash_tiered("gemini-3.9-flash"),
            Some("gemini-3.9-flash-tiered".to_string())
        );
        assert_eq!(
            resolve_gemini_3x_flash_tiered("gemini-3.10-flash"),
            Some("gemini-3.10-flash-tiered".to_string())
        );
        assert_eq!(
            resolve_gemini_3x_flash_tiered("gemini-3.9.1-flash"),
            Some("gemini-3.9.1-flash-tiered".to_string())
        );

        // x <= 8 must return None, governed by dedicated mappings
        assert_eq!(resolve_gemini_3x_flash_tiered("gemini-3.8-flash"), None);
        assert_eq!(resolve_gemini_3x_flash_tiered("gemini-3.7-flash"), None);
        assert_eq!(resolve_gemini_3x_flash_tiered("gemini-3.6-flash"), None);
        assert_eq!(resolve_gemini_3x_flash_tiered("gemini-3.5-flash"), None);

        // Non-flash models or those with suffixes do not match
        assert_eq!(
            resolve_gemini_3x_flash_tiered("gemini-3.9-flash-high"),
            None
        );
        assert_eq!(resolve_gemini_3x_flash_tiered("gemini-3.9-pro"), None);
    }
}
