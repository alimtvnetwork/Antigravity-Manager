# Subtask 01: Quota Threshold Defaults Alignment to 12%

## Description
Set default `low_quota_threshold_percent` and `quota_protection.threshold_percentage` to `12.0` / `12` in Rust backend (`src-tauri/src/models/config.rs`) and matching frontend settings/types.

## Actions
- Update `src-tauri/src/models/config.rs`: `default_low_quota_threshold_percent() -> f64 = 12.0`.
- Update `src-tauri/src/models/config.rs`: `default_quota_protection_threshold() -> u32 = 12`.
- Update `src/types/config.ts` if default constants exist.
- Update `src/pages/Settings.tsx` to default to 12%.
