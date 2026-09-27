# Subtask 04: CLI Backup & Restore Parity in `agm.rs`

## Description
Expose intuitive CLI commands in `agm`:
- `agm backup` (alias `agm backpack`, `agm backup-running-prompts`, `agm brp`)
- `agm restore` (alias `agm restore-running-prompts`, `agm rrp`, `agm resend-running`)
- `agm prompt backup` and `agm prompt restore` under `cmd_prompt_dispatch`

## Actions
- Edit `src-tauri/src/bin/agm.rs`.
