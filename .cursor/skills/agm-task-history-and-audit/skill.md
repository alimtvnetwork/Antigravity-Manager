---
name: agm-task-history-and-audit
description: Specialized skill for managing the task history and audit trail subsystem, 500-row auto-rotation split SQLite databases (task_index.db and history-*.db), action code mappings (AuditAction), and audit UI/CLI views in src-tauri/src/modules/task_history_db.rs.
---

# AGM Task History & Split-DB Audit Subsystem

This skill provides comprehensive architectural guidance, schema rotation protocols, and querying standards for the Task History & Audit Trail subsystem in **Antigravity-Manager (AGM)**.

---

## 1. Subsystem Architecture Overview

To provide an immutable, performant audit trail of all automated account switches, profile updates, prompt reinjections, and schedule executions without bloating a single SQLite database, AGM uses a Sharded Split-DB design:

```
+----------------------------------------------------------------------------------------------------+
|                                    Automated & User Actions                                        |
|    - Account Addition / Removal / Quota Update (AuditAction::AddAccount, UpdateAccount)            |
|    - Profile & Account Switching (AuditAction::SwitchAccount)                                      |
|    - Prompt Resumption & Queue Scheduling (AuditAction::SchedulePrompt, RequeueConversation)       |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                     Master Task Index Database                                     |
|                               %APPDATA%/data/task_history/task_index.db                             |
|    - Table `splits`: Tracks all split files, created_at, closed_at, row_count, is_current          |
|    - Automatic Split Rotation: When active shard reaches SPLIT_ROW_CAP (500 rows),                 |
|      closes current split and creates new shard: history-<timestamp>.db                            |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                 Rotating Shard Databases (history-*.db)                             |
|    - Table `task_history`: id, action_code, action, status, subject, detail, instance_id,         |
|      from_email, to_email, created_at, finished_at, payload_json                                   |
|    - Lazy Payload Loading: payload_json only fetched on demand via get_task_history_detail         |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                  Audit Consumer Interfaces                                         |
|    - Native CLI: `agm history` (supports pagination, filtering by action, JSON output)             |
|    - GUI Audit Page: /audit route (`src/pages/Audit.tsx`), InstanceAuditTrailModal                 |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/modules/task_history_db.rs` | Split DB lifecycle management, `SPLIT_ROW_CAP = 500`, `append_task()`, `query_task_history_page()`, `get_task_detail()`, and prune logic. |
| `src-tauri/src/modules/audit_action.rs` | `AuditAction` enum (1: AddAccount, 2: UpdateAccount, 3: SwitchAccount, 4: SchedulePrompt, 5: RequeueConversation), code/title resolution, and legacy string parsing. |
| `src/pages/Audit.tsx` | Web & Desktop Audit Trail table, pagination controls (100/200 items), search by email/instance, and detail view drawers. |
| `src/components/instances/InstanceAuditTrailModal.tsx` | Dedicated modal presenting the last 2-3 account switches for a specific IDE instance with full step-by-step lifecycle details. |
| `src-tauri/src/bin/agm.rs` | CLI handler `cmd_history` for inspecting task records from the terminal. |

---

## 3. Core Invariants & Rules

1. **Strict 500-Row Split Rotation**:
   - `SPLIT_ROW_CAP` is fixed at **500 rows**. Once a split reaches this threshold, `append_task()` must mark the split `closed_at` and initialize the next `history-<timestamp>.db` shard.
   - Cross-shard queries must aggregate from newest to oldest shards to maintain chronological pagination (`query_task_history_page`).

2. **Lazy Payload Invariant**:
   - The list query `query_task_history_page()` must NEVER return `payload_json` to keep table rendering fast and lightweight.
   - `payload_json` is strictly retrieved on-demand via `get_task_history_detail(id)` when the user opens the detail drawer or modal.

3. **Email Masking Invariant**:
   - When displaying audit records in the list view, emails must be presented with masked domains or masked characters to prevent accidental exposure during screen sharing.

4. **Action Code Consistency**:
   - Every logged action must store both the numeric `action_code` and its normalized PascalCase string (`SwitchAccount`, `SchedulePrompt`).

---

## 4. Verification & Testing

```bash
# 1. Quality pre-flight checks
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features

# 2. Test CLI history command
agm history --limit 10 --json
```
