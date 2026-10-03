# Plan 106: UI Account & Instance Compacting, VS Code Theming, Supabase Secrets Loading, & Multi-Instance Account Lease Isolation

## Metadata
- **Task ID**: `106-ui-account-instance-compacting-and-supabase-lease`
- **Spec Reference**: `02-spec/21-app/106-ui-account-instance-compacting-and-supabase-lease.md`
- **Total Steps**: 300 (Continuous N-Step Self-Loop)
- **Subagent Concurrency**: A = 2, H = 2

---

## Task Decomposition & Waves

### Wave 0: Ingestion & Discovery
- [x] Ingest and persist user visual evidence to `assets/screenshots/106-*.png`.
- [x] Research Agent 01: Deep inspection of frontend Accounts & Instances styling.
- [x] Research Agent 02: Deep inspection of backend Supabase repo-secrets & lease locking.
- [x] Initialize Task Database (`agent-task.db`) and Task Execution Ledger (`ledger.md`).

### Wave 1: Frontend Accounts Refinement & VS Code Quota Progress Bar
- [x] Subtask 106-01: Author formal Plan and Specification documents (Task-01).
- [x] Subtask 106-02: Account table borders, subtle row grouping, and VS Code navy hover styling (Task-02).
- [x] Subtask 106-03: Quota progress bars VS Code teal/cyan/sky gradient styling and softer badges (Task-03).

### Wave 2: Backend Supabase Secrets, Account Lease Locking & Email Cooldown
- [x] Subtask 106-04: Auto-discovery and loading of `repo-secrets` in `supabase_sync.rs` & `supabase_client.rs` (Task-04).
- [x] Subtask 106-05: Multi-instance and cross-machine account lease lock enforcement in `workspace_lease_manager.rs` and `instance.rs` (Task-05).
- [x] Subtask 106-06: Configurable email cooldown check (30m / 60m) with fallback in `auto_switcher.rs` & `config.rs` (Task-06).
- [x] Subtask 106-07: CLI parity for Supabase commands in `bin/agm.rs` & `modules/cli.rs` (Task-15).

### Wave 3: Frontend Instance Table Consolidation
- [x] Subtask 106-08: Eliminate table horizontal scroll in `src/components/instances/InstanceTable.tsx` (Task-07).
- [x] Subtask 106-09: Combine Profile Name and Email into single compact column (Task-08).
- [x] Subtask 106-10: Display ending path (`formatEndingPath`) in table & cards (Task-09).
- [x] Subtask 106-11: Integrate Prompts button into Actions dropdown/capsule (Task-10).

### Wave 4: Frontend Instance Card Grid & Button Styling Standard
- [x] Subtask 106-12: Enforce 5-6px border radius (`rounded-[5px]`) across buttons (Task-11).
- [x] Subtask 106-13: "Rotate to Next Best" padding, size, and hover tooltip (Task-12).
- [x] Subtask 106-14: Compact Card Mode 4 items per row (`xl:grid-cols-4`) in `src/pages/Instances.tsx` (Task-13).
- [x] Subtask 106-15: 2-Row clean professional card action toolbar (Task-14).
- [x] Subtask 106-16: Soften Audit button color to VS Code slate theme (Task-16).

### Wave 5: Preflight Checks, Minor Bump & Release
- [ ] Subtask 106-17: Rust fmt & clippy, frontend build, minor bump (`npm run bump minor`), changelogs (@aukgit attribution), and release (Task-17).
