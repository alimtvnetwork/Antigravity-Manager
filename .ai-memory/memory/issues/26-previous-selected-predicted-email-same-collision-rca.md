# Memory Issue 26: Previous, Selected, and Predicted Account Same-Email Collision

- **Spec Reference:** [02-spec/22-app-issues/26-previous-selected-predicted-email-same-collision-rca.md](../../../02-spec/22-app-issues/26-previous-selected-predicted-email-same-collision-rca.md)
- **Status:** `resolved`

## Root Cause Summary
1. `resolve_switch_context` in `notification_hub.rs` defaulted `old_email` to `cur_acc.email` when empty, even if `cur_acc.email == new_email`, causing `previous_email` to equal `selected_email`.
2. `select_candidate_profiles` in `auto_switcher.rs` used case-sensitive `<[String]>::contains()` for exclusion checking, allowing excluded accounts to be re-selected if casing or identifier type (ID vs. Email) mismatched.
3. `dispatch_email_switch_alert` only filtered `predicted_display` against `selected_display`, neglecting to filter against `from_display` (`previous_email`).

## Fix Applied
1. Enforced strict mutual exclusivity:
   - `previous_email != selected_email` (if equal, `previous_email` resets to `(none / standby)`).
   - `predicted_email != selected_email && predicted_email != previous_email` (if equal to either, resets to `(none / pool exhausted)`).
2. Centralized case-insensitive and trimmed exclusion matching in `is_account_excluded()` across `select_candidate_profiles()` and `trigger_manual_rotation_for_instance()`.
3. Injected both account ID and email into candidate exclusion sets.
