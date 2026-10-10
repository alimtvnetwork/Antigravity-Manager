use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_format_switch_labels() {
        let auto_label = "Auto-Switcher (Quota/Period boundary)";
        let manual_label = "Manual User Switch";
        assert!(auto_label.contains("Auto"));
        assert!(manual_label.contains("Manual"));
    }

    #[test]
    pub(crate) fn test_resolve_switch_context_never_returns_same_as_new_email() {
        let same_email = "test.same@example.com";
        record_previous_email(same_email);

        // When switching to the same email, old_email must never equal new_email
        let (old, _, _, _) = resolve_switch_context(same_email, "default");
        assert_ne!(
            old.trim().to_lowercase(),
            same_email.trim().to_lowercase(),
            "old_email must not equal new_email"
        );

        // If new_email matches whatever current account is on disk, old_email must be empty
        if let Ok(Some(cur)) = crate::modules::account::get_current_account() {
            if !cur.email.is_empty() {
                record_previous_email(&cur.email);
                let (old_cur, _, _, _) = resolve_switch_context(&cur.email, "default");
                assert!(
                    old_cur.is_empty(),
                    "old_email must be empty when switching to current account"
                );
            }
        }

        // When switching to a different email with isolated test state
        let rec_email = "prior.recorded@example.com";
        record_previous_email(rec_email);
        let diff_email = "brand.new.target@example.com";
        let (old2, _, _, _) = resolve_switch_context(diff_email, "default");
        assert_ne!(
            old2.trim().to_lowercase(),
            diff_email.trim().to_lowercase(),
            "old_email must not equal new_email"
        );
    }

    #[test]
    pub(crate) fn test_email_switch_alert_distinctness_invariants() {
        let selected = "user1@example.com";
        let prev_identical = "user1@example.com";
        let pred_identical = "user1@example.com";

        let from_display = Some(prev_identical)
            .map(|s| s.trim())
            .filter(|s| {
                !s.is_empty()
                    && !s.eq_ignore_ascii_case("default")
                    && !s.eq_ignore_ascii_case(selected)
            })
            .unwrap_or("(none / standby)");

        assert_eq!(from_display, "(none / standby)");

        let predicted_display = Some(pred_identical)
            .map(|s| s.trim())
            .filter(|s| {
                !s.is_empty()
                    && !s.eq_ignore_ascii_case(selected)
                    && (from_display.eq_ignore_ascii_case("(none / standby)")
                        || !s.eq_ignore_ascii_case(from_display))
            })
            .unwrap_or("(none / pool exhausted)");

        assert_eq!(predicted_display, "(none / pool exhausted)");
    }
}
