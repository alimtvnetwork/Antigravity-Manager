# Subtask 02: Fast-Forward Target Account Resolver Unit Test Suite

## 1. Objective
Author `src-tauri/src/modules/auto_switcher/tests_fast_forward_target_resolver.rs` to provide comprehensive unit test coverage for `fast_forward_target_resolver.rs`:
1. `test_weekly_quota_floor_skips_sub_15_percent`:
   - Account with 14.9% weekly quota is skipped as zero credits.
   - Account with 15.0% weekly quota is considered eligible.
2. `test_sub_100_percent_4h_quota_skips_primary_tier`:
   - Account with 95% 4h quota cannot be selected in Primary tier.
3. `test_recent_fleet_usage_in_last_4h_skips_primary_tier`:
   - Account used 2 hours ago (within 14400s) is skipped in Primary tier.
   - Account used 5 hours ago (beyond 14400s) or never used is eligible.
4. `test_active_on_other_instance_excluded`:
   - Account flagged `is_active_on_other_instance == true` is skipped across all tiers.
5. `test_primary_tier_selects_closest_to_expire`:
   - Multiple 100% 4h accounts: the one with `reset_epoch_seconds` closest to `now` is chosen.
6. `test_degraded_fallback_selects_highest_combined_credits`:
   - When no 100% 4h account exists, selects highest combined `tier_multiplier * (weekly + 4h)`.
7. `test_configurable_weekly_threshold`:
   - Changing `min_weekly_quota_percent` to 25.0% correctly skips 20% accounts.

## 2. Disjoint File Ownership
- `src-tauri/src/modules/auto_switcher/tests_fast_forward_target_resolver.rs`
