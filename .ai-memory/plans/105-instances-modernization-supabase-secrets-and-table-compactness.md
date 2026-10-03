# Execution Plan: Task 105 - Instances Modernization, Supabase Secrets Auto-Discovery & Account Exclusivity Guard

**Task Identifier**: `105-instances-modernization-supabase-secrets-and-table-compactness`  
**Total Steps Budget**: `N = 300`  
**Subagent Concurrency**: `A = 2, H = 2` (System capacity: 4 concurrent subtask operations)  
**Status**: `ACTIVE (PHASE 2 EXECUTION)`  
**Maintainer**: `@aukgit`

---

## 1. Disjoint File Partitioning (Rule R6)

### Worker 01 Owned Files (Backend Specialist):
- `src-tauri/src/models/config.rs`
- `src-tauri/src/modules/supabase_sync.rs`
- `src-tauri/src/modules/workspace_lease_manager.rs`
- `src-tauri/src/modules/instance.rs`
- `src-tauri/src/modules/auto_switcher.rs`
- `scripts/setup-supabase.ps1`

### Worker 02 Owned Files (Frontend Specialist):
- `src/services/instanceService.ts`
- `src/components/instances/InstanceTable.tsx`
- `src/pages/Instances.tsx`
- `src/pages/Accounts.tsx`
- `src/components/accounts/QuotaItem.tsx`
- `src/pages/Settings.tsx`

---

## 2. Work Breakdown Structure (Wave Plan)

### Wave 1 (Subagent 01 & Subagent 02 Execution):
- **Subtask 105-01 (Worker 01)**: Supabase Secrets Auto-Discovery & Account Exclusivity Engine
  - Auto-discover `d:\work\repo-secrets\02-antigravity-manager\vault\supabase_config.json` in `supabase_sync.rs`.
  - Fix email matching bug in `workspace_lease_manager.rs`.
  - Implement configurable `account_lockout_window_minutes` in `config.rs`.
  - Enforce cross-instance and cross-machine lockout in `auto_switcher.rs` and `instance.rs`.
- **Subtask 105-02 (Worker 02)**: Frontend Table Zero-Scroll Optimization, 4-Col Grid, VS Code Progress Theming & 5-6px Button Radii
  - Merge Profile Name + Bound Account columns in `InstanceTable.tsx`.
  - Truncate data paths (`.../Roaming/Antigravity`) with copy button.
  - Move Prompts button into Actions group and remove separate column.
  - Scale Card View to 4-column compact grid (`xl:grid-cols-4`).
  - Standardize button geometry to 5-6px (`rounded-md`).
  - Fix "Rotate to Next Best" button padding and hover tooltip.
  - Reintroduce middle quota grouping border in `Accounts.tsx` and apply VS Code cyan-teal gradient progress bars.
  - Add Lockout Window dropdown (30m / 60m / 120m) to `Settings.tsx`.

---

## 3. Verification & Release Gates (Phase 3)
1. `npm run build` frontend build verification.
2. `cargo fmt -- --check` Rust formatting check.
3. `scripts/e2e-instance-commands-test.ps1` CLI & instance command E2E verification.
4. `npm run bump minor` (v4.130.0 -> **v4.131.0**).
5. Changelog & README synchronization attributing strictly `@aukgit`.
6. Atomic GitMap commit, push, and pipeline monitoring (`gitmap pe -t`).
