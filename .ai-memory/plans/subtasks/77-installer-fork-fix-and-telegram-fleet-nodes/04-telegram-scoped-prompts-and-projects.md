# Subtask 04: Telegram Scoped Prompts & Projects Catalog

## Metadata
- **Parent Plan:** `77-installer-fork-fix-and-telegram-fleet-nodes.md`
- **Status:** Completed
- **Target Files:**
  - `src-tauri/src/modules/telegram_inbound.rs`

---

## 1. Description
Allow operators to query prompts scoped to specific nodes and list active projects:
1. Handle `/nodes <alias> prompts` and `/node <alias> prompts`:
   - Inspect active running prompts and workspace status on the designated node.
2. Handle `/projects` and `/workspaces`:
   - List active workspace paths, project IDs, and status.
3. Handle `/prompts` and `/prompts ls`:
   - List recent prompts in a compact table with prompt ID snippets.

---

## 2. Verification Criteria
- [x] `/projects` lists active projects with IDs.
- [x] `/nodes local prompts` displays running prompts.
