# 73 - Real-Time Running Prompt Heartbeat, Instance E2E, and SSH Key Management Specification

## Overview
This specification defines the architectural invariants and execution flows for:
1. **Real-Time Running Prompt Heartbeat Verification**: Establishing an active background heartbeat mechanism writing timestamps every 5 seconds to a project workspace log (`.antigravity_goal_prompt.log`), providing tamper-proof evidence that an in-flight prompt is running, backed up, cleanly suspended upon instance process termination, and faithfully re-invoked post-switch.
2. **Deterministic Multi-Instance Account Switching & Process Lifecycle**: Creating a dedicated isolated instance via CLI, tracking and observing it, backing up its running prompts to SQLite `backup_prompts.db`, consciously resolving its PID from data directory arguments, cleanly terminating only that instance, updating credentials in `state.vscdb` / `storage.json`, re-opening the instance, capturing Settings screenshot with visible system date & time and confirmed email, and re-invoking the prompt.
3. **AGM-GitMap SSH Fleet Interoperability**: Equipping `agm ssh` with full parity commands for deploying authorization keys (`deploy-keys`, `fix-auth`, `copy-id`) and exporting/importing nodes JSON (`nodes export-json`, `nodes import-json`).

## Architecture & Data Flow

### 1. 5-Second Heartbeat Prompt Worker
- **File Location**: `.antigravity_goal_prompt.log` in workspace directory.
- **Cadence**: Appends every 5 seconds with ISO-8601 UTC timestamp, active PID, instance ID, prompt ID, and sequence counter.
- **Lifecycle Guard**:
  - Pre-switch: File is actively updated every 5 seconds.
  - Switch: Instance and prompt process are terminated based on data directory path and PID. File modification stops.
  - Post-switch: Prompt is restored from `backup_prompts_db` and re-invoked. Sequence counter increments and new timestamps are appended.

### 2. Settings Screenshot Evidence Contract
- Must render:
  - System Date & Time in title and breadcrumb headers (`YYYY-MM-DD HH:MM:SS UTC`).
  - Confirmed authenticated email address matching target switch state.
  - Active verified process PID.
  - Bound workspace folder path.
  - Active prompt queue with recent heartbeat timestamp.

### 3. SSH Command Parity Matrix
- `agm ssh deploy-keys [targets...]`: Deploy public keys to remote fleet.
- `agm ssh fix-auth <target>`: Push public key to remote `~/.ssh/authorized_keys`.
- `agm ssh nodes export-json [path]`: Export cluster/node inventory to JSON.
- `agm ssh nodes import-json [path]`: Import cluster/node inventory from JSON.
