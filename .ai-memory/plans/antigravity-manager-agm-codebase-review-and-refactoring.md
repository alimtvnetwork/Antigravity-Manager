# Master Plan: AGM Codebase Review, E9001 Resolution, Throttling & UI Modernization

**Application:** Antigravity Manager (AGM)  
**Task Slug:** `antigravity-manager-agm-codebase-review-and-refactoring`  
**Target Version:** v4.226.0  
**Spec References:**
- [01-architecture-spec.md](02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/01-architecture-spec.md)
- [02-component-and-ui-spec.md](02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/02-component-and-ui-spec.md)
- [02-component-spec.md](02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/02-component-spec.md)

---

## User Request (Verbatim)

```markdown
You are Spec Author 01 for task antigravity-manager-agm-codebase-review-and-refactoring. You have no prior chat context; this brief is your complete specification.

### Objectives:
Author the architecture specification and backend subtask plan files:
1. 02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/01-architecture-spec.md
2. .ai-memory/plans/antigravity-manager-agm-codebase-review-and-refactoring.md
3. .ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/01-fastforward-sibling-rca-and-fix.md
4. .ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/02-backend-loops-supabase-backoff-and-quota-intervals.md
5. .ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/07-gitmap-agm-jwt-bridge-architecture.md
6. .ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/08-updater-typography-and-terminal-fonts.md

### Core Technical Details to Capture:
- Verbatim capture of incoming prompt under ## User Request (Verbatim) in architecture spec and parent plan.
- Include relative link to uploaded screenshot: ![Screenshot](assets/screenshots/antigravity-manager-agm-codebase-review-and-refactoring-01.png)
- Forensic RCA for Error E9001:
  - Error: "Cannot switch instance to account 'cirajayu11@gmail.com': Account is actively bound to running sibling instance 'Default' on this machine".
  - Root Cause: In `src-tauri/src/modules/auto_switcher/instances.rs` (`is_instance_binding_stale_or_exhausted`), quota exhaustion (`q_4h <= 0.0`) returned `true` without checking `if is_running { return false; }`. When `cirajayu11@gmail.com` had 0% 4h quota on running instance 'Default', it was marked stale/exhausted, omitted from `get_active_in_use_account_ids()`, and offered as a fallback candidate to another instance. The OS check in `switch.rs:59` then halted the switch with E9001.
  - Solution: In `is_instance_binding_stale_or_exhausted`, immediately return `false` if `is_running`. Exclude accounts bound to all running siblings in candidate selection and manual rotation.
- Loop & Scheduler Optimization:
  - Strict >= 30s floor on all background poller loops in `src-tauri/src/bin/agm/green_cmds.rs` (`fpug`, `sug`).
  - Eliminate 5s infinite spin in `repo_db/queue.rs` when workspace directory is missing.
  - Supabase error handling: log warning and enforce 5-minute (300s) backoff before re-checking.
  - Account quota refresh rate limiting: 100% quota -> 5m, <50% -> 3m, <20% -> 2m, untouched accounts -> 5-10m.
- GitMap <-> AGM Secure JWT Bridge:
  - Mutual HMAC-SHA256 JWT authentication over localhost HTTP (`:18188`), short-lived tokens (60s exp), replay nonce cache, scoped permissions (`email:read`, `cluster:sync`).
- Updater Typography:
  - Windows: Consolas 18pt font. Linux: Ubuntu Mono. macOS: SF Mono / Apple system font.

### Invariants:
- Strict relative Git paths ONLY (e.g. `02-spec/...`, `.ai-memory/...`). Ban on absolute paths (`C:\...`) and `file:///` URIs.
- Total ban on git commands.
- Total ban on raw grep. Search exclusively via GitMap.
- Positive booleans only.

Send a completion message with summary when finished.
```

---

## Visual Ground Truth

![Screenshot](assets/screenshots/antigravity-manager-agm-codebase-review-and-refactoring-01.png)

*Figure 1: Reference UI screenshot captured in `assets/screenshots/antigravity-manager-agm-codebase-review-and-refactoring-01.png`.*

---

## Subtask Inventory

### Subtask 01: FastForward Sibling Running Account RCA & Fix (Error E9001)
- **File:** `.ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/01-fastforward-sibling-rca-and-fix.md`
- **Domain:** Backend / Auto Switcher
- **Target Files:**
  - `src-tauri/src/modules/auto_switcher/instances.rs`
  - `src-tauri/src/modules/auto_switcher/selection.rs`
  - `src-tauri/src/modules/auto_switcher/manual.rs`
- **Key Responsibilities:**
  1. Guard `is_instance_binding_stale_or_exhausted` with `if is_running { return false; }`.
  2. Exclude running sibling accounts from `select_candidate_profiles` and `select_and_verify_next_best_profile`.
  3. Aggregate all running sibling instance accounts in `trigger_manual_rotation`.

### Subtask 02: Backend Loops, Supabase Backoff & Quota Refresh Intervals
- **File:** `.ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/02-backend-loops-supabase-backoff-and-quota-intervals.md`
- **Domain:** Backend / CLI / Queue Scheduler / Network Resilience
- **Target Files:**
  - `src-tauri/src/bin/agm/green_cmds.rs`
  - `src-tauri/src/modules/repo_db/queue.rs`
  - `src-tauri/src/modules/supabase_sync/fleet.rs`
  - `src-tauri/src/modules/account/quota_fetch.rs`
- **Key Responsibilities:**
  1. Enforce strict `>= 30s` floor on CLI poller loops (`fpug`, `sug`).
  2. Check workspace existence in `queue.rs` and transition missing workspaces to `'failed'`.
  3. Enforce 300s (5-minute) backoff on Supabase errors and heartbeat failures.
  4. Implement adaptive L1 quota cache TTL (100% -> 5m, <50% -> 3m, <20% -> 2m, untouched -> 5-10m).

### Subtask 03: Instance Card & Account Card Toolbar Redesign & Riseup Yellow 3D
- **File:** `.ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/03-instance-card-toolbar-and-riseup-yellow-3d.md`
- **Domain:** Frontend / React / Card Toolbars
- **Target Files:**
  - `src/components/accounts/AccountCard/AccountCardActions.tsx`
  - `src/pages/instances/InstanceCardActions.tsx`
  - `src/components/accounts/AccountTable/cells/ActionsCell.tsx`
- **Key Responsibilities:**
  1. Replace 4th action icon (Tag) with Fast-Forward (`FastForward`, `>>`).
  2. Relocate Tag custom label button into More menu dropdown (`...` menu) under `MoreMenuPortal`.
  3. Flatten Switch (yellow) and Delete (red) buttons when idle (transparent bg, `opacity-40`, colored on group-hover).
  4. Apply authoritative Riseup Yellow 3D gradient on Switch hover (`from-[#FDE047] via-[#FACC15] to-[#EAB308]`, border `#CA8A04`, shadow `[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.2)]`, active `translate-y-0.5`).

### Subtask 04: UI Hover Contrast & Green Duration Quota Readability
- **File:** `.ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/04-ui-hover-contrast-and-green-quota-readability.md`
- **Domain:** Frontend / UI / Theming / Typography
- **Target Files:**
  - `src/components/accounts/AccountTable/SortableAccountRow.tsx`
  - `src/components/accounts/AccountTable/AccountRowContent.tsx`
  - `src/pages/instances/InstanceCard.tsx`
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/components/accounts/QuotaProgressBar.tsx`
- **Key Responsibilities:**
  1. Replace table row hover `hover:bg-slate-50` with balanced dark-glass opacity (`hover:bg-slate-100/80` light, `dark:hover:bg-slate-800/60` dark).
  2. Remove opaque backgrounds from inner `td` cells, eliminating white flashing on hover.
  3. Upgrade `default-copy` badge (after Pro badge) to high-contrast `bg-indigo-100 text-indigo-950 border-indigo-300 font-black` (light) / `dark:bg-indigo-950 dark:text-indigo-200 dark:border-indigo-500` (dark).
  4. Replace low-contrast `#1af18d` plain text with high-contrast micro-pill badge (`bg-emerald-500/15 text-emerald-950 dark:text-emerald-200 border-emerald-500/30 font-bold`).

### Subtask 05: Boolean Modernization & DRY Enums Centralization
- **File:** `.ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/05-boolean-modernization-and-dry-enums.md`
- **Domain:** Frontend / State Management / Code Quality
- **Target Files:**
  - `src/pages/instances/InstanceCard.tsx`
  - `src/pages/instances/instanceCardData.ts`
  - `src/pages/audit/audit-helpers.tsx`
  - `src/types/audit.ts`
  - `src/types/instance.ts`
- **Key Responsibilities:**
  1. Flatten nested JSX ternary operators and compound multi-condition ifs into extracted helper sub-renderers.
  2. Centralize magic strings into shared enums (`InstanceRunState`, `RotationCondition`, `AuditActionCode`).
  3. Enforce positive boolean variables exclusively (`isRunning`, `hasActiveTask`, `isEmailRevealed`, `isDefaultProfile`).

### Subtask 06: Prompt Parameter Context Menu & Reset Filter Controls
- **File:** `.ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/06-prompt-parameter-context-menu-and-reset-filters.md`
- **Domain:** Frontend / Prompt Tree / Navigation / Controls
- **Target Files:**
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src/pages/instances/InstancesStates.tsx`
  - `src/components/settings/EmailNotifications/SshGitmapSection.tsx`
- **Key Responsibilities:**
  1. Add `onContextMenu` handler to prompt parameter badges (Sequence, Tier, Context) in `DetailHeader.tsx` with copy and filter actions.
  2. Add 1-click Reset / Clear button (`✕`) in `InstancesFilterBar` and `InstancesNoResultsState`.
  3. Add dedicated "Reset to Local" button in remote SSH prompt dispatcher.

### Subtask 07: GitMap <-> AGM Secure JWT Bridge Architecture
- **File:** `.ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/07-gitmap-agm-jwt-bridge-architecture.md`
- **Domain:** Backend / Proxy / Security / IPC
- **Target Files:**
  - `src-tauri/src/proxy/server/`
  - `src-tauri/src/proxy/security.rs`
- **Key Responsibilities:**
  1. Define HMAC-SHA256 mutual JWT authentication over localhost HTTP (`:18188`).
  2. Implement short-lived tokens (60s exp) and replay nonce cache with 120s TTL.
  3. Validate scoped permissions (`email:read`, `cluster:sync`).

### Subtask 08: Updater Typography & Terminal Fonts
- **File:** `.ai-memory/plans/subtasks/antigravity-manager-agm-codebase-review-and-refactoring/08-updater-typography-and-terminal-fonts.md`
- **Domain:** Backend / CLI Updater / OS Integration
- **Target Files:**
  - `src-tauri/src/modules/delegate_updater/run.rs`
- **Key Responsibilities:**
  1. Configure Windows console typography: Consolas 18pt font (`font_family = 54`, `dw_font_size.y = 18`, `font_weight = 600`) with Lucida Console fallback.
  2. Specify Linux Ubuntu Mono and macOS SF Mono terminal parity.
  3. Mask out `DISABLE_NEWLINE_AUTO_RETURN` (0x0008) and enable UTF-8 CodePage 65001.

---

## Execution Waves

| Wave | Primary Objective | Assigned Subtasks | Verification Gate |
|---|---|---|---|
| **Wave 1** | Backend Core Stabilization & E9001 Elimination | Subtask 01, Subtask 02 | `cargo test -p antigravity-tools-lib auto_switcher`<br/>`cargo test -p antigravity-tools-lib repo_db` |
| **Wave 2** | Frontend Modernization, Card Toolbars & Contrast | Subtask 03, Subtask 04, Subtask 05, Subtask 06 | `npm run build`<br/>Visual hover & badge verification |
| **Wave 3** | Security Bridge & System Typography | Subtask 07, Subtask 08 | `cargo check --lib`<br/>`cargo check --bin agm` |
| **Wave 4** | Full Quality Verification & Gate Checks | All Subtasks | `cd src-tauri && cargo fmt -- --check`<br/>`cd src-tauri && cargo clippy --all-targets --all-features`<br/>`npm run build` |

---

## Invariants & Rules

1. **Strict Relative Git Paths:** All file paths must be relative to the repository root. Never use absolute system drive paths or URI schemes.
2. **Positive Booleans Only:** Ensure positive naming conventions across all code and plans (`is_running`, `is_valid`, `is_bound_to_running_sibling`, `is_sync_enabled`).
3. **Total Ban on Git Commands:** No running `git` commands in terminal or scripts.
4. **Total Ban on Raw Grep:** Search codebase exclusively via `gitmap` or file viewing tools.
