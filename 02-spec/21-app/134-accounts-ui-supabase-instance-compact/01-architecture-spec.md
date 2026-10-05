# UI Architecture & Engineering Specification: Accounts Table Theming, Supabase Multi-Machine Leases, and Instance Compactness

- **Specification Slug:** `134-accounts-ui-supabase-instance-compact`
- **Specification ID:** `02-spec/21-app/134-accounts-ui-supabase-instance-compact/01-architecture-spec.md`
- **Target Release Version:** `v4.154.0` (Minor Bump from `v4.153.0`)
- **Author:** Spec Author 01 (Task 134 Architecture Working Group)
- **Status:** APPROVED & MANDATED

---

## 1. User Request (Verbatim)

> Hi there. So far it looks good in most cases, but there are some issues. For example, when I go into the account section, there was a little bit of lines and it looked like the middle section has some, let's say, grouping or showed nicely. There was a border type thing, and then rows were a little bit showcased before. I really like that you can have this. And also I requested for a different type of coloring for the green ones. You could give me some suggestions or try it out, making the progress bar a little bit better. That's for the home screen account section. I think this is the improvement that you can do. Okay? Now going to the instance section. Also, one more important point, the Supabase is not loaded from the repo secrets folder. Okay? Load the Supabase. Make sure that the same account is not tapped into by multiple instances or multiple account across the other machines as well. Okay? Using the Supabase ensure that, and also the email can be another way of checking. So let's say in Supabase or email, we would check that the current email has been used or taken by any other account in last one hour or 30 minutes, something like this. That should be changeable from the settings. Okay? So based on that, if some email has been taken by some other account or instance, we skip that email. Okay? Unless we have no other email. Okay, so this is how it should be coded. Remember that. And instance section looks nice, but the greener color, I really do not appreciate that much of a green in the table section. It could be a little bit like the VS Code theme. I would appreciate that. You could improve the coloring and also the table that is there. It has a scroll, actually. It does not look good with the scroll. So you could actually reduce some options. For example, the profile name. The profile name and email that can be configured together. Right. There is no need to have two columns for this. Respect that, I believe. Okay, the path for the location, we only wanted to see the ending path rather than the starting path, because the starting path most probably same. So starting part will be dot, dot, dot. The ending part is required. Now, there's a prompts button. Prompts button can be added to the action section. Okay? As with the drop-downs. That would also save some of the spaces. Okay, so improve that table. And also in the header section of the instances, there are buttons which does not look very good. Buttons has too many rounded corners. Usually, the rounded corners would be five to six pixel for the rounded corner, not more than that. And also the rotate to the next best is terrible button. So you should fix that button using padding. Okay? And what is this rotate to next best? Which instance will apply to this? Have no clue. If I hover over, I should be able to see that, but nothing is there. Okay. The card mode is very terrible still. The card mode needs to be more compact. That means at least one row, we need to able to show four items. Okay? So try to do it like this, four items. So compact the option, and also the buttons in the card mode is still broken. The buttons does not look professional. It's been several times that I have requested. I hope you keep the request. So first you are going to include the task, put the images, create the spec, inside the spec, refer to those images, and then you start fixing this one. Okay, and also include the Supabase by the settings by running the CLI and other stuff. Okay. So I have changed the theme, but the audit color looks like very green, which is odd, actually. I think you may need to fix this for me. Yeah, I think this is enough. Please help me in fixing this, and at the end, make a minor bump and release.

---

## 2. Screenshot Evidence & Visual Audit Analysis

The user provided five photographic exhibits illustrating current defects and target design states:

### 2.1 Rename Profile Modal Corner Radius Defect
![Rename Profile Modal](assets/screenshots/134-accounts-ui-01-rename-profile-modal.png)
*Figure 2.1: The rename modal currently applies inconsistent border radii (`rounded-2xl` on modal container vs `rounded-full` / excessive pill radius on the text input). The Save button exhibits cramped text padding. In the background card view, path truncation to ending directory (`...\gitmap-7418\data`) is visible as the preferred format.*

### 2.2 Duplicate / Clone Profile Modal Header and Capsule Radii
![Duplicate Profile Modal](assets/screenshots/134-accounts-ui-02-duplicate-clone-modal.png)
*Figure 2.2: The top navigation header demonstrates the instance switcher `#1 Default` and rotate button. Modal input fields and selection radio cards utilize oversized radii instead of the standardized 5–6px (`rounded-[5px]`) system invariant.*

### 2.3 Theme Catalogue & Quota Progress Bar Neon Green Saturation
![Theme Catalogue and Quota Progress](assets/screenshots/134-accounts-ui-03-theme-catalogue-quota-bars.png)
*Figure 2.3: In dark themes (such as One Dark Pro, Tokyo Night Slate, and Antigravity Dark), the 100% quota progress bars emit an intensely saturated neon green gradient with glowing neon dot indicators. The user explicitly requests replacing this neon green with a modern VS Code cyan/teal theme palette that blends harmoniously into dark and light backgrounds.*

### 2.4 Show All Quotas Header Toggle & Toolbar Alignment
![Show All Quotas Header Toggle](assets/screenshots/134-accounts-ui-04-show-all-quotas-toggle.png)
*Figure 2.4: Toolbar actions (`Focus`, `+`, `Refresh`, `Show All Quotas` toggle, `Import / Export`) show subtle borders and segmented groups. Table rows below lack prominent row divider borders, making row separation difficult to discern.*

### 2.5 Accounts Table Red Box Diagnostic Deficiencies
![Accounts Table Red Boxes](assets/screenshots/134-accounts-ui-05-accounts-table-red-boxes.png)
*Figure 2.5: The user highlighted three critical problem areas with red boxes:*
1. *Box 1 (Top-Left & 4H Model Quota): The harsh neon green progress bar track and checkmark badges. Must be converted to VS Code cyan/teal (`#007acc`, `#0098ff`, `#06b6d4`, `#0d9488`).*
2. *Box 2 (Middle Section - 4H Quota): Lack of structural column grouping. 4H Model Quota and Weekly Quota must form a visually cohesive middle section with subtle tinted background and distinct border boundaries.*
3. *Box 3 (Middle Section - Weekly Quota): Weekly quota progress bars and clock milestones need clear visual boundaries and consistent VS Code color scales.*

---

## 3. High-Level System Architecture

```
+----------------------------------------------------------------------------------------------------+
|                                    ANTIGRAVITY MANAGER DESKTOP                                     |
+----------------------------------------------------------------------------------------------------+
                                                  |
           +--------------------------------------+--------------------------------------+
           |                                                                             |
           v                                                                             v
+------------------------------------+                         +-------------------------------------+
|         ACCOUNTS SUBSYSTEM         |                         |         INSTANCES SUBSYSTEM         |
+------------------------------------+                         +-------------------------------------+
| 1. Table Row Borders               |                         | 1. Horizontal Scroll Removal         |
|    - border-b border-slate-200/80  |                         |    - Fixed compact column budgeting |
|    - border-b dark:border-slate-800|                         | 2. Merged Profile & Email Column    |
| 2. Middle Section Grouping         |                         |    - Top: Profile badge & Seq #     |
|    - [4H Quota | Weekly Quota]     |                         |    - Bottom: Bound email + Tier     |
|    - bg-slate-50/50, border-x      |                         | 3. Path Truncation: "...\<folder>"  |
| 3. VS Code Cyan/Teal Palette       |                         | 4. Prompts in Actions Dropdown      |
|    - Healthy: Teal-500 -> Cyan-500 |                         | 5. Header Rotate to Next Best:      |
|    - Warning: Amber-500            |                         |    - 5-6px radius, hover tooltip    |
|    - Exhausted: Rose-500           |                         | 6. Compact Card Mode (4 cols):      |
| 4. Audit Badge Theming             |                         |    - grid-cols-1 md:2 lg:3 xl:4     |
|    - Cyan/Teal/Slate status badges |                         |    - 2-row structured button grid   |
+------------------------------------+                         +-------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                SUPABASE CLUSTER & COOLDOWN SUBSYSTEM                               |
+----------------------------------------------------------------------------------------------------+
| 1. Repo-Secrets Auto-Discovery:                                                                    |
|    - Search `D:\work\repo-secrets\02-antigravity-and-event-manager\vault\supabase_config.json`      |
|    - Search `D:\work\repo-secrets\02-antigravity-manager\vault\supabase_config.json`               |
|    - Search `D:\work\repo-secrets\03-supabase\01-own\supabase-credentials.json`                    |
|    - Search `D:\work\repo-secrets\03-supabase\02-lovable\supabase-credentials.json`                |
| 2. Cross-Machine Account Lease Lock:                                                               |
|    - Pre-switch lease verification via RPC `acquire_workspace_lease` / REST `workspace_leases`     |
|    - Intercept and reject switch if held by foreign node_id                                        |
| 3. Configurable Email Cooldown Window:                                                             |
|    - Setting: `account_cooldown_minutes` (30m to 60m default, configurable in GUI & CLI)           |
|    - Skip candidate accounts used within cooldown window across any machine or lease              |
|    - Deadlock Prevention Fallback: Use oldest cooldown account if all accounts are cooling down   |
+----------------------------------------------------------------------------------------------------+
```

---

## 4. Subsystem 1: Accounts Table UI & Visual Styling

### 4.1 Table Row Borders and Hover Physics
- **Row Divider Borders:**
  - In `src/components/accounts/AccountTable.tsx` and `src/components/accounts/AccountRow.tsx`, every `<tr>` must have an explicit bottom border:
    `border-b border-slate-200/90 dark:border-slate-800/90`
  - The table container must maintain an inner divider:
    `<tbody className="divide-y divide-slate-200/90 dark:divide-slate-800/90">`
- **Subtle Row Hover Treatment:**
  - Remove harsh black or yellow hover backgrounds.
  - Apply VS Code dark slate hover: `hover:bg-slate-50/90 dark:hover:bg-[#0f273d]/70` with a subtle left accent line `border-l-2 border-l-cyan-500/80`.
  - Active/Current row: `bg-blue-50/70 dark:bg-[#0a2033] border-l-cyan-400 font-semibold`.

### 4.2 Middle Section Column Grouping (4H Model Quota & Weekly Quota)
- **Visual Grouping Strategy:**
  - Group Column 3 (`4H MODEL QUOTA`) and Column 4 (`WEEKLY QUOTA`) into a unified visual cluster.
  - Header styling:
    - Left boundary of group: `border-l-2 border-slate-300 dark:border-cyan-900/60`
    - Right boundary of group: `border-r-2 border-slate-300 dark:border-cyan-900/60`
    - Shared background tint: `bg-slate-100/60 dark:bg-[#091b2c]/80`
    - Header sub-badge indicating: `QUOTA ENVELOPE (ROLLING 4H & WEEKLY)`
  - Body row cell styling:
    - 4H Quota cell: `border-l-2 border-slate-300/80 dark:border-cyan-900/50 bg-slate-50/60 dark:bg-[#081b2b]/50`
    - Weekly Quota cell: `border-r-2 border-slate-300/80 dark:border-cyan-900/50 bg-slate-50/60 dark:bg-[#081b2b]/50`
    - Subtle inner divider between 4H and Weekly: `border-r border-slate-200/40 dark:border-[#15334d]/40`

### 4.3 VS Code Cyan/Teal Progress Bar & Audit Palette
- **Progress Bar Track & Fill (`src/components/accounts/QuotaProgressBar.tsx`):**
  - **Healthy Quota (>= 50%):**
    - Gradient: `bg-gradient-to-r from-teal-500 via-cyan-500 to-[#38bdf8]`
    - Milestone nodes: `bg-cyan-400 dark:bg-cyan-500 border-[1.5px] border-cyan-300 shadow-[0_0_6px_rgba(6,182,212,0.4)]`
    - Percentage text: `text-cyan-700 dark:text-cyan-400`
  - **Warning Quota (25% - 49%):**
    - Gradient: `bg-gradient-to-r from-amber-500 to-amber-400`
    - Milestone nodes: `bg-amber-400 border-[1.5px] border-amber-300`
    - Percentage text: `text-amber-700 dark:text-amber-400`
  - **Critical / Exhausted Quota (< 25%):**
    - Gradient: `bg-gradient-to-r from-rose-500 to-rose-400`
    - Milestone nodes: `bg-rose-500 border-[1.5px] border-rose-300`
    - Percentage text: `text-rose-600 dark:text-rose-400`
- **Audit Badges Palette (`src/pages/Audit.tsx`):**
  - Replace harsh emerald/lime green badges with VS Code cyan/teal theme:
    - Success / Dispatch OK: `bg-cyan-50 text-cyan-700 dark:bg-cyan-950/60 dark:text-cyan-300 border-cyan-200/60 dark:border-cyan-800/60`
    - Positive Action Taken: `bg-teal-50 text-teal-700 dark:bg-teal-950/60 dark:text-teal-300 border-teal-200/60 dark:border-teal-800/60`
    - Idle Check Passed: `bg-cyan-50 text-cyan-800 dark:bg-cyan-950/70 dark:text-cyan-300 border border-cyan-300/70 dark:border-cyan-800` with cyan pulse indicator `bg-cyan-500 animate-pulse`
    - Action Code 1 (Add): `bg-teal-100 text-teal-700 dark:bg-teal-900/40 dark:text-teal-300 border-teal-200/50 dark:border-teal-800/50`

---

## 5. Subsystem 2: Supabase Auto-Discovery, Multi-Machine Leases & Email Cooldown

### 5.1 Comprehensive Repo-Secrets Auto-Discovery Ladder
- **Expanded Candidate Path Inventory (`src-tauri/src/modules/supabase_sync.rs`):**
  Auto-discovery must probe all variations of repo-secrets folders on the host machine:
  1. `D:/work/repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json`
  2. `D:/work/repo-secrets/02-antigravity-manager/vault/supabase_config.json`
  3. `D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json`
  4. `D:/work/repo-secrets/03-supabase/02-lovable/supabase-credentials.json`
  5. `C:/work/repo-secrets/...` (matching paths for C: drive setups)
  6. Environment variable `REPO_SECRETS_DIR` (resolving subfolders `02-antigravity-and-event-manager`, `02-antigravity-manager`, `03-supabase`)
  7. Parent relative fallbacks: `../repo-secrets/...`, `../../repo-secrets/...`, `repo-secrets/...`
  8. User home directory fallback: `dirs::home_dir()/repo-secrets/...`
- **Auto-Seed Trigger Condition:**
  - Invoke `auto_seed_from_repo_secrets(&mut config)` in `load_config()` whenever `endpoints.is_empty()`.
  - Also provide an explicit CLI command `agm supabase load-json` and Settings UI button "Auto-Discover from Repo-Secrets" that forces re-probing even when endpoints already exist, merging missing endpoints without overwriting custom configurations.

### 5.2 Cross-Machine Account Lease Lock
- **Distributed Lock Protocol (`src-tauri/src/modules/workspace_lease_manager.rs`):**
  - **Pre-Switch Validation Gate:**
    - In `src-tauri/src/modules/account.rs` (`switch_account`) and `src-tauri/src/modules/instance.rs` (`switch_instance_account`), perform a mandatory pre-switch lease check before terminating processes or injecting DB state:
      ```rust
      let lease_check = workspace_lease_manager::check_account_lease_conflict(&account_id, &account_email).await;
      if let Err(e) = lease_check {
          return Err(AppError::Conflict(format!(
              "Cannot switch to {}: Currently leased by remote machine '{}' (Profile: '{}', expires in {}s)",
              account_email, e.owner_alias, e.profile_name, e.remaining_secs
          )));
      }
      ```
    - Acquire lease immediately during switch, setting lease TTL derived from `account_cooldown_minutes * 60`.
  - **Foreign Node Collision Guard:**
    - If `workspace_leases` record has `expires_at > now` and `node_id != local_node_id`, access is strictly denied.
    - If `workspace_leases` record has expired (`expires_at <= now`), local node may atomically reclaim the lease.

### 5.3 Configurable Email Cooldown Window
- **Settings Configuration Model:**
  - Model: `AutoProfileSwitcherConfig` in `src-tauri/src/models/config.rs`:
    - Field: `pub account_cooldown_minutes: u32`
    - Default: `60` minutes
    - Permitted Range: `15` to `180` minutes (UI presets: `15m`, `30m`, `45m`, `60m`, `90m`, `120m`)
- **Cooldown Filter Logic in Auto-Switcher (`src-tauri/src/modules/auto_switcher.rs`):**
  - Check whether `(now - account.last_used) < (cooldown_minutes * 60)` OR `(now - lease.leased_at) < (cooldown_minutes * 60)`.
  - **Partitioning Strategy:**
    - Pool A: Non-cooldown candidates (`is_in_cooldown == false`)
    - Pool B: Cooldown candidates (`is_in_cooldown == true`)
  - **Selection Priority:**
    - If Pool A has any eligible accounts with acceptable quota (> threshold), pick the highest health account from Pool A.
    - **Deadlock Prevention Fallback:** If and only if Pool A is empty (all accounts have been used within the cooldown window), select the account from Pool B with the oldest `last_used` timestamp to prevent workflow stalls.

---

## 6. Subsystem 3: Instances Page Table & Card Mode Compacting

### 6.1 Table Horizontal Scroll Elimination
- **Root Cause of Scroll:** Unbounded min-widths on multiple columns (`min-w-[200px]`, `min-w-[220px]`) causing total table width to exceed container width on 1280px–1440px displays.
- **Compact Layout Budget (Total: 100% / max ~1100px):**
  - Column 1: `Profile & Account` (Combined) — `w-[32%] min-w-[220px] max-w-[340px]`
  - Column 2: `Model & Weekly Quota` — `w-[30%] min-w-[200px] max-w-[300px]`
  - Column 3: `Status & PID` — `w-[14%] min-w-[110px] max-w-[140px]`
  - Column 4: `File / Data Path` (Ending Path) — `w-[12%] min-w-[90px] max-w-[130px]`
  - Column 5: `Actions` (Segmented Capsule) — `w-[12%] min-w-[120px] max-w-[140px]`
  - Outer container: `overflow-hidden` with `overflow-x-auto` as fallback, but strictly sized so scrollbar does not trigger at >= 1200px.

### 6.2 Merged Profile Name & Email Column
- **Structure in `InstanceTable.tsx`:**
  - **Top Row:**
    - Sequence badge: `#{seq}` (`px-1.5 py-0.5 rounded-[5px] text-[10px] font-mono font-bold`)
    - Profile Name: Bold truncated display (`font-bold text-xs text-slate-800 dark:text-slate-100`)
    - Status Indicator: Dot pulse (`w-2 h-2 rounded-full`, teal when running, slate when idle)
    - Default badge: `DEFAULT` (amber badge)
    - Active badge / button: `ACTIVE` badge or compact `Set Active` button
  - **Bottom Row:**
    - Mail icon + Masked/Unmasked email (`text-[11px] font-mono text-slate-500`)
    - Tier badge: `PRO` (sky-100) or `ULTRA` (purple-100)
    - Disabled badge (if disabled)

### 6.3 Truncate Location Paths to Ending Folder (`...\<folder>`)
- **Formatting Specification:**
  - Function: `formatEndingFolder(path: string): string`
  - Rule: Extract only the parent and leaf directory, replacing all preceding drive and directory prefixes with `...\`:
    - `D:\work\repo-secrets\02-antigravity-manager` -> `...\02-antigravity-manager`
    - `C:\Users\Admin\AppData\Roaming\Antigravity` -> `...\Roaming\Antigravity`
  - Tooltip: Hovering over the truncated path displays the full absolute path in native HTML title and tooltip.
  - Copy Button: Retain quick-copy button copying the full canonical absolute path.

### 6.4 Prompts Action in Actions Dropdown
- **Consolidation into Actions Capsule:**
  - In `InstanceTable.tsx`, the primary row actions capsule contains:
    1. Launch / Stop / Restart
    2. Switch Account
    3. Fast Forward
    4. More Dropdown (`...`)
  - Inside the "More Dropdown" portal, the top item is:
    - `<Layers className="w-3.5 h-3.5 text-cyan-600 dark:text-cyan-400" /> Prompts & History`
    - Triggers `onOpenPromptTree(inst.config.id)`.

---

## 7. Subsystem 4: Button Corner Radius Normalization & Rotate Button Fix

### 7.1 Global 5–6px Corner Radius Standard
- **Design Token Standard:**
  - Replace all pill classes (`rounded-full`, `rounded-2xl`, `rounded-3xl`, `rounded-xl` on interactive controls) with `rounded-[5px]`.
  - Applies to:
    - Table action capsules and individual buttons
    - Modal input fields and modal action buttons
    - Card mode action buttons
    - Header segmented action groups
  - Preservation: Status indicator pulse dots and circular avatars remain circular (`rounded-full`).

### 7.2 Header "Rotate to Next Best" Button Polish
- **Padding & Typography:**
  - Padding: `px-3 py-1.5` (compact and proportional)
  - Radius: `rounded-[5px]`
  - Background: `bg-blue-600 hover:bg-blue-500 text-white font-semibold text-xs`
- **Contextual Hover Tooltip:**
  - Tooltip content dynamically computes:
    `Target: #<seq> <profile_name> → Next Best Candidate: <email> (<tier> · 4H Quota: <pct>%) · Reason: <rationale>`
  - Hover state immediately informs the user which running instance will be affected and which account will be injected.

---

## 8. Subsystem 5: Instance Card Mode 4-per-Row Compact Grid

### 8.1 4-Column Responsive Grid Layout
- **Container CSS Class:**
  ```tsx
  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
  ```
  - Standard desktop / 1080p display (>= 1280px): Strictly 4 cards per row.
  - Laptop / tablet (768px - 1279px): 2 or 3 cards per row.
  - Mobile (< 768px): 1 card per row.

### 8.2 2-Row Professional Card Action Toolbar
- **Row 1: Primary Lifecycle & Account Actions (Grid: 4 columns):**
  1. Launch / Stop / Restart (Split button when running)
  2. Switch Account (`ArrowRightLeft` icon)
  3. Fast-Forward (`Zap` / `FastForward` icon)
  4. Sync PID & Quota (`Cpu` icon)
- **Row 2: Secondary Tools & Utilities (Grid: 5 columns):**
  1. Prompts (`Layers` icon with cyan pulse badge if task active)
  2. Settings (`SlidersHorizontal` icon)
  3. Clone Profile (`Copy` icon)
  4. Audit Trail (`History` icon)
  5. More Dropdown Popover (`MoreHorizontal` icon for Wipe, Path copy, Delete)
- **Button Styling in Cards:**
  - All buttons use `p-1.5 text-xs rounded-[5px]` with subtle border `border border-slate-300 dark:border-slate-700` and dark slate backgrounds `bg-slate-100 dark:bg-slate-800/80`.

---

## 9. Verification & Quality Gates

| Checkpoint | Scope | Verification Command / Metric | Gate Standard |
|---|---|---|---|
| **V1: Accounts Borders** | `AccountTable.tsx`, `AccountRow.tsx` | Visual inspection | Row borders `border-b` visible, subtle slate hover |
| **V2: Middle Grouping** | `AccountTable.tsx` | DOM & CSS inspection | 4H and Weekly columns share visual boundary |
| **V3: VS Code Palette** | `QuotaProgressBar.tsx`, `Audit.tsx` | Hex/Tailwind audit | Teal/cyan gradients (`#06b6d4`, `#0d9488`), zero neon green |
| **V4: Supabase Discovery** | `supabase_sync.rs` | Auto-seed from `D:\work\repo-secrets` | Discovers own & lovable endpoints, parses tokens |
| **V5: Lease Lock** | `workspace_lease_manager.rs` | Unit / simulation test | Denies switch if leased by foreign node |
| **V6: Email Cooldown** | `auto_switcher.rs` | Filter evaluation test | Skips accounts in 30-60m window unless all cooling down |
| **V7: Instance Table** | `InstanceTable.tsx` | Width measurement | Zero horizontal scrollbar at >= 1200px |
| **V8: Ending Path** | `InstanceTable.tsx` | Render test | Format `...\<folder>`, full path in title |
| **V9: Button Radius** | Global Instances/Accounts UI | Style audit | All buttons strictly `rounded-[5px]` |
| **V10: Rotate Tooltip** | `Instances.tsx` | Tooltip hover check | Shows target profile and next best candidate email |
| **V11: Card Mode Grid** | `Instances.tsx` | Viewport >= 1280px | Strictly 4 cards per row, 2-row button toolbar |
| **V12: Rust Pre-flight** | Rust backend | `cargo fmt -- --check`, `cargo clippy` | Zero errors, zero warnings |
| **V13: Frontend Build** | React frontend | `npm run build` | Clean Vite production bundle compilation |
| **V14: Release Bump** | Version sync | `npm run bump minor` | Bumped to `v4.154.0` |

---

## 10. Non-Negotiable Invariants

1. **NO GIT COMMANDS BY SPEC AUTHOR:** Spec authors must never run `git add`, `git commit`, `git status`, `git diff`, or `git checkout`.
2. **ZERO BREAKING SCHEMA CHANGES:** All JSON configuration envelopes (`supabase_config.json`, `gui_config.json`) must remain backward-compatible with older client versions.
3. **GRACEFUL SUPABASE OFFLINE FALLBACK:** If Supabase is offline or unconfigured, the application must run locally without error, falling back to local SQLite databases.
4. **NO PERMANENT ACCOUNT LOCKOUT:** The email cooldown window must always yield to the deadlock prevention fallback if all available accounts are cooling down.
5. **ATOMIC ATTRIBUTION:** Attribution in release changelogs must strictly attribute `@aukgit` (`(Thanks to @aukgit)`).
