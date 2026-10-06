# 62 - Switch Audit Read the Wrong Prompt Key and macOS Wrapper PID Hid a Running Instance (RCA)

- Status: solved
- Found: 2026-10-02 by research subagents "Trace prompt resume failure" (`c1bbbc83-dfbd-44af-ae6c-93679fb4229d`) and "Trace quota and PID scans" (`66597f07-852a-4a7f-836f-783f2139f8d1`)
- Fixed in: `366d84e5` ("fix(switch): audit default prompt and recover wrapper PID"), shipped in `v4.126.0`
- Related: issue 60, issue 61 (lists the wrapper symptom only), memory `.ai-memory/memory/learned/23-switch-audit-default-key-and-wrapper-pid-recovery.md`

## Symptom 1: empty prompt in the default switch audit entry

After v4.124.0, switching the default instance's account wrote an audit row whose prompt and conversation id were empty, even while a prompt was running.

### Root cause

`switch_account` in `src-tauri/src/modules/account.rs` called `switch_prompt_snapshot(target_ide)`. `target_ide` is the IDE flavor (`"ide"`, `"agy"`), while `repo_db` stores prompts under the instance id. The lookup never matched.

### Fix

`let snap = crate::modules::repo_db::switch_prompt_snapshot("default");` because this path always switches the default instance. Instance switches already pass the instance id.

## Symptom 2: running instance reported stopped (macOS)

### Root cause

On macOS the launch records the PID of the `open` wrapper, which exits after it starts the IDE. `is_instance_running` trusted only that saved PID, so a live instance returned false. The auto switcher crash watchdog and the UI then treated it as stopped.

### Fix

`is_instance_running(instance_id, data_dir, config_pid)` keeps the saved-PID identity check first. If it fails, it runs `find_pids_for_data_dir(data_dir, is_default)` once, records the first match with `record_instance_pid`, and returns true. No match returns false.

## Verification

- `cargo fmt -- --check`: clean.
- `cargo clippy --all-targets --all-features`: zero errors, no warnings on the touched lines.
- Tests: `saved_pid_identity_and_refresh_floor`, `switch_spares_another_instances_pid`, `switch_payload_keeps_from_to_reason_and_reinject` passed.
- Not verified: live switch on macOS; live audit row after a real default switch.

## Later change to watch

`a0c5cc18` and `3581207d` (v4.155.0) now also scan when the saved PID matches. See open ambiguity `02-is-instance-running-saved-pid-fast-path.md`.

## Prevention

- Key per-instance data by instance id, never by IDE flavor.
- Treat a saved PID as a hint and keep a bounded fallback search that rewrites it.
