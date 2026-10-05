# Execution Plan: 130-accounts-ui-supabase-sync-instance-compact-and-release

## User Request (Verbatim)

> Hi there. So far it looks good in most cases, but there are some issues. For example, when I go into the account section, there was a little bit of lines and it looked like the middle section has some, let's say, grouping or showed nicely. There was a border type thing, and then rows were a little bit showcased before. I really like that you can have this. And also I requested for a different type of coloring for the green ones. You could give me some suggestions or try it out, making the progress bar a little bit better. That's for the home screen account section. I think this is the improvement that you can do. Okay? Now going to the instance section. Also, one more important point, the Supabase is not loaded from the repo secrets folder. Okay? Load the Supabase. Make sure that the same account is not tapped into by multiple instances or multiple account across the other machines as well. Okay? Using the Supabase ensure that, and also the email can be another way of checking. So let's say in Supabase or email, we would check that the current email has been used or taken by any other account in last one hour or 30 minutes, something like this. That should be changeable from the settings. Okay? So based on that, if some email has been taken by some other account or instance, we skip that email. Okay? Unless we have no other email. Okay, so this is how it should be coded. Remember that. And instance section looks nice, but the greener color, I really do not appreciate that much of a green in the table section. It could be a little bit like the VS Code theme. I would appreciate that. You could improve the coloring and also the table that is there. Look at the screenshot in the table section. You have to scroll to the right. We do not want that. For example, the profile name and the bound email can be put together in one section. Just look at the size of the table. You don't have to spread things around. And also the folder, only the ending path is enough, like which project is that. So that would be enough. Also, we had that button, the prompt button. It's not necessarily needed to be separately there. It can be merged in the capsule button that is there. And the other buttons like the card format, you can have 4 items per row. The buttons look bad. In card mode, you can have 2 rows of buttons instead of 1 row, and with the rounded 5 or 6px. Even in the card format and everywhere, the rounded border of the buttons should be 5 to 6px. Also the Rotate to Next Best button: make sure that the button has proper padding. If there's an instance selected, it should rotate to the next best for that instance. If no instance selected, on hover tell the user which instance it will rotate to (or the active/default instance). And release: minor bump and release ceremony at the end.

---

## Visual Reference Assets

- `![Theme Catalogue and Quota Progress](assets/screenshots/130-accounts-ui-01.png)`
- `![Show All Quotas Header Toggle](assets/screenshots/130-accounts-ui-02.png)`
- `![Accounts Table and Quota Bars](assets/screenshots/130-accounts-ui-03.png)`
- `![Duplicate Profile Modal](assets/screenshots/130-accounts-ui-04.png)`

---

## Core Architectural Objectives

### 1. Accounts Section UI Modernization (Task-01)
- **QuotaProgressBar & WaterDrainProgressBar Color Palette**:
  - Replace harsh neon green hex codes (`#1af18d`, `#10b981`, `#059669`) with a calming, professional VS Code teal/cyan/sky palette (`from-teal-500 via-cyan-500 to-sky-500`).
  - Soften milestone checkpoint nodes: eliminate harsh neon glow (`shadow-[0_0_8px_rgba(26,241,141,0.6)]`), replacing it with a soft cyan node (`bg-cyan-500 border-cyan-400 shadow-[0_0_6px_rgba(6,182,212,0.4)]`).
- **AccountTable & AccountRow Border Lines & Middle Section Grouping**:
  - Restore crisp row border lines (`border-b border-slate-200/80 dark:border-slate-800/80`).
  - Re-introduce subtle visual grouping in the middle section (Gemini and Claude 4H and Weekly quota bars) so rows and metrics are cleanly showcased without blending together (`bg-slate-50/30 dark:bg-slate-900/20`).
  - Soften audit and status badges (`DISABLED`, `FORBIDDEN`, `PROXY_DISABLED`, `VALIDATION_BLOCKED`, running indicators) to muted VS Code slate/teal/cyan styling with 5–6px rounded corners (`rounded-[5px]`).

### 2. Supabase Auto-Discovery from Repo-Secrets (Task-02)
- Auto-discover root database credentials from candidate paths including `D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json`, `D:/work/repo-secrets/03-supabase/02-lovable/supabase-credentials.json`, `D:/work/repo-secrets/02-antigravity-manager/vault/supabase_config.json`, and `REPO_SECRETS_DIR`.
- Auto-seed endpoint configurations when `supabase_config.json` is empty, setting `is_sync_enabled = true`.
- Normalize URLs (strip trailing slashes, enforce valid PostgREST hostnames) and ensure zero duplicate endpoints.

### 3. Cross-Machine Account Locks & Configurable Email Cooldown Window (Task-03)
- Prevent multiple instances from accessing the same account across physical and virtual machines using Supabase distributed leases (`workspace_leases` table).
- Enforce atomic lease acquisition via `acquire_workspace_lease` RPC or PostgREST upsert with TTL derived from app cooldown.
- Add configurable 30–60 minute email cooldown window (`account_cooldown_minutes: u32`) to `AutoProfileSwitcherConfig`, surfaced in `AutoSwitcherSettings.tsx`.
- Auto-switcher candidate evaluation:
  - If an account or email was used or leased within the cooldown window, skip it in favor of available accounts.
  - Graceful fallback: If all available accounts are in cooldown, automatically select the oldest cooling account so operations never dead-end.

### 4. Instances Table Compacting & Horizontal Scrollbar Removal (Task-04)
- Compact column widths and padding to guarantee zero horizontal scroll on standard 1280px+ desktop viewports.
- Merge Profile Name and Bound Email into a single consolidated cell with stacked hierarchy.
- Path truncation: enforce `formatShortPath(fullPath)` (`.../folder` or `...\<parent>\<leaf>`, max-w 140px) with clipboard copy action and full path hover tooltip.
- Merge the standalone "Prompts" button into the segmented action capsule alongside Launch/Stop, Switch, Fast-Forward, Audit, and Sync.
- Modernize emerald green accents (launch spinner, active dots) to cyan/teal.

### 5. Instances Card View & Universal Button Radius Standards (Task-05)
- Standardize button border radii strictly to 5–6px (`rounded-[5px]`).
- Card grid layout: enforce strictly 4 cards per row (`grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4`) for standard card view.
- Action buttons in card mode: structured into 2 clean, balanced rows:
  - **Row 1 (Primary Lifecycle - 5 buttons)**: Launch/Stop, Switch Account, Fast-Forward, Audit Trail, Sync PID & Quota.
  - **Row 2 (Utilities & Danger - 6 buttons)**: Prompts, Settings & Sync, Clone Profile, Clone Executable, Wipe Credentials, Delete Profile.
- "Rotate to Next Best" Header Button:
  - Balanced padding: `px-3 py-1.5 text-xs font-semibold rounded-[5px]`.
  - Dynamic target hover tooltip: explicitly informs the user which instance will be rotated (active target or default profile) and health scoring rationale.

### 6. Verification & Minor Version Bump Release Ceremony (Task-06)
- Pre-flight quality verification: Rust `cargo clippy`, `cargo fmt`, and frontend `npm run build`.
- Atomic minor version bump: `npm run bump minor`.
- Attribution invariant: Strictly attribute `@aukgit` in changelog files.
- Tag and release synchronization across release channels.

---

## Requirements Traceability Matrix

| Requirement / Prompt Item | Canonical Spec File (`02-spec/21-app/`) | Subtask File (`.ai-memory/plans/subtasks/`) | Target Code Files |
|:---|:---|:---|:---|
| Accounts: Table borders, grouping, cyan/teal palette, softened badges | [02-spec/21-app/130-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md](../../02-spec/21-app/130-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md) | `01-accounts-table-borders-grouping-and-cyan-palette.md` | `src/components/accounts/QuotaProgressBar.tsx`, `WaterDrainProgressBar.tsx`, `AccountTable.tsx`, `AccountRow.tsx` |
| Supabase: repo-secrets autoload, cross-machine leases, 30-60m email cooldown | [02-spec/21-app/130-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md](../../02-spec/21-app/130-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md) | `02-supabase-secrets-auto-discovery-and-email-cooldown.md` | `src-tauri/src/modules/supabase_sync.rs`, `workspace_lease_manager.rs`, `auto_switcher.rs`, `models/config.rs`, `AutoSwitcherSettings.tsx` |
| Instances Table: Zero scroll, merged profile/email, path truncation, capsule prompts | [02-spec/21-app/130-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md](../../02-spec/21-app/130-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md) | `03-instances-table-compact-and-prompts-capsule.md` | `src/components/instances/InstanceTable.tsx` |
| Instances Cards & Buttons: 4 cards/row, 2-row buttons, 5-6px radius, Rotate button tooltip | [02-spec/21-app/130-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md](../../02-spec/21-app/130-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md) | `04-instance-cards-four-cols-and-button-radius.md` | `src/pages/Instances.tsx` |
| Verification & Release: Minor bump ceremony, changelog attribution to @aukgit | [02-spec/21-app/130-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md](../../02-spec/21-app/130-accounts-ui-supabase-sync-instance-compact-and-release/01-architecture-spec.md) | `05-verification-and-minor-release-bump.md` | `package.json`, `src-tauri/Cargo.toml`, `changelog.md`, `README.md` |

---

## Multi-Agent Execution Waves

- **Wave 1: Specification Authoring & Subtask Planning (Current Wave)**
  - Spec Author 01: Canonical Architecture Spec (`01-architecture-spec.md`) + Master Plan + Subtasks 01 & 02.
  - Spec Author 02: Subtasks 03, 04, and 05.
- **Wave 2: Parallel Implementation**
  - Worker 01 (Frontend UI): Execute Subtasks 01, 03, and 04 (Accounts UI, Instance Table compacting, Cards 4-col layout, 5–6px button standards).
  - Worker 02 (Backend Rust & Settings): Execute Subtask 02 (Supabase auto-discovery, distributed leases, email cooldown window in `auto_switcher.rs` and `AutoSwitcherSettings.tsx`).
- **Wave 3: Consolidation, Quality Gates & Release Ceremony**
  - Release Lead: Run pre-flight checks (`cargo clippy`, `cargo fmt`, `npm run build`), execute `npm run bump minor`, verify changelog attribution strictly to `@aukgit`, and finalize release.
