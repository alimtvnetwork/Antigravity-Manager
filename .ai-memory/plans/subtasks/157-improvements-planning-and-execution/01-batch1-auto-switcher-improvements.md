# Subtask 01: Batch 1 Execution (Improvements 01–06: Auto-Switcher & Quota Robustness)

## Improvements In Scope
- **Imp-01**: Proportional Headroom Score Normalization (`src-tauri/src/modules/auto_switcher/scoring.rs`)
- **Imp-02**: Target Model Wildcard Hierarchy Matching (`src-tauri/src/modules/auto_switcher/scoring.rs`)
- **Imp-03**: Low-Quota Alert Settling Suppression (`src-tauri/src/modules/auto_switcher/daemon.rs`)
- **Imp-04**: Exclusion String Normalization (`src-tauri/src/modules/auto_switcher/selection.rs`)
- **Imp-05**: Cooldown Fallback Never-Used Priority (`src-tauri/src/modules/auto_switcher/selection.rs`)
- **Imp-06**: Pre-Switch Live Refresh Timeout Guard (`src-tauri/src/modules/auto_switcher/selection.rs`)

## Status
`[COMPLETED]`

## Verification Summary
- **Imp-01**: Normalization of weekly quota base score by `100.0` applied in `scoring.rs`.
- **Imp-02**: Target model wildcard hierarchy matching added for `gemini-*-flash-*`, `gemini-*-pro-*`, `gpt`, `o1`, `o3` in `scoring.rs`.
- **Imp-03**: 10-second post-switch settling suppression implemented in `daemon.rs` to eliminate false alarms during IDE boot.
- **Imp-04**: Exclusion strings normalized with `.trim().to_lowercase()` deduplication in `selection.rs`.
- **Imp-05**: Never-used accounts (`last_used == 0`) prioritized first in cooldown pool fallback sorting in `selection.rs`.
- **Imp-06**: Wrapped live Google API quota probe in 5000ms `tokio::time::timeout` guard in `selection.rs`.
