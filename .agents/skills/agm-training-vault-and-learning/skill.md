---
name: agm-training-vault-and-learning
description: Specialized skill for managing the machine training vault (training_vault.db), reinforcement learning telemetry REST API endpoints, routing adjustments, and node telemetry queries in src-tauri/src/modules/training_api.rs.
---

# AGM Machine Training Vault & Telemetry Subsystem

This skill provides comprehensive architectural guidance, schema definitions, and REST endpoint specifications for the Machine Training Vault and Reinforcement Learning subsystem in **Antigravity-Manager (AGM)**.

---

## 1. Subsystem Architecture Overview

The Training subsystem provides an external telemetry ingestion and reinforcement learning feedback interface, allowing orchestrators and automated AI training loops to monitor cluster node health, capture inference latency/quality scores, and dynamically tune model routing:

```
+----------------------------------------------------------------------------------------------------+
|                                    External AI / Training Loops                                    |
|    - Query Node Telemetry: GET /api/training/telemetry                                             |
|    - Record Training / Inference Event: POST /api/training/log                                     |
|    - Adjust Model Routing Dynamically: POST /api/training/adjust-routing                           |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                   Security & Feature Guard                                         |
|    - Check training_api_enabled toggle in app_config.json                                          |
|    - If disabled: Immediately reject with HTTP 403 FORBIDDEN (TRAINING_API_DISABLED)               |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                SQLite Training Vault Database                                      |
|                               %APPDATA%/data/training_vault.db                                     |
|    - WAL mode enabled, 5000ms busy timeout                                                         |
|    - Table `training_logs`: id, session_id, model, prompt_type, input_tokens, output_tokens,       |
|      latency_ms, success, score, feedback, adjust_routing, created_at                              |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/modules/training_api.rs` | Axum router for training endpoints, SQLite `training_vault.db` connection and schema initialization, `NodeTelemetry` synthesis, and dynamic routing adjustment. |
| `src-tauri/src/commands/mod.rs` | Tauri IPC commands: `get_training_api_status`, `set_training_api_status`, `get_training_telemetry`. |
| `src-tauri/src/bin/agm.rs` | CLI handler `cmd_test_training` for verifying telemetry and database records. |
| `src/pages/Settings.tsx` | UI toggle switch for enabling/disabling the Machine Training REST API. |

---

## 3. Core Invariants & Rules

1. **Security Guard by Default**:
   - The Training REST API must be strictly disabled by default (`training_api_enabled = false`).
   - Any unauthenticated or unauthorized access attempt when disabled must return `403 Forbidden` with the standard error payload `{ "code": "TRAINING_API_DISABLED" }`.

2. **WAL Concurrency & Safe Connection Pooling**:
   - `connect_training_db()` must enable WAL mode (`PRAGMA journal_mode = WAL`) and set a busy timeout of 5,000ms to eliminate file lock contention between concurrent telemetry writes and UI reads.

3. **Node Telemetry Completeness**:
   - `NodeTelemetry` must accurately report the current system state, including OS, CPU architecture, active accounts, instance PIDs, active auto-switcher threshold, and the count of currently running/queued prompts.

4. **Dynamic Routing Safety**:
   - Routing adjustments submitted via `/api/training/adjust-routing` must validate model identifiers against known models before writing to proxy mapping tables, preventing invalid model fallbacks.

---

## 4. Verification & Testing

```bash
# 1. Quality pre-flight checks
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features

# 2. Test via CLI
agm test-training
```
