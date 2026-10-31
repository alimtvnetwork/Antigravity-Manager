# Subtask 02: Batch 2 (Imp 277–282: Auto-Switcher Score Serialization, Cooldown Watchdog & Headroom Ratios)

- **Parent Plan**: `.ai-memory/plans/pending/166-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/166-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Targeted Improvements
- [x] **Imp-277: Candidate Score JSON Snapshotter** (`src-tauri/src/modules/auto_switcher/score_snapshot.rs`)
- [x] **Imp-278: Account Cooldown Expiration Watchdog** (`src-tauri/src/modules/auto_switcher/cooldown_watchdog.rs`)
- [x] **Imp-279: Weekly Quota-to-Time Headroom Ratio Calculator** (`src-tauri/src/modules/auto_switcher/weekly_headroom_ratio.rs`)
- [x] **Imp-280: Target Model Quota Filter** (`src-tauri/src/modules/auto_switcher/model_quota_filter.rs`)
- [x] **Imp-281: Switch Audit Reason Sanitizer** (`src-tauri/src/modules/auto_switcher/switch_reason_sanitizer.rs`)
- [x] **Imp-282: Candidate Account List Deduplicator** (`src-tauri/src/modules/auto_switcher/candidate_dedup.rs`)

## Verification Summary
- `score_snapshot.rs`: JSON snapshotter capturing candidate ranking, factors, and assigned ranks.
- `cooldown_watchdog.rs`: Calculates dynamic wake durations until earliest cooldown expiration.
- `weekly_headroom_ratio.rs`: Headroom ratio comparing actual weekly quota against ideal consumption slope.
- `model_quota_filter.rs`: Direct and family wildcard matching for target model quota checks.
- `switch_reason_sanitizer.rs`: OAuth token and email masking filter protecting privacy in audit records.
- `candidate_dedup.rs`: Deduplicates candidate accounts by ID and email, retaining the highest composite score.
