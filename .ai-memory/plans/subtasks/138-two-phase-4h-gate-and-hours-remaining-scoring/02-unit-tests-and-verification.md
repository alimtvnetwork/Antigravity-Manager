# Subtask 02: Unit Tests & Edge Case Verification

**Subtask Slug:** `02-unit-tests-and-verification`  
**Parent Task:** `138-two-phase-4h-gate-and-hours-remaining-scoring`  
**File:** `src-tauri/src/modules/auto_switcher.rs`  

---

## Technical Details
1. Update `test_score_candidate_account_reset_time_priority`:
   - Account A has elapsed reset time -> `is_period_finished == true` -> full score `3.0 * 168.0 = 504.0`.
   - Account B has 10% 4h quota with future reset time -> returns `0.0` in primary phase.
2. Update `test_score_candidate_account_weekly_quota_groups_bottleneck`:
   - Verify accounts with 100% 4h quota are ranked by `weekly_pct * (168 - hours_remaining)`.
3. Add `test_score_candidate_account_fallback_partial_4h`:
   - Verify fallback mode scales by `(q_4h / 100.0) * base_score`.
4. Add `test_weekly_quota_sub_8_percent_zero`:
   - Verify accounts with < 8% weekly quota strictly score `0.0`.
