# Plan 77: Installer Upstream Fork Elimination, Telegram Fleet Nodes & Remote Prompt Injection

## Metadata
- **Status:** Completed
- **Created:** 2026-09-27
- **Target Release:** v4.83.0
- **Authority Spec:** `02-spec/21-app/57-telegram-fleet-nodes-and-prompt-injection.md`
- **Root Cause Analysis:** `02-spec/22-app-issues/20-installer-upstream-fork-inversion-rca.md`

---

## 1. Objectives & Scope
1. **Installer Upstream Fork Elimination:** Completely eradicate all references to `lbjlaq/Antigravity-Manager` across `install.ps1`, `install.sh`, `deploy/arch/install.sh`, and `README_EN.md`. Guarantee 100% of download targets and API queries target `alimtvnetwork/Antigravity-Manager`.
2. **Telegram 4096-Character Chunking:** Implement robust message splitting in `send_telegram_message` / `send_telegram_message_chunked` to prevent `400 Bad Request: message is too long` rejections.
3. **Telegram Cluster VM Nodes Topology (`/nodes`, `/node ls`):** Aggregate GitMap cluster fleet and Supabase Root DB `nodes` table, returning active VM nodes with status badges, IP, uptime, and accounts.
4. **Telegram Scoped Running Prompts (`/nodes <alias> prompts`):** Inspect active projects, recent prompts, and prompt statuses scoped to individual nodes.
5. **Telegram Workspace & Project Registry (`/projects`, `/prompts`):** List running workspaces and prompt IDs so operators can reference them easily.
6. **Telegram Remote Prompt Injection (`/prompt <node> <project> <text>`):** Inject prompts into local `repo_db` or remote Supabase command queues.
7. **Headless CLI Parity:** Support `agm telegram nodes`, `agm telegram prompts`, and `agm telegram prompt`.

---

## 2. Subtask Breakdown
- [x] `01-installer-upstream-elimination.md`: Remove `$UpstreamRepo` and fallback URL rewrites in `install.ps1`, `install.sh`, `deploy/arch/install.sh`, and documentation links in `README_EN.md`.
- [x] `02-telegram-message-chunking.md`: Implement 4000-character line-boundary message chunking with 80ms transmission pacing in `src-tauri/src/modules/telegram_inbound.rs`.
- [x] `03-telegram-cluster-nodes-topology.md`: Implement `/nodes` and `/node ls` aggregating GitMap cluster status and Supabase Root DB nodes.
- [x] `04-telegram-scoped-prompts-and-projects.md`: Implement `/nodes <alias> prompts`, `/projects`, and `/prompts` with compact format and ID snippets.
- [x] `05-telegram-prompt-injection-and-cli-parity.md`: Implement `/prompt <node> <project> <text>` dispatch and CLI parity under `agm telegram`.

