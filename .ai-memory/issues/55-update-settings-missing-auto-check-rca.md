# Issue 55: update_last_check_time fails when update_settings.json lacks auto_check

Status: fixed (2026-09-30)
Raised: 2026-09-30
Spec: [85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)

## 1. Symptom
Closing the update modal on `/accounts` triggered `tauri.update_last_check_time`, which returned `Failed to parse settings: missing field auto_check at line 38 column 1` (reported as E9001).

## 2. Trigger
An on-disk `update_settings.json` written before `auto_check` and `last_check_time` existed on `UpdateSettings`.

## 3. Root cause
`UpdateSettings` in `update_checker.rs` required `auto_check` and `last_check_time` without `#[serde(default)]`, so `serde_json::from_str` failed on older files. `update_last_check_time` calls `load_update_settings()` and propagates the parse error.

## 4. Why it escaped
New fields were added without backward-compatible deserialization and without a migration test.

## 5. Fix
Added `#[serde(default = "default_true")]` on `auto_check` and `#[serde(default)]` on `last_check_time`, plus unit test `update_settings_deserialize_missing_auto_check_and_last_check_time`.

## 6. Prevention
Any new field on persisted JSON settings must use `serde(default)` or an explicit migration; add a deserialize test for the previous on-disk shape.

## 7. Regression check
`cargo test update_settings_deserialize` (requires MSVC linker on Windows build hosts).
