# Plan: 02-supabase-fleet-machines-instances-sync

## Specification References
- Architecture Spec: [01-architecture-spec.md](../../02-spec/21-app/02-supabase-fleet-machines-instances-sync/01-architecture-spec.md)
- Component Spec: [02-component-spec.md](../../02-spec/21-app/02-supabase-fleet-machines-instances-sync/02-component-spec.md)

## Subtasks
- Subtask 1: [01-backend-supabase-fleet-and-ipc.md](subtasks/02-supabase-fleet-machines-instances-sync/01-backend-supabase-fleet-and-ipc.md)
- Subtask 2: [02-frontend-fleet-table-and-instances.md](subtasks/02-supabase-fleet-machines-instances-sync/02-frontend-fleet-table-and-instances.md)

## User Request (Verbatim)

```text
# High Priority Instruction

In the Antigravity Manager, in the instance section, if we have the email or if we have the Superbase connected to the same one, we should be able to see all the machines using the database that what are the other machines are using as the accounts, and it should have all this information. You need to confirm that this is already implemented if the Superbase is already there. And I want you to connect to the Superbase, and you get the Superbase information in the repo secrets. So do a repo pull. I mean, get pull on the repo secrets and try to connect the Superbase credential to this, Antigravity Manager. That's the first thing. Second is that in the instance section, it should show the current instance nicely, and then afterwards, it would show the other machines in a short way. Because in the other machines, we are not going to open the instance, but it should show as a table at the end. Even in the card mode, it would show as a table by grouping like worker one, worker two with the IP and whatever the instance is running. And on those instance, what are the emails are connected and how many prompts are running, if that is possible, because prompts should be able to possible because it should be sending some of the details. We shouldn't be able to opening the prompt, at least for now. Make sure that these are the codes are already there, and that if not, then we can implement it, the UI visualization, and at the end, we can bump the minor version and make a release. Do you understand the requirements? Can you please work on it and make sure that you release it properly?
```

## Work Breakdown & Phased Execution

### Phase 1: Planning, Research & Specifications
- [x] Step 1: Pre-flight checks, GitMap task DB initialization, `repo-secrets` git pull.
- [x] Step 2: Spawn Research 01 (Backend) and Research 02 (Frontend) subagents.
- [ ] Step 3: Spawn Spec Author 01 and Spec Author 02 to generate specifications.

### Phase 2: Parallel Implementation
- [ ] Worker 01 (Subtask 1):
  - Add `running_prompts_count INT NOT NULL DEFAULT 0` to `src-tauri/src/modules/supabase_schema.rs`.
  - In `src-tauri/src/modules/supabase_sync.rs`, calculate running prompt counts from `repo_db` during heartbeat sync.
  - Implement `fetch_fleet_machines() -> AppResult<Vec<FleetMachineInfo>>` aggregating `nodes`, `instance_profiles`, and `workspace_leases`.
  - Add Tauri IPC commands `get_supabase_fleet_machines` and `sync_supabase_now` in `src-tauri/src/commands/supabase.rs` and register in `src-tauri/src/lib.rs`.
- [ ] Worker 02 (Subtask 2):
  - In `src/services/supabaseService.ts`, define `FleetMachineInfo` and `FleetInstanceItem` interfaces, and add `getFleetMachines()` and `syncNow()` methods.
  - Implement `src/components/instances/FleetMachinesTable.tsx` with worker grouping, IP pills, instance status, masked email badges, and running prompt counters.
  - In `src/pages/Instances.tsx`, mount `<FleetMachinesTable />` at the bottom of the page so it is visible in both Card Mode and Table Mode, with 3s polling.

### Phase 3: Verification, Pre-flight Checks & Minor Release
- [ ] Targeted tests: `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, `npm run build`.
- [ ] Atomic minor version bump: `npm run bump minor`.
- [ ] Sync changelogs (`CHANGELOG.md`, `CHANGELOG_EN.md`) and READMEs (`README.md`, `README_EN.md`) with strictly `@aukgit` attribution.
- [ ] Commit, tag, and push release.
