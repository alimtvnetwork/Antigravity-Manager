# Component and End-to-End Specification: Account Switch, IDE Detection, Prompts Management & Audit Trail

| Field | Value |
|---|---|
| Spec | 143, part 02 |
| Slug | `143-e2e-testing-account-switch-prompts-and-release` |
| Created | 2026-10-06 |
| Status | planned |
| Architecture spec | `02-spec/21-app/143-e2e-testing-account-switch-prompts-and-release/01-architecture-spec.md` |
| Parent plan | `.ai-memory/plans/143-e2e-testing-account-switch-prompts-and-release.md` |
| Subtasks owned by this spec | `.ai-memory/plans/subtasks/143-e2e-testing-account-switch-prompts-and-release/04-prompts-queue-and-heartbeat-e2e.md`, `.ai-memory/plans/subtasks/143-e2e-testing-account-switch-prompts-and-release/05-logs-audit-and-cli-ui-parity.md` |
| Reused E2E material | `02-spec/21-app/92-cli-instance-switch-e2e.md`, `02-spec/21-app/93-e2e-cli-commands-ai-instruction.md`, `02-spec/21-app/94-blind-ai-instance-test-and-release.md`, `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/02-component-and-e2e-spec.md` |

---

## 1. Executive Summary & Scope

This specification establishes the comprehensive component interface definitions and end-to-end (E2E) testing framework for Antigravity-Manager (AGM) across three primary operational tiers:
1. **Native CLI Binary (`agm`)**: The terminal entry point handling status inquiries, instance observation, prompt queries, backups, restorations, audit histories, and logs inspection.
2. **Tauri IPC Command Layer (`src-tauri/src/commands/`)**: The bridge exposing backend state machines, SQLite database access, and instance lifecycle controls to frontend consumers.
3. **Frontend UI Views & Modals (`src/pages/`, `src/components/`)**: The visual user interface featuring instance overview tables, prompt conversation trees, audit trail drawers, and account management views.

The E2E test matrix defined in Section 4 verifies the end-to-end stability of active IDE detection, multi-database state consistency across account rotations, prompt queuing and task handoff, real-time 5-second heartbeat continuity, and split SQLite audit logging.

---

## 2. Component Interface Definitions

### 2.1 CLI Interface Catalog (`src-tauri/src/bin/agm.rs`)

The `agm` CLI binary exposes standardized subcommands supporting human-readable output as well as `--json` machine-readable output.

| Command | Purpose | Key Arguments & Flags | Exit Codes |
|---|---|---|---|
| `agm status` (aliases: `credits`, `credit`) | Displays proxy gateway health, upstream endpoints, token pool, and account quota. | `--json` | `0`: Success, `1`: Internal Error |
| `agm observe <instance>` | Performs real-time inspection of active IDE PIDs, bound vs injected email (credential drift detector), active prompts, and heartbeat freshness. | `<instance_id_or_path>`, `--json` | `0`: Clean / Observed, `1`: Error, `2`: Missing Target |
| `agm wpr` (alias: `agm running-prompts ls`) | Discovers active in-flight prompts across workspace conversation databases. | `-i, --instance <id>`, `--words <N>`, `--limit <N>`, `--full`, `--json` | `0`: Success, `1`: Query Error |
| `agm running-prompts` | Root command for prompt snapshotting, restoration, inspection, and migration. | Subcommands: `ls`, `backup`, `restore`, `export`, `import` | `0`: Success, `1`: Command Error |
| `agm running-projects` (alias: `agm projects`) | Enumerates workspace folders currently bound to active IDE sessions with last conversation snippet. | `--json`, `-f <path.json>`, `--ssh` | `0`: Success, `1`: Failed Detection |
| `agm tree [all]` | Hierarchical tree view displaying projects and conversations, step counts, running status, and prompt preview snippets. | `-i, --instance <id>`, `--words <N>`, `--json` | `0`: Success, `1`: Parse Error |
| `agm prompt <text>` | Directly dispatches or enqueues a prompt into the specified workspace instance. | `-i, --instance <id>`, `--repo <path>`, `enqueue` subcommand | `0`: Dispatched, `1`: Delivery Failure |
| `agm brp` (alias: `agm backup-running-prompts`) | Performs a parallel snapshot of running prompts across workspaces into `backup-prompts.db`. | `-i, --instance <id>`, `-f <file>`, `--json` | `0`: Snapshot OK, `1`: Backup Error |
| `agm rrp` (alias: `agm restore-running-prompts`) | Re-injects backed-up prompts post-rotation into targeted workspaces. | `-i, --instance <id>`, `--keep`, `-f <file>`, `--json` | `0`: Restored, `1`: Restore Error |
| `agm rrc` | Resends in-flight commands into active workspace IDE sessions. | `-i, --instance <id>`, `--json` | `0`: Resent, `1`: Dispatch Error |
| `agm history` (alias: `agm audit`) | Queries sharded task history audit records across `task_index.db` and `history-*.db`. | `--limit <N>`, `--action <code\|name>`, `--offset <N>`, `--json` | `0`: Success, `1`: DB Error |
| `agm logs` (alias: `agm log`) | Tails application and proxy logs with log-level filtering. | `--level <lvl>`, `--tail <N>`, `--json`, `clean` | `0`: Stream OK, `1`: Read Error |

#### CLI Output Contracts

##### 1. `agm observe <instance> --json`
```json
{
  "instance_id": "test-cli-flow-a",
  "data_dir": "~/.antigravity_tools/instances/test-cli-flow-a/data",
  "bound_email": "user-a@example.com",
  "injected_email": "user-a@example.com",
  "credential_drift": false,
  "pids": [14208, 18392],
  "active_prompts_count": 1,
  "heartbeat": {
    "is_running": true,
    "is_fresh": true,
    "last_iteration": 42,
    "pid": 14208,
    "log_path": "./my-project/.antigravity_goal_prompt.log"
  }
}
```

##### 2. `agm running-projects --json`
```json
[
  {
    "id": "proj-uuid__test-cli-flow-a",
    "project_name": "backend-core",
    "project_path": "/home/user/workspace/backend-core",
    "instance_id": "test-cli-flow-a",
    "is_running": true,
    "pids": [14208],
    "last_conversation_id": "conv-88219",
    "last_conversation_short_id": "c882",
    "step_count": 14,
    "last_prompt_snippet": "Implement database connection pool"
  }
]
```

##### 3. `agm history --limit 1 --json`
```json
[
  {
    "id": "task-uuid-9912",
    "action_code": 3,
    "action": "SwitchAccount",
    "status": "Success",
    "subject": "Switch account to user-b@example.com",
    "detail": "Rotated instance test-cli-flow-a to account-b",
    "instance_id": "test-cli-flow-a",
    "from_email": "u***a@example.com",
    "to_email": "u***b@example.com",
    "created_at": 1728211000,
    "finished_at": 1728211002
  }
]
```

---

### 2.2 Tauri IPC Command Registry (`src-tauri/src/commands/`)

All IPC handlers follow the standard Rust `Result<T, E>` pattern, serializable via serde to frontend JSON promises.

| IPC Command | Signature | Source File | Description |
|---|---|---|---|
| `list_instances` | `() -> Result<Vec<InstanceStatus>, String>` | `instance.rs` | Returns all registered instances, their active status, bound account, and conscious PIDs. |
| `create_instance` | `(name: String, bound_account_id: Option<String>, from_instance_id: Option<String>) -> Result<InstanceConfig, String>` | `instance.rs` | Clones or initializes a new instance sandbox profile. |
| `switch_account_to_instance` | `(account_id: String, instance_id: Option<String>) -> Result<(), AppError>` | `instance.rs` | Executes token injection into SQLite profile databases and triggers prompt backup/restore. |
| `fast_forward_instance` | `(instance_id: String) -> Result<String, String>` | `instance.rs` | Triggers immediate rotation using the highest-scored available account. |
| `list_running_projects` | `() -> Result<Vec<RunningProject>, String>` | `instance.rs` | Queries `repo_prompts.db` for projects with active IDE execution states. |
| `list_backed_up_prompts` | `() -> Result<Vec<ActivePrompt>, String>` | `instance.rs` | Lists prompts currently staged in `backup-prompts.db` awaiting post-switch restoration. |
| `resume_recent_project_prompts` | `(instance_id: Option<String>, max_age_seconds: Option<i64>) -> Result<AutoResumeResult, String>` | `instance.rs` | Re-enqueues recent prompts from SQLite history into active workspaces. |
| `get_project_conversation_tree` | `(instance_id: Option<String>, max_words: Option<usize>, only_running: Option<bool>, force: Option<bool>) -> Result<Vec<AgmProjectTreeNode>, String>` | `instance.rs` | Generates hierarchical tree of workspaces, conversations, step counts, and prompt snippets. |
| `get_instance_switch_history` | `(instance_id: String, limit: Option<u32>) -> Result<InstanceSwitchHistoryResponse, String>` | `instance.rs` | Retrieves recent account switch audit records for a specific instance with full metadata. |
| `get_instance_audit_trail` | `(instance_id: String, limit: Option<u32>) -> Result<Vec<TaskRecord>, String>` | `instance.rs` | Fetches raw task records filtered by target instance. |
| `sync_instance_pid_and_quota` | `(instance_id: String) -> Result<InstanceStatus, String>` | `instance.rs` | Force-polls conscious process PIDs and current upstream quota for an instance. |
| `sync_all_instances_and_quotas` | `() -> Result<Vec<InstanceStatus>, String>` | `instance.rs` | Synchronizes PIDs and quotas across all registered instance profiles. |
| `list_task_history` | `(offset: Option<usize>, limit: Option<usize>) -> Result<TaskHistoryPage, String>` | `commands/mod.rs` | Paginated query across sharded `history-*.db` excluding heavyweight `payload_json`. |
| `get_task_history_detail` | `(id: String) -> Result<TaskDetail, String>` | `commands/mod.rs` | On-demand lazy fetch of complete task payload JSON for the audit detail drawer. |
| `clear_logs` | `() -> Result<(), String>` | `commands/mod.rs` | Purges transient execution and diagnostic log buffers. |

---

### 2.3 Frontend UI Views & Components

#### 1. `src/pages/Instances.tsx` & `InstanceTable.tsx`
- **Instance Overview**: Displays instance status cards or table rows with sequence badges (`#01 default`, `#02 test-cli-flow`).
- **Conscious PID Indicators**: Visual indicator (green dot + PID pill) showing live IDE process attachment.
- **Contextual Pill Toolbar**: Segregated action pills (`Switch`, `Backup`, `Restore`, `Tree`, `Audit`, `Restart`).
- **Sync Trigger**: Automatic 5-second polling via `sync_all_instances_and_quotas`.

#### 2. `src/components/instances/PromptTreeViewModal.tsx`
- **Hierarchical Tree View**: Project folder nodes expandable to conversation leaves.
- **Conversation Badges**: Dual sequence codes (`[AGM:P001 | GM:#1]`), short conversation ID, step counter, running badge.
- **Prompt Snippets**: 200-word preview of initial prompt and tail snippet of the latest agent turn.
- **Action Controls**: "Send Now" button, "Enqueue Prompt" input form, and "Copy Prompt" action.

#### 3. `src/components/instances/InstanceAuditTrailModal.tsx`
- **Recent Switch History**: Displays the last 2-3 account rotation cycles for the selected instance.
- **Step-by-Step Lifecycle Telemetry**: Shows backup verification, token injection in `state.vscdb`, process restart confirmation, and prompt restore status.
- **Email Privacy**: Automatic masking of source and destination emails (`u***1@example.com`).

#### 4. `src/pages/Audit.tsx` (`/audit`)
- **Global Audit Trail Table**: Sharded pagination with 100/200 item limits.
- **Action Code Filtering**: Dropdown filtering by `AddAccount (1)`, `UpdateAccount (2)`, `SwitchAccount (3)`, `SchedulePrompt (4)`, `RequeueConversation (5)`.
- **Lazy Detail Drawer**: Clicking a row opens a slide-over drawer calling `get_task_history_detail` to render formatted payload JSON without stalling the main table.

#### 5. `src/pages/Accounts.tsx`
- **Quota Visualization**: Dual progress bars for 5-hour rolling quota window and 7-day weekly quota limit.
- **Account Health**: Upstream status badge (Active, Rate Limited, Expired).

---

## 3. Storage Architecture: 3-Database Invariant

Antigravity-Manager persists state across three decoupled SQLite database systems to guarantee isolation, high concurrency (WAL mode), and data integrity:

```
+----------------------------------------------------------------------------------------------------+
|                                    Three-Database Architecture                                     |
+----------------------------------------------------------------------------------------------------+
  1. Profile Storage: state.vscdb (per instance)
     - Location: ~/.antigravity_tools/instances/<id>/data/User/globalStorage/state.vscdb
     - Table: ItemTable (key-value store)
     - Keys: Token credentials, session cookies, telemetry machine GUIDs.
     - Role: Injected during switch_account_to_instance to authenticate IDE.

  2. Prompts Repository: repo_prompts.db
     - Location: ~/.antigravity_tools/repo_prompts.db
     - Tables:
       * active_prompts: id, instance_id, project_id, repo_path, session_id, status, created_at, updated_at
       * running_projects: id, project_name, project_path, instance_id, is_running, pids, last_conversation_id
       * agm_conversation_sequences: seq_id, conversation_id, project_path, instance_id
     - Role: Real-time tracking of active running and queued prompts.

  3. Prompt Backups: backup-prompts.db
     - Location: ~/.antigravity_tools/backup-prompts/backup-prompts.db
     - Tables:
       * prompt_backups: id, instance_id, prompt_id, repo_path, prompt_text, session_id, batch_id, created_at
       * backup_batches: batch_id, instance_id, count, created_at
     - Role: Immutable snapshot store holding in-flight prompts during credential switch.

  *. Task History & Audit Shards: task_index.db & history-*.db
     - Location: ~/.antigravity_tools/data/task_history/
     - Tables: splits (in task_index.db), task_history (in history-*.db)
     - Cap: SPLIT_ROW_CAP = 500 rows per shard.
     - Role: Complete immutable audit log of all account rotations and scheduled operations.
+----------------------------------------------------------------------------------------------------+
```

---

## 4. End-to-End Test Matrix (E2E-01 to E2E-06)

### E2E-01: Running IDE Detection & Conscious PID Discovery
- **Objective**: Verify that AGM correctly detects running IDE processes, associates them with the appropriate instance profile directory, extracts conscious PIDs, and excludes helper/renderer worker processes.
- **Preconditions**:
  - Test instance profile initialized (`test-cli-flow-a`).
  - IDE or mock IDE process launched with target `--user-data-dir`.
- **Execution Steps**:
  1. Launch IDE instance bound to `test-cli-flow-a` data directory.
  2. Execute CLI: `agm status --json`. Verify gateway sees active environment.
  3. Execute CLI: `agm observe test-cli-flow-a --json`.
  4. Query IPC: `list_running_projects()`.
  5. Inspect UI: Verify `Instances.tsx` displays active status indicator and correct PID badge.
- **Expected Results & Assertions**:
  - `agm observe` JSON output contains `pids` array with length >= 1.
  - Resolved PID matches the conscious process holding open handle on `state.vscdb`.
  - `credential_drift` evaluates to `false`.
  - `repo_prompts.db::running_projects` contains row matching project path and instance ID.
- **Verification Query**:
  ```sql
  SELECT project_name, instance_id, is_running, pids
  FROM running_projects
  WHERE instance_id = 'test-cli-flow-a';
  ```

---

### E2E-02: Account Switch Execution & 3-Database State Verification
- **Objective**: Execute an account switch on a target instance and verify atomic state synchronization across `state.vscdb`, `repo_prompts.db`, `backup-prompts.db`, and `task_index.db`.
- **Preconditions**:
  - Instance `test-cli-flow-a` is running bound to `account-alpha@example.com`.
  - In-flight active prompt recorded in `active_prompts`.
  - Target account `account-beta@example.com` has valid OAuth refresh tokens.
- **Execution Steps**:
  1. Trigger switch via CLI: `agm switch account-beta@example.com --instance test-cli-flow-a`.
  2. Alternatively trigger switch via IPC: `switch_account_to_instance(beta_id, Some("test-cli-flow-a"))`.
  3. Inspect `state.vscdb` in instance data directory.
  4. Inspect `repo_prompts.db::active_prompts`.
  5. Inspect `backup-prompts.db::prompt_backups`.
  6. Inspect `task_index.db` and active shard in `history-*.db`.
- **Expected Results & Assertions**:
  - **Database 1 (`state.vscdb`)**: `ItemTable` value for `antigravity.auth.token` contains updated access/refresh token matching `account-beta`.
  - **Database 2 (`repo_prompts.db`)**: Pre-switch prompts transitioned from `running` to `backed_up` and post-switch restored to `dispatched`.
  - **Database 3 (`backup-prompts.db`)**: A new `backup_batches` entry exists with corresponding `prompt_backups` rows stamped with `instance_id = 'test-cli-flow-a'`.
  - **Audit Shard**: A new `task_history` row with `action_code = 3` (`SwitchAccount`), `from_email` matching alpha, and `to_email` matching beta.
- **Verification Queries**:
  ```sql
  -- Check backup record
  SELECT id, instance_id, prompt_id, batch_id
  FROM prompt_backups
  WHERE instance_id = 'test-cli-flow-a'
  ORDER BY created_at DESC LIMIT 1;

  -- Check audit entry in latest shard
  SELECT action_code, action, status, from_email, to_email, instance_id
  FROM task_history
  WHERE instance_id = 'test-cli-flow-a'
  ORDER BY created_at DESC LIMIT 1;
  ```

---

### E2E-03: Prompt Tracking (Running, Queued & Last Conversations)
- **Objective**: Verify discovery and attribution of running prompts, queued prompts, and active projects with their most recent conversation summaries and step counts across CLI, IPC, and UI.
- **Preconditions**:
  - Instance `test-cli-flow-a` contains conversation logs in its local Gemini directory (`brain/<cid>/transcript.jsonl`).
  - `active_prompts` table contains seeded prompts in `running` and `queued` states.
- **Execution Steps**:
  1. Execute CLI: `agm wpr -i test-cli-flow-a --words 100 --json`.
  2. Execute CLI: `agm running-prompts ls -i test-cli-flow-a --json`.
  3. Execute CLI: `agm tree -i test-cli-flow-a --json`.
  4. Query IPC: `get_project_conversation_tree(Some("test-cli-flow-a"), Some(200), Some(false), Some(true))`.
  5. Open UI: Inspect `PromptTreeViewModal.tsx` for `test-cli-flow-a`.
- **Expected Results & Assertions**:
  - CLI `agm wpr` returns active prompts filtered strictly to `test-cli-flow-a`.
  - CLI `agm tree` outputs JSON containing `AgmProjectTreeNode` with `conversations` array.
  - Each conversation leaf displays correct sequence codes (`[AGM:P001 | GM:#1]`), step count > 0, and non-empty prompt preview.
  - UI `PromptTreeViewModal` renders tree nodes with corresponding status badges and step counts.
- **Verification Query**:
  ```sql
  SELECT id, instance_id, repo_path, session_id, status
  FROM active_prompts
  WHERE instance_id = 'test-cli-flow-a';
  ```

---

### E2E-04: Prompt Enqueueing & Task Resumption Document Generation
- **Objective**: Enqueue a new prompt for execution, verify queue persistence in SQLite, confirm generation of the atomic resumption document `.antigravity_resume_task.json`, and track dispatch state progression.
- **Preconditions**:
  - Target workspace directory initialized (`repo-a`).
  - AGM proxy daemon or CLI runner active.
- **Execution Steps**:
  1. Enqueue prompt via CLI: `agm prompt enqueue -i test-cli-flow-a --repo /path/to/repo-a "Test Prompt E2E-04"`.
  2. Verify insertion into `repo_prompts.db::active_prompts` with `status = 'queued'`.
  3. Trigger dispatch check: `check_and_dispatch_enqueued_prompts`.
  4. Inspect workspace filesystem for `.antigravity_resume_task.json`.
  5. Read and validate JSON document content.
- **Expected Results & Assertions**:
  - `active_prompts` row created with row ID `queued-test-cli-flow-a-<uuid>` and `status = 'queued'`.
  - When dispatched, status updates atomically to `'dispatched'`.
  - File `.antigravity_resume_task.json` created in `/path/to/repo-a` with schema:
    ```json
    {
      "task_id": "prompt-uuid",
      "instance_id": "test-cli-flow-a",
      "prompt": "Test Prompt E2E-04",
      "created_at": 1728211050,
      "status": "pending"
    }
    ```
  - File permissions allow read/write by IDE process.

---

### E2E-05: 5-Second Real-Time Heartbeat Continuity Across Account Rotations
- **Objective**: Verify that the detached background prompt heartbeat runner maintains 5-second interval ticks in `.antigravity_goal_prompt.log`, survives or cleanly resumes across account rotations, preserves monotonic iteration numbering (`Iteration: N -> N+1`), and maintains `AGM_INSTANCE_STATUS.md`.
- **Preconditions**:
  - Heartbeat script `scripts/prompt_heartbeat_runner.py` present and executable.
  - Target workspace has an active prompt.
- **Execution Steps**:
  1. Launch heartbeat worker:
     ```bash
     python scripts/prompt_heartbeat_runner.py start --log-path ./repo-a/.antigravity_goal_prompt.log --instance test-cli-flow-a --prompt-id p-101
     ```
  2. Wait 15 seconds (allow 3 iterations: Iteration 1, 2, 3).
  3. Execute check: `python scripts/prompt_heartbeat_runner.py check --log-path ./repo-a/.antigravity_goal_prompt.log`. Verify `FRESH=True`.
  4. Query CLI: `agm observe test-cli-flow-a --json`. Verify `heartbeat.is_fresh = true`.
  5. Simulate account rotation: Cleanly stop worker via `python scripts/prompt_heartbeat_runner.py stop --log-path ...`.
  6. Restart worker post-switch:
     ```bash
     python scripts/prompt_heartbeat_runner.py start --log-path ./repo-a/.antigravity_goal_prompt.log --instance test-cli-flow-a --prompt-id p-101
     ```
  7. Wait 10 seconds (allow 2 more iterations).
  8. Inspect `.antigravity_goal_prompt.log` and workspace `AGM_INSTANCE_STATUS.md`.
- **Expected Results & Assertions**:
  - Log entries are spaced strictly 5 seconds apart ($\pm 0.5\text{s}$).
  - Post-switch restart detects prior `Iteration: 3` and starts at `Iteration: 4` (monotonic continuity, zero gap).
  - Freshness check verifies file age <= 9.0s (`interval + 4.0s` tolerance).
  - `AGM_INSTANCE_STATUS.md` is updated with active PID, iteration count, and status `RUNNING`.

---

### E2E-06: Split SQLite Audit Trail Logging & CLI/UI Parity
- **Objective**: Verify that user and automated actions generate structured audit entries in sharded databases (`task_index.db` and `history-*.db`), enforce the 500-row rotation cap (`SPLIT_ROW_CAP`), support lazy payload loading, mask emails in list views, and maintain strict data parity across CLI (`agm history`), Tauri IPC (`list_task_history`), and UI (`/audit`).
- **Preconditions**:
  - Master index `task_index.db` and active shard initialized.
  - Several audit actions performed (AddAccount, SwitchAccount, SchedulePrompt).
- **Execution Steps**:
  1. Execute CLI: `agm history --limit 50 --json`.
  2. Call IPC: `list_task_history(Some(0), Some(50))`.
  3. Compare CLI and IPC output lists: verify record count, IDs, action codes, and timestamps match identically.
  4. Confirm that list outputs omit `payload_json` (lazy payload invariant).
  5. Request specific detail via IPC: `get_task_history_detail(record_id)`. Confirm full `payload_json` is returned.
  6. Verify email masking in list views (`maskEmail` applied to `from_email` and `to_email`).
  7. In simulated high-volume test: append rows up to 500. Verify shard automatically closes and new `history-<ts>.db` shard is registered in `task_index.db::splits`.
- **Expected Results & Assertions**:
  - CLI `agm history --json` and IPC `list_task_history` produce identical serialized records.
  - Heavyweight payload JSON is loaded strictly on-demand.
  - Shard table `splits` in `task_index.db` tracks shard file names, row counts, and `is_current` flags.
  - UI `/audit` displays masked emails and renders the slide-over drawer upon row selection.

---

## 5. CLI, IPC, and UI Parity Matrix

The following table cross-references system capabilities across CLI, IPC, and UI implementations to ensure functional and behavioral equivalence:

| Operational Capability | CLI Binary Command | Tauri IPC Handler | UI View / Modal Component | Shared Core Service |
|---|---|---|---|---|
| View instance list & status | `agm status`, `agm instances ls --json` | `list_instances()` | `Instances.tsx` (`InstanceTable`) | `modules::instance::list_instances` |
| Live process observation | `agm observe <id> --json` | `sync_instance_pid_and_quota(id)` | `Instances.tsx` (PID pills) | `modules::instance::observe_instance` |
| Account rotation / switch | `agm switch <acc> -i <id>` | `switch_account_to_instance(acc, id)` | `InstanceTable.tsx` (Switch pill) | `modules::instance::switch_account_to_instance` |
| View active running prompts | `agm wpr -i <id> --json` | `list_running_projects()` | `PromptTreeViewModal.tsx` | `modules::repo_db::list_running_projects` |
| View conversation tree | `agm tree -i <id> --json` | `get_project_conversation_tree(id, ...)` | `PromptTreeViewModal.tsx` | `modules::repo_db::get_project_conversation_tree_cached` |
| Enqueue prompt for workspace | `agm prompt enqueue -i <id> ...` | `enqueue_prompt(id, ...)` | `PromptTreeViewModal.tsx` (Form) | `modules::repo_db::enqueue_prompt_for_instance` |
| Backup running prompts | `agm brp -i <id> --json` | `backup_running_prompts_for_instance(id)` | `InstanceTable.tsx` (Backup pill) | `modules::backup_prompts_db::backup_prompts_for_instance` |
| Restore backed-up prompts | `agm rrp -i <id> --keep --json` | `restore_running_prompts_for_instance(id)` | `InstanceTable.tsx` (Restore pill) | `modules::backup_prompts_db::restore_backed_up_prompts_for_instance` |
| View audit history | `agm history --limit N --json` | `list_task_history(offset, limit)` | `Audit.tsx` (`/audit` route) | `modules::task_history_db::list_page` |
| View instance switch history | `agm history -i <id> --json` | `get_instance_switch_history(id, limit)` | `InstanceAuditTrailModal.tsx` | `modules::task_history_db::get_instance_switch_history` |
| View audit record details | `agm history <id> --json` | `get_task_history_detail(id)` | `Audit.tsx` (Detail slide-over) | `modules::task_history_db::get_detail` |

---

## 6. Acceptance Criteria

| ID | Criterion Description | Verification Method |
|---|---|---|
| **AC-01** | Running IDE instances are detected and attributed to their exact instance profiles with conscious PIDs. | `agm observe <id> --json` returns matching PIDs and `credential_drift: false`. |
| **AC-02** | Account switch atomically synchronizes token state across `state.vscdb`, `repo_prompts.db`, `backup-prompts.db`, and `task_index.db`. | Direct read-only SQLite inspection of all three stores immediately following switch. |
| **AC-03** | In-flight prompts are cleanly snapshotted into `backup-prompts.db` before process termination and restored post-relaunch. | Query `backup-prompts.db::prompt_backups` and `active_prompts` state transition. |
| **AC-04** | Prompt tree accurately exposes projects, conversation short codes, sequence codes `[AGM:P001 | GM:#1]`, and step counts. | CLI `agm tree --json` and IPC `get_project_conversation_tree` match UI modal tree. |
| **AC-05** | Prompt enqueueing creates `active_prompts` row and produces `.antigravity_resume_task.json` in workspace root. | Inspect filesystem and database row status progression `queued` -> `dispatched`. |
| **AC-06** | Real-time heartbeat worker records entries every 5 seconds and resumes iteration count $N \to N+1$ across rotation. | Verify `.antigravity_goal_prompt.log` timestamps and iteration continuity. |
| **AC-07** | Task history sharding enforces 500-row cap per `history-*.db` file and maintains master catalog in `task_index.db`. | Verify `splits` table row counts and new shard creation upon exceeding 500 rows. |
| **AC-08** | Full feature and data parity exists across CLI (`agm`), Tauri IPC commands, and React UI components. | Cross-validation against Section 5 Parity Matrix with identical serialized outputs. |
