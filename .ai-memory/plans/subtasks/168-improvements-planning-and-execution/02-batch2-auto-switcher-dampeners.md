# Subtask 02: Batch 2 (Imp 337–342: Auto-Switcher Jitter & Dampeners)

- **Parent Plan**: `.ai-memory/plans/pending/168-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/168-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- Created `jittered_velocity_estimator.rs` applying bounded micro-jitter to rotation calculations.
- Created `bounce_dampener.rs` suppressing rapid flip-flop switching between near-identical accounts.
- Created `quota_window_align.rs` prioritizing accounts approaching their 5h replenishment reset.
- Created `switch_settling_watchdog.rs` enforcing the mandatory 10s settling delay after account switch.
- Created `multimodel_composite_scorer.rs` computing combined weighted score across model families.
- Created `stale_binding_sweeper.rs` releasing orphaned bindings when child IDE processes terminate.
- Re-exported all items in `src-tauri/src/modules/auto_switcher/mod.rs`.

## Targeted Improvements
- **Imp-337: Jittered Quota Velocity Estimator** (`src-tauri/src/modules/auto_switcher/jittered_velocity_estimator.rs`)
- **Imp-338: Rapid Account Bounce Dampener** (`src-tauri/src/modules/auto_switcher/bounce_dampener.rs`)
- **Imp-339: Candidate Quota Window Align Checker** (`src-tauri/src/modules/auto_switcher/quota_window_align.rs`)
- **Imp-340: Profile Switch Settling Watchdog** (`src-tauri/src/modules/auto_switcher/switch_settling_watchdog.rs`)
- **Imp-341: Multi-Model Headroom Composite Scorer** (`src-tauri/src/modules/auto_switcher/multimodel_composite_scorer.rs`)
- **Imp-342: Stale Binding Sweeper Sentinel** (`src-tauri/src/modules/auto_switcher/stale_binding_sweeper.rs`)
