# Subtask 03: Batch 3 (Imp 103–108: Auto-Switcher Adaptive Quota & Scoring Primacy)

- **Parent Plan**: `.ai-memory/plans/pending/160-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/160-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- **Imp-103: Adaptive Dynamic Quota Poller Interval** (`src-tauri/src/modules/auto_switcher/scheduling.rs`): Adaptive ladder scaling from 60s (>50%) down to 30s (<=20%) and 15s (<=10% emergency).
- **Imp-104: Quota Jitter Shield** (`src-tauri/src/modules/auto_switcher/jitter.rs`): Pseudo-randomized xorshift variation (default ±10%) preventing synchronous cluster bursts.
- **Imp-105: Proportional Fallback Score Calculator** (`src-tauri/src/modules/auto_switcher/scoring.rs`): Explicit helper `calculate_proportional_fallback_score` computing `(q_4h / 100.0) * weekly_score`.
- **Imp-106: Cooldown Expiry Countdown Helper** (`src-tauri/src/modules/auto_switcher/cooldown_timer.rs`): Real-time countdown calculation and human duration formatter (`format_cooldown_duration`).
- **Imp-107: Candidate Score Reason Breakdown Formatter** (`src-tauri/src/modules/auto_switcher/breakdown.rs`): Detailed explanation of tier multipliers, 4h quota gate checks, weekly scores, and fallback paths.
- **Imp-108: Dual-Key Sibling Conflict Checker** (`src-tauri/src/modules/auto_switcher/conflict_checker.rs`): Sibling collision validator matching both `account_id` and `bound_email` across instances.

## Targeted Improvements
- **Imp-103: Adaptive Dynamic Quota Poller Interval** (`src-tauri/src/modules/auto_switcher/daemon.rs`)
- **Imp-104: Quota Jitter Shield** (`src-tauri/src/modules/auto_switcher/jitter.rs`)
- **Imp-105: Proportional Fallback Score Calculator** (`src-tauri/src/modules/auto_switcher/scoring.rs`)
- **Imp-106: Cooldown Expiry Countdown Helper** (`src-tauri/src/modules/auto_switcher/cooldown_timer.rs`)
- **Imp-107: Candidate Score Reason Breakdown Formatter** (`src-tauri/src/modules/auto_switcher/breakdown.rs`)
- **Imp-108: Dual-Key Sibling Conflict Checker** (`src-tauri/src/modules/auto_switcher/conflict_checker.rs`)
