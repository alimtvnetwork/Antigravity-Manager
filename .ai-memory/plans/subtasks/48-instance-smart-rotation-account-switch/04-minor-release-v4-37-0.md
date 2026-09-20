# Subtask 04: Minor Release v4.37.0 Ceremony

## Status: Completed

## Parent Plan
[Plan 48: Instance Smart Rotation, Process Termination, Account Selection Algorithm & Minor Release v4.37.0](.ai-memory/plans/pending/48-instance-smart-rotation-account-switch.md)

## Goal
Execute version bump from `4.36.0` to `4.37.0` across all project manifests and finalize the automated release ceremony.

## Key Actions
1. Synchronize version `4.37.0` across:
   - `version.json`
   - `package.json`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`
2. Update documentation and changelogs.
3. Record modified files into test inventory if applicable.
4. Consolidate plan and subtasks into `.ai-memory/plans/completed/48-instance-smart-rotation-account-switch.md`.
5. Atomic git commit, tag `v4.37.0`, and push to remote `main`.

## Constraints
- Single atomic commit at the end.
- Strict adherence to release guidelines.
