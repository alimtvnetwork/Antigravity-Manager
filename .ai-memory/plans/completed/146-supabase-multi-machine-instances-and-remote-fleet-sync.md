# Master Plan: 146-supabase-multi-machine-instances-and-remote-fleet-sync

## 1. Executive Summary
This milestone implements multi-machine fleet instance visibility in Antigravity-Manager powered by Supabase synchronization. It connects to the Supabase Root DB using credentials auto-discovered from `repo-secrets`, queries all connected fleet nodes (`nodes`), their active profiles (`instance_profiles`), and account locks (`workspace_leases`), enhances heartbeat telemetry with active prompt counts, and renders a sleek, summarized dark-glass table of "Other Machines / Fleet Nodes" in the Instance section beneath the local instances in both Card mode and Table mode. The milestone concludes with a minor version bump and release ceremony.

## 2. Problem Diagnosis & Requirements
1. **Repo-Secrets Pull & Supabase Connection:** Ensure `D:/work/repo-secrets` is pulled, Supabase credentials are auto-discovered and validated, and synchronization is enabled.
2. **Backend Multi-Machine Fleet Query:** Expose a unified Tauri command `get_fleet_machines` that returns all active fleet nodes, their machine alias, IP, active running instances, bound account emails, and running prompt counts.
3. **Heartbeat Prompt Telemetry:** Enhance the background heartbeat in `supabase_sync.rs` so each node pushes its live in-flight prompts count to Supabase.
4. **Instance Section UI Visualization:** In `src/pages/Instances.tsx`, maintain local instances prominently, and render a dedicated `<FleetMachinesTable />` at the bottom (in both Card and Table views) showing remote fleet machines grouped by worker with IP, bound emails, and prompt counts.
5. **Minor Version Bump & Release Ceremony:** Execute pre-flight gates, bump minor version, synchronize manifests, update changelogs, and push atomically.

## 3. Subtask Breakdown
- [x] **Subtask 01:** `01-supabase-credentials-and-fleet-query-backend.md` (Credentials auto-discovery from `repo-secrets`, schema validation, and Root DB multi-machine query)
- [x] **Subtask 02:** `02-heartbeat-prompt-telemetry-and-ipc-registration.md` (Node heartbeat active prompts telemetry, `get_fleet_machines` Tauri command, IPC registration in `lib.rs`)
- [x] **Subtask 03:** `03-fleet-machines-table-ui-component.md` (`FleetMachinesTable.tsx` component, dark-glass table styling, and mounting in `src/pages/Instances.tsx`)
- [x] **Subtask 04:** `04-version-bump-and-release-ceremony.md` (Pre-flight checks, minor version bump, changelog sync, and atomic release via `gitmap cpf`)

## 4. Invariants & Rules
- Minimalist & Contextual UI: Follow segmented dark-glass capsules (`rounded-full`, shared border, dark-glass backdrop) per `AGENTS.md`.
- Zero builds or test runs during routine turns; targeted `cargo fmt`, `cargo clippy`, and `npm run build` only at pre-flight stage.
- Finish with single atomic GitMap commit (`gitmap cpf "<module> - <summary>"`).
