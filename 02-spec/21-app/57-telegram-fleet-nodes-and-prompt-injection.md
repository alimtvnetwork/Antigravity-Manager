# Specification 57: Telegram Bot VM Cluster Fleet Nodes, Scoped Prompts & Long-Message Chunking

## Metadata
- **Version:** 1.0.0
- **Updated:** 2026-09-27
- **AI Confidence:** High (Production Invariant)
- **Ambiguity:** None (Strict Grounded Rules)

---

## 1. Problem Statement & Motivation
Operators managing distributed Antigravity Manager instances across remote virtual machines (VMs) need comprehensive command and control directly from Telegram without opening a browser or SSH session:
1. **Cluster Node Visibility (`/nodes`, `/node ls`):** Operators need to view all active cluster VM nodes (alias, IP, heartbeat, uptime, active accounts) by aggregating GitMap cluster fleet (`gitmap cluster status`) and Supabase Root DB `nodes` table.
2. **Node-Scoped Prompts Query (`/nodes <alias> prompts`):** Operators must be able to inspect currently executing or queued prompts on a specific VM node.
3. **Workspace & Project Catalog (`/projects`, `/prompts`):** Prompts cannot be dispatched without knowing the active project IDs or conversation IDs. Telegram must provide a compact registry of running projects and recent prompt IDs.
4. **Direct Prompt Injection (`/prompt <node> <project> <text>`):** Operators need to inject prompts remotely to specific nodes and projects, matching the capabilities of Email Inbound remote dispatch.
5. **Telegram 4096-Character Limit Protection:** Telegram Bot API strictly rejects any `sendMessage` payload exceeding 4096 UTF-8 characters with `400 Bad Request: message is too long`. High-volume telemetry (large prompt queues or multi-node clusters) must be chunked sequentially along newline boundaries.
6. **Headless & CLI Parity:** Ensure GUI, Telegram, and CLI (`agm telegram nodes`, `agm telegram prompts`, `agm telegram prompt`) share identical execution pathways.

---

## 2. Architectural Invariants

### 2.1 Telegram Message Chunking & Transport Safety
- **Maximum Chunk Threshold:** Set to `4000` characters (safely below Telegram's 4096-byte ceiling).
- **Boundary Preservation:**
  - Messages `<= 4000` characters are dispatched in a single `sendMessage` call.
  - Messages `> 4000` characters are split into contiguous chunks at newline boundaries (`\n\n` or `\n`), ensuring HTML tags and formatting remain intact.
  - Each chunk is transmitted with a brief pacing interval (`80ms`) to avoid Telegram API rate limits.
  - Fallback: If no newline is available within a 4000-character window, hard chunking occurs at unicode character boundaries.

### 2.2 Distributed Cluster Nodes Aggregation
The node collector compiles a unified topology using a three-tier probe:
1. **Tier 1: Supabase Root DB `nodes` Table:**
   - Queries `nodes` table (`status=eq.online&order=last_heartbeat_at.desc`).
   - Retrieves `alias`, `ip_address`, `uptime_seconds`, `project_count`, and `last_heartbeat_at`.
2. **Tier 2: GitMap Cluster Fleet Registry:**
   - Probes `gitmap cluster status` / `gitmap cluster nodes`.
   - Parses connected node aliases (`vm-01`, `vm-02`, `vm-03`, etc.) and liveness timestamps.
3. **Tier 3: Local Node Telemetry:**
   - Retrieves `node_alias` from `supabase_config.json`, local network IP via UDP socket, and process uptime.
4. **Deduplication:** Merges tiers by alias or IP, decorating each node with `🟢 ONLINE`, `⚪ IDLE`, or `🔴 OFFLINE`.

### 2.3 Scoped Running Prompts Query (`/nodes <alias> prompts`)
When an operator issues `/nodes <alias> prompts` or `/node <alias>`:
- If target matches the local node alias (or `local`):
  - Ingests active projects from `repo_db::get_live_project_execution_info()`.
  - Ingests queued / running prompts from `repo_db::list_all_prompts()`.
  - Formats a compact tabular view: project ID snippet, workspace directory, status (`running`, `idle`, `dispatched`), and prompt content snippet.
- If target is a remote node:
  - Queries Supabase Secondary DB `command_queue` / `prompt_queue` for records matching `target_node == alias`.
  - Queries GitMap cluster state if available.

### 2.4 Prompt Injection & Remote Dispatch (`/prompt`)
Syntax:
- `/prompt <node> <project> <prompt text>`
- `/prompt <project> <prompt text>` (defaults to local node)

Execution:
1. **Local Dispatch:**
   - Creates an `ActivePrompt` entry in `repo_db` with status `running` or `dispatched`.
   - Injects the resume instruction `.antigravity_resume_task.json` into the target workspace directory.
   - Spawns background execution via `agy` or native runner.
2. **Remote Dispatch:**
   - Inserts record into Supabase Secondary DB `command_queue` with payload:
     `{"id": "<uuid>", "target_node": "<node>", "project_id": "<project>", "command": "prompt", "prompt": "<text>", "status": "pending"}`.
   - Peers polling the secondary DB pick up and execute the prompt immediately.

### 2.5 Telegram Command Router Specification
The dispatcher in `src-tauri/src/modules/telegram_inbound.rs` supports:
- `/nodes`, `/nodes ls`, `/node ls` ➔ Format full cluster nodes table.
- `/nodes <alias> prompts`, `/node <alias> prompts`, `/node <alias>` ➔ Node-scoped prompt telemetry.
- `/projects`, `/workspaces` ➔ Discovered workspaces and project IDs.
- `/prompts`, `/prompts ls` ➔ Recent and active prompt queue.
- `/prompt <target> <proj> <text>` ➔ Prompt injection.
- `/gitmap <args>`, `/agm <args>` ➔ Embedded CLI execution.
- `/ping`, `/observe`, `/api`, `/email`, `/backup`, `/restore`, `/ff` ➔ Standard telemetry and actions.

---

## 3. Verification & Acceptance Criteria
1. **Chunking Verification:** Send a test message exceeding 5000 characters; confirm successful sequential delivery without Telegram API errors.
2. **Nodes Command Verification:** Run `/nodes` and verify both local machine and remote cluster nodes appear with correct status badges.
3. **Scoped Prompts Verification:** Run `/nodes local prompts` and verify active workspace prompts are returned.
4. **Prompt Injection Verification:** Run `/prompt local <project_id> "test prompt"` and verify prompt is accepted into `repo_db`.
5. **CLI Parity Verification:** Run `agm telegram nodes` and `agm telegram prompts` locally and confirm matching outputs.
