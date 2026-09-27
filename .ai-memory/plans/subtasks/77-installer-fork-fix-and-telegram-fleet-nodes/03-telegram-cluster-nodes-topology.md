# Subtask 03: Telegram Cluster VM Nodes Topology (`/nodes`, `/node ls`)

## Metadata
- **Parent Plan:** `77-installer-fork-fix-and-telegram-fleet-nodes.md`
- **Status:** Completed
- **Target Files:**
  - `src-tauri/src/modules/telegram_inbound.rs`
  - `src-tauri/src/bin/agm.rs`

---

## 1. Description
Provide operators with a unified view of all cluster VM nodes:
1. Aggregate GitMap cluster status (`gitmap cluster status`) and Supabase Root DB `nodes` table.
2. Fallback to local node telemetry if external cluster tools or Supabase are unconfigured.
3. Handle `/nodes`, `/nodes ls`, and `/node ls` in `process_telegram_command_text`.
4. Render compact status table: Alias, IP, Status badge (`🟢 ONLINE`, `⚪ IDLE`, `🔴 OFFLINE`), Uptime, and Active account / lease.

---

## 2. Verification Criteria
- [x] `/nodes` command returns formatted list with local machine and connected VM nodes.
