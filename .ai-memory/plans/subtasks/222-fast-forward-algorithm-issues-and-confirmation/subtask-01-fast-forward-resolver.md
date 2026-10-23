# Subtask 01: Fast-Forward Target Account Resolver Refactoring

## 1. Objective
Refactor `src-tauri/src/modules/auto_switcher/fast_forward_target_resolver.rs` to implement the complete 4-tier Fast-Forward account selection logic specified by the user:
1. **Weekly Quota Floor**: Any account with weekly quota < `min_weekly_quota_percent` (default: 15.0%) is treated as having 0 credits and is unconditionally skipped.
2. **Four-Hour Clean Headroom & Fleet Invariant**:
   - Four-hour quota must be 100% (`rolling_4h_quota_percent >= 100.0`, with 99.5% float tolerance).
   - Must NOT be active on any other local or remote instance.
   - Must NOT have been used in the last 4 hours by any source (SSH, Email, GitMap, Telegram, local instance bindings: `now_epoch_seconds - last_used_epoch_seconds >= 14400` or `last_used_epoch_seconds == 0`).
3. **Primary Choice (Earliest Expiry)**:
   - Among eligible primary candidates, sort ascending by `reset_epoch_seconds` (closest time to expire).
   - Break ties by higher weekly quota, then subscription tier multiplier.
4. **Degraded Fallback (When No 100% 4h Candidate Exists)**:
   - Filter candidates meeting the weekly quota floor (`weekly_quota_percent >= min_weekly_quota_percent`) and not active on other instances.
   - Calculate combined score: `tier_multiplier * (weekly_quota_percent + rolling_4h_quota_percent)`.
   - Sort descending by score.
5. **Output**:
   - `FfResolutionOutcome::FoundCandidate { account_id, email, selection_tier }`
   - `FfResolutionOutcome::NoEligibleCandidates { reason }`

## 2. Disjoint File Ownership
- `src-tauri/src/modules/auto_switcher/fast_forward_target_resolver.rs`
