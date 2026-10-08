# Master Audit Ledger: 146-supabase-multi-machine-instances-and-remote-fleet-sync

**Task Slug:** `146-supabase-multi-machine-instances-and-remote-fleet-sync`  
**Parent Plan:** `.ai-memory/plans/pending/146-supabase-multi-machine-instances-and-remote-fleet-sync.md`  
**Status:** `IN_PROGRESS` (Phase 1: Spec Authoring)  
**Creation Time:** 2026-10-08T19:31:00+08:00  

---

## 1. User Request (Verbatim)

```markdown
In the Antigravity Manager, in the instance section, if we have the email or if we have the Superbase connected to the same one, we should be able to see all the machines using the database that what are the other machines are using as the accounts, and it should have all this information. You need to confirm that this is already implemented if the Superbase is already there. And I want you to connect to the Superbase, and you get the Superbase information in the repo secrets. So do a repo pull. I mean, get pull on the repo secrets and try to connect the Superbase credential to this, Antigravity Manager. That's the first thing. Second is that in the instance section, it should show the current instance nicely, and then afterwards, it would show the other machines in a short way. Because in the other machines, we are not going to open the instance, but it should show as a table at the end. Even in the card mode, it would show as a table by grouping like worker one, worker two with the IP and whatever the instance is running. And on those instance, what are the emails are connected and how many prompts are running, if that is possible, because prompts should be able to possible because it should be sending some of the details. We shouldn't be able to opening the prompt, at least for now. Make sure that these are the codes are already there, and that if not, then we can implement it, the UI visualization, and at the end, we can bump the minor version and make a release. Do you understand the requirements? Can you please work on it and make sure that you release it properly?
```

---

## 2. Technical Scope & Architecture Decisions

1. **Repo-Secrets & Supabase Auto-Discovery:**
   - Verified `repo-secrets` in `D:/work/repo-secrets` pulled and up-to-date.
   - Credentials located at `D:/work/repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json`, `03-supabase/01-own/supabase-credentials.json`, etc.
   - Ensured `auto_discover_and_enable_supabase_sync()` connects, validates root endpoint, and populates `supabase_config.json`.

2. **Backend Fleet Machinery (`src-tauri/src/commands/supabase.rs` & `src-tauri/src/modules/supabase_sync.rs`):**
   - New Tauri command `get_fleet_machines` returning `Vec<FleetMachineInfo>`.
   - Aggregate `nodes`, `instance_profiles`, and `workspace_leases`.
   - Enhance node heartbeat payload to include active prompt count (`in_flight_prompts_count`).
   - Register `get_fleet_machines` in `src-tauri/src/lib.rs` and expose in `src/services/supabaseService.ts`.

3. **Frontend UI Visualization (`src/components/instances/FleetMachinesTable.tsx` & `src/pages/Instances.tsx`):**
   - In `src/pages/Instances.tsx`, maintain local instances in both Card mode and Table mode.
   - Below the local instances, mount `<FleetMachinesTable />`.
   - Compact dark-glass table grouped by machine name (e.g. `worker-1`, `worker-2`) showing:
     * Machine Name & Online pulse dot
     * Monospace IP Address with copy action
     * Active Running Instance(s)
     * Bound Account Email(s) (masked with click-to-unmask)
     * Running Prompts count (`2 running` or `Idle`)
     * Uptime & Last Heartbeat relative timestamp
   - Read-only table; no prompt opening or remote instance launching.

4. **Minor Version Bump & Release:**
   - Execute pre-flight gates (`cargo fmt`, `cargo clippy`, `npm run build`).
   - Run minor release workflow via `npm run bump minor` synchronizing manifests (`package.json`, `tauri.conf.json`, `Cargo.toml`, `version.json`, `releases-manifest.json`, `changelog.md`, `changelog_en.md`, `README.md`, `README_EN.md`).
   - Final atomic release commit via `gitmap cpf`.

---

## 3. Subtask Traceability Matrix

| Subtask Code | File | Owner | Description | Status |
| :--- | :--- | :--- | :--- | :--- |
| `Subtask-01` | `.ai-memory/plans/subtasks/146-supabase-multi-machine-instances-and-remote-fleet-sync/01-supabase-credentials-and-fleet-query-backend.md` | Author 01 | Supabase credentials discovery, Root DB aggregation query, and `FleetMachineInfo` struct | `PENDING` |
| `Subtask-02` | `.ai-memory/plans/subtasks/146-supabase-multi-machine-instances-and-remote-fleet-sync/02-heartbeat-prompt-telemetry-and-ipc-registration.md` | Author 01 | Heartbeat prompt count telemetry, `get_fleet_machines` Tauri command & IPC registration | `PENDING` |
| `Subtask-03` | `.ai-memory/plans/subtasks/146-supabase-multi-machine-instances-and-remote-fleet-sync/03-fleet-machines-table-ui-component.md` | Author 02 | `FleetMachinesTable.tsx` component, dark-glass table styling, and mounting in `Instances.tsx` | `PENDING` |
| `Subtask-04` | `.ai-memory/plans/subtasks/146-supabase-multi-machine-instances-and-remote-fleet-sync/04-version-bump-and-release-ceremony.md` | Author 02 | Minor version bump, manifest sync, changelog update (`@aukgit`), and atomic push | `PENDING` |
