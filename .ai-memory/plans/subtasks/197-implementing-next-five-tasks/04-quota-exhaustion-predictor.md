# Subtask 04: Quota Exhaustion Velocity Predictor
Status: COMPLETED
Sequence: 197
Owner: Subagent-1
Target Module: `src-tauri/src/modules/auto_switcher/quota_exhaustion_predictor.rs`
Test Suite: `src-tauri/src/modules/auto_switcher/tests_quota_exhaustion_predictor.rs`
Objective: Calculate token burn rate velocity and forecast exhaustion time horizon before HTTP 429 lockouts.

Evidence:
- Module `src-tauri/src/modules/auto_switcher/quota_exhaustion_predictor.rs` created with `UsageObservation`, `BurnRateVelocity`, `ExhaustionSeverity`, and `predict_quota_exhaustion`.
- Test suite `src-tauri/src/modules/auto_switcher/tests_quota_exhaustion_predictor.rs` created covering velocity calculations, constant burn rates, unsorted timestamps, and severity classifications.
