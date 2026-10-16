# Subtask 05: Fleet Remote JSON Connectivity & RCA Documentation

- **Task ID**: `Task-05`
- **Subtask Path**: `.ai-memory/plans/subtasks/ui-and-functionality-enhancements/05-fleet-remote-json-connectivity-and-rca.md`
- **Specification**: `02-spec/21-app/ui-and-functionality-enhancements/02-backend-fleet-and-rca-spec.md`
- **Parent Plan**: `.ai-memory/plans/ui-and-functionality-enhancements.md`
- **Target Modules**:
  - `src-tauri/src/modules/fleet_sync.rs`
  - `src-tauri/src/commands/fleet.rs`
  - `src-tauri/src/modules/notification_hub/email_alerts.rs`
  - `src-tauri/src/modules/repo_db/agy.rs`
  - `src-tauri/src/commands/instance.rs`
  - `src-tauri/src/bin/agm/fleet_cmds.rs`
- **Status**: `READY_FOR_EXECUTION`
- **Lead / Assignee**: Worker 02 (Backend Engineer)

---

## 1. Objectives & Scope

1. **Formalize & Verify Root Cause Analysis (RCA)**:
   - Verify that all six historical prompt dispatch and enqueue failure modes (F1 lingering worker lockout, F2 poisoned mutexes, F3 uncaught task panics, F4 no CDP fallback, F5 missing queue drain hook, F6 UI button coupling) have verified test coverage and adhere to runtime invariants.
   - Confirm poison resilience in `get_active_agy_workers()`, panic trapping in `run_caught_prompt_task`, and automated FIFO queue advancement in `drain_next_queued_prompt_for_workspace`.

2. **Fleet Remote JSON Connectivity Enforcement**:
   - Standardize fleet telemetry serialization strictly to pure JSON (`FleetSyncPayload` schema `agm-fleet-sync-v1.0`).
   - Validate ingestion of remote machine nodes, running instance counts, bound accounts, and active workspace prompt counts across Email IMAP, Telegram bot, and Supabase PostgREST transports.
   - Enforce the `[AGM IN-USE]` account lease schema for multi-machine account collision prevention and fast-forward coordination.

3. **CLI & IPC Parity**:
   - Ensure `agm fleet nodes`, `agm fleet broadcast`, and Tauri commands (`get_fleet_sync_machines`, `generate_fleet_sync_json`, `ingest_fleet_sync_json`) return clean, validated machine state without panics or silent errors.

---

## 2. Technical Requirements

### 2.1 RCA Verification Matrix
- [ ] Confirm `spawn_or_replace_prompt_via_agy` terminates lingering dead or hanging child processes using `sysinfo::System` and `proc.kill()` when `replace_existing == true`.
- [ ] Guarantee `ACTIVE_AGY_WORKERS.lock()` recovers gracefully from poisoned states (`into_inner()`).
- [ ] Ensure `run_caught_prompt_task` wraps all Tauri commands with `std::panic::catch_unwind(AssertUnwindSafe(...))` and logs stack traces to `tracing::error!`.
- [ ] Verify `submit_prompt_via_cdp_for_instance` is dispatched alongside CLI worker spawning to populate the open GUI IDE composer DOM.
- [ ] Verify that background thread `agy-monitor-<prompt_id>` automatically calls `drain_next_queued_prompt_for_workspace` upon process completion.
- [ ] Verify that queue queries strictly enforce `ORDER BY created_at ASC, id ASC`.

### 2.2 Fleet Sync Pure JSON Requirements
- [ ] `generate_local_fleet_sync_payload` produces valid `FleetSyncPayload` conforming to `$schema: "agm-fleet-sync-v1.0"`.
- [ ] Ingesting an external node payload via `ingest_fleet_sync_payload` correctly populates `FleetNodes`, `FleetInstances`, and `FleetWorkspaces` in `fleet_sync.db`.
- [ ] Offline threshold is strictly enforced: nodes with $(\text{now} - \text{last\_heartbeat\_at}) \ge 300\text{s}$ or `is_online == 0` display as offline.
- [ ] Account lease broadcast in `email_alerts.rs` produces `[AGM IN-USE]` subject and JSON body with 1h expiry (`lease_duration_seconds: 3600`).
- [ ] Candidate scoring in auto-switch checks `fleet_sync.db` active leases and excludes in-use accounts from candidate selection.

---

## 3. Implementation Steps

### Step 1: RCA Invariant Verification
1. Review `src-tauri/src/modules/repo_db/agy.rs` to ensure:
   - `get_active_agy_workers()` handles poisoned locks safely.
   - Lingering processes are terminated via `kill()` when `replace_existing` is true.
   - Monitor thread calls `drain_next_queued_prompt_for_workspace` on exit.
2. Review `src-tauri/src/commands/instance.rs` to ensure `run_caught_prompt_task` surrounds `send_prompt_now` and `enqueue_prompt`.

### Step 2: Fleet JSON Ingestion & Synchronization
1. Inspect `src-tauri/src/modules/fleet_sync.rs` and verify:
   - Schema version is `agm-fleet-sync-v1.0`.
   - Node hardware, IP, version, instances, accounts, and workspaces serialize/deserialize cleanly.
   - `get_fleet_summary_machines()` self-ingests the local heartbeat so local machine always appears in fleet views.
2. Verify IPC commands in `src-tauri/src/commands/fleet.rs`:
   - `get_fleet_sync_machines`
   - `generate_fleet_sync_json`
   - `ingest_fleet_sync_json`

### Step 3: Account Lease Collision Shielding
1. Verify `email_alerts.rs` generates `[AGM IN-USE | <node> | <ip>] <email> (1h lease)` alerts with full JSON telemetry.
2. Verify cross-channel signaling prevents dual-node usage of identical Google accounts.

---

## 4. Acceptance Criteria & Definition of Done

- [ ] All 6 RCA failure modes have explicit code protections and invariants documented in `02-backend-fleet-and-rca-spec.md`.
- [ ] `cargo check` and `cargo test` pass for `repo_db` and `fleet_sync` modules without regressions.
- [ ] `generate_fleet_sync_json` returns a formatted JSON string containing `$schema`, `node`, `instances`, `accounts`, and `workspaces`.
- [ ] `ingest_fleet_sync_json` parses sample JSON without errors and updates `fleet_sync.db`.
- [ ] Strict relative paths used in all docs.
- [ ] No silent errors: fallible operations route to `AppError` or `crate::error::record_ignored`.
