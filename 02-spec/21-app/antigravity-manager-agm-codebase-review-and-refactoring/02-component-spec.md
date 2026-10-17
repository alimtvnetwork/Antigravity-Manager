# Component Specification: AGM Codebase Review, E9001 Resolution, Throttling & UI Modernization

**Application:** Antigravity Manager (AGM)
**Version:** v4.225.0
**Target Release:** v4.226.0

---

## 1. Component Interface Changes

### 1.1 Backend: Auto-Switcher & Instance Binding (`instances.rs`, `selection.rs`, `manual.rs`)
- `is_instance_binding_stale_or_exhausted(inst, is_running, stale_timeout_hours, target_model, now_sec) -> bool`:
  - Enforce immediate return `if is_running { return false; }`.
- `get_active_in_use_account_ids() -> Vec<String>`:
  - Collects accounts from all instances that are running OR active non-stale.
- `manual::trigger_manual_rotation_for_instance(target_instance_id: Option<&str>) -> Result<String, String>`:
  - Collects all accounts bound or actively selected by running sibling instances, checking both UUID and email, including default instance active account.
- `selection::select_candidate_profiles(...) -> Result<Vec<ProfileCandidate>, String>`:
  - Enforces running sibling instance account isolation in Section 1 and Section 2.
- `selection::select_and_verify_next_best_profile(...) -> Result<Option<ProfileCandidate>, String>`:
  - Live pre-switch verification strictly rejects any candidate bound to an actively running sibling instance on this host.

### 1.2 Backend: Queue Scheduler & Backoff (`queue.rs`, `quota_fetch.rs`, `fleet.rs`)
- `queue::check_and_dispatch_enqueued_prompts(target_instance: Option<&str>) -> Result<usize, String>`:
  - Validates `Path::new(&prompt.repo_path).exists()`.
  - If workspace directory does not exist, logs warning and marks prompt `status = 'failed'` in `active_prompts`, removing it from the active FIFO loop.
  - On `spawn_prompt_via_agy` error, updates `updated_at = now` so prompt backoff is respected.
- `quota_fetch::store_l1_quota_cache(account_id: &str, quota_data: &QuotaData, now_sec: i64)`:
  - Computes adaptive TTL based on quota level:
    - `percentage < 20.0`: 120s (2m)
    - `percentage < 50.0`: 180s (3m)
    - `percentage >= 50.0`: 300s (5m)
- `fleet::start_sync_worker()`:
  - Incorporates 300s (5-minute) error backoff when Supabase synchronization encounters network or HTTP failures.

### 1.3 Frontend: Accounts Table Styling (`SortableAccountRow.tsx`, `AccountRowContent.tsx`, `ActionsCell.tsx`)
- `SortableAccountRow.tsx`:
  - Replaces `hover:!bg-white dark:hover:!bg-white hover:!text-black dark:hover:!text-black` with:
    `hover:bg-slate-100/70 dark:hover:bg-slate-800/50`
  - Replaces drag handle and checkbox cell `group-hover:!bg-white` with transparent / subtle glass style.
- `AccountRowContent.tsx`:
  - Removes all `group-hover:!bg-white` and `group-hover:!text-black`.
  - Maintains `Default Copy` badge contrast with crisp dark border (`border-indigo-300 dark:border-indigo-700`).
- `ActionsCell.tsx`:
  - Reorders action buttons:
    1. Fast-Forward button (`FastForward`, `>>`)
    2. Refresh button (`RefreshCw`, `↻`)
    3. Switch button (`ArrowRightLeft`, ⇄)
    4. Delete button (`Trash2`, 🗑️)
    5. More button (`MoreHorizontal`, ···)
  - Moves Tag button (`Tag`) inside the More menu dropdown.
  - Renders Switch button in Riseup yellow gradient with 3D button effect:
    `bg-gradient-to-b from-[#FDE047] via-[#FACC15] to-[#EAB308] text-slate-950 font-bold border border-[#CA8A04] shadow-[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.2)] active:translate-y-0.5 active:shadow-[0_0_0_0_#A16207]`
  - Flatten Switch and Delete buttons when row is idle; highlight on row hover.

### 1.4 Frontend: Prompt Tree Filter Reset (`TreePanel.tsx`)
- Renders an interactive "Reset Filters" button when `searchQuery`, `activeFilter !== 'all'`, `categoryFilter !== 'all'`, or `selectedProjectScope !== 'all'`.
- Reset action sets `searchQuery = ''`, `activeFilter = 'all'`, `categoryFilter = 'all'`, `selectedProjectScope = 'all'`.

### 1.5 System: Terminal Updater Font (`delegate_updater/run.rs`)
- Win32 Console configuration sets `font_family = 54` (`FF_MODERN | FIXED_PITCH`), `dw_font_size.y = 18`, `font_weight = 600`, with Consolas / Lucida Console fallback.

---

## 2. File Ownership Matrix

| File Path | Responsible Worker | Primary Edits |
|---|---|---|
| `src-tauri/src/modules/auto_switcher/instances.rs` | Worker 01 | Running instance protection in `is_instance_binding_stale_or_exhausted` |
| `src-tauri/src/modules/auto_switcher/manual.rs` | Worker 01 | Collect all accounts on running sibling instances into `excluded` |
| `src-tauri/src/modules/auto_switcher/selection.rs` | Worker 01 | Running sibling instance account check in Section 1 and Section 2 |
| `src-tauri/src/modules/repo_db/queue.rs` | Worker 01 | Non-existent repo directory check, mark failed, and dispatch backoff |
| `src-tauri/src/modules/account/quota_fetch.rs` | Worker 01 | Adaptive L1 cache TTL (2m / 3m / 5m) |
| `src-tauri/src/modules/supabase_sync/fleet.rs` | Worker 01 | Supabase 5-minute error backoff |
| `src/components/accounts/AccountTable/SortableAccountRow.tsx` | Worker 02 | Remove blinding white hover, apply dark-glass hover |
| `src/components/accounts/AccountTable/AccountRowContent.tsx` | Worker 02 | Remove blinding white cell hover, ensure badge contrast |
| `src/components/accounts/AccountTable/cells/ActionsCell.tsx` | Worker 02 | Reorder buttons (FF, Refresh, 3D Switch, Delete, More), 3D yellow gradient, flatten idle |
| `src/components/instances/PromptTreeView/TreePanel.tsx` | Worker 02 | Working Reset Filters button |
| `src-tauri/src/modules/delegate_updater/run.rs` | Worker 02 | Console font configuration (Consolas 18pt, font_family=54) |
