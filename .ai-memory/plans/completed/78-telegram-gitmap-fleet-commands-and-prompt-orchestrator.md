# Plan 78: Telegram & AGM GitMap AGY Parity, VM Fleet Orchestration & Remote Prompt Routing (Consolidated)

## Metadata
- **Status:** Completed
- **Created:** 2026-09-27
- **Completed:** 2026-09-27
- **Target Release:** v4.83.0 / v4.84.0
- **Total Execution Loops / Steps:** 18
- **Authority Spec:** [02-spec/21-app/58-telegram-gitmap-fleet-commands-and-prompt-orchestrator.md](../../../02-spec/21-app/58-telegram-gitmap-fleet-commands-and-prompt-orchestrator.md)
- **Root Cause Analysis:** [02-spec/22-app-issues/20-installer-upstream-fork-inversion-rca.md](../../../02-spec/22-app-issues/20-installer-upstream-fork-inversion-rca.md)

---

## 1. Summary of Execution & Objectives

This parent task originated from the user directive requiring:
1. Complete elimination of all upstream fork fallback paths (`lbjlaq/Antigravity-Manager`) in installation scripts (`install.ps1`, `install.sh`, `deploy/arch/install.sh`).
2. Telegram & AGM CLI commands mirroring GitMap AGY capabilities:
   - `/active`, `/running` (compact table of active running prompts).
   - `/queues`, `/queue` (workspace prompt queues).
   - `/projects`, `/workspaces` (discovered projects, IDs, and sample prompt syntax).
   - `/prompts`, `/templates` (available reusable prompt templates).
   - `/nodes`, `/node ls` (VM cluster fleet topology and connectivity).
   - `/nodes <alias> prompts` (node-scoped running prompts).
   - `/prompt <node> <project> <text>` (cross-machine remote prompt injection).
3. Telegram 4096-character limit protection via 3800-character line-boundary message chunking with 80ms transmission pacing and HTML parsing error fallback.

---

## 2. Consolidated Subtask Outcomes

### Subtask 01: Installer Upstream Fork Zero-Tolerance Audit
- **Status:** Completed
- **Target Files:** `install.ps1`, `install.sh`, `deploy/arch/install.sh`, `README_EN.md`
- **Outcome:** Verified zero occurrences of `lbjlaq` in download URLs or API fallback routines. Verified `powershell.exe -File .\install.ps1 -DryRun` resolves exclusively to `alimtvnetwork/Antigravity-Manager` and exits with code 0.

### Subtask 02: Telegram & AGM GitMap AGY Active and Queues Parity
- **Status:** Completed
- **Target Files:** `src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/bin/agm.rs`
- **Outcome:** Implemented `format_active_prompts_report()` and `format_prompt_queues_report()`. Integrated `/active`, `/running`, `/queues`, `/queue` in Telegram command processor and `agm agy active`/`agm agy queues`/`agm active`/`agm queues` in CLI.

### Subtask 03: Telegram Cluster Nodes Topology & GitMap Credentials Integration
- **Status:** Completed
- **Target Files:** `src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/bin/agm.rs`
- **Outcome:** Enhanced `/nodes` and `/node ls` to aggregate GitMap cluster mesh (`gitmap cluster status`), Supabase Root DB `nodes` table, and local host telemetry. Formatted rich status cards with online/idle/offline badges, IP, uptime, and last seen timestamps.

### Subtask 04: Node-Scoped Running Prompts, Projects Catalog & Reusable Prompt Templates
- **Status:** Completed
- **Target Files:** `src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/bin/agm.rs`
- **Outcome:** Implemented `/nodes <alias> prompts` with flexible alias parsing. Updated `/projects` with concrete sample prompt invocations. Implemented `format_prompts_templates_report()` for `/prompts` and `/prompt ls`.

### Subtask 05: Cross-Machine Remote Prompt Injection Routing & CLI Parity
- **Status:** Completed
- **Target Files:** `src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/bin/agm.rs`
- **Outcome:** Handled `/prompt <node> <project> <text>` and `/prompt <project> <text>` routing locally to `repo_db`/`agy` and remotely to GitMap cluster SSH delegation or Supabase queue. Added top-level commands `agm nodes`, `agm projects`, `agm active`, `agm queues`, and `agm agy` in `src-tauri/src/bin/agm.rs`. Guaranteed all Telegram responses are safely chunked under 4000 characters.
