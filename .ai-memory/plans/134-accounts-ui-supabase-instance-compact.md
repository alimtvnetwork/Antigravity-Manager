# Master Execution Plan: 134-accounts-ui-supabase-instance-compact

- **Plan Slug:** `134-accounts-ui-supabase-instance-compact`
- **Plan File:** `.ai-memory/plans/134-accounts-ui-supabase-instance-compact.md`
- **Canonical Architecture Spec:** `02-spec/21-app/134-accounts-ui-supabase-instance-compact/01-architecture-spec.md`
- **Target Release Version:** `v4.154.0` (Minor Bump from `v4.153.0`)
- **Author:** Spec Author 01 (Task 134 Working Group)
- **Status:** APPROVED & ACTIVE

---

## 1. User Request (Verbatim)

> Hi there. So far it looks good in most cases, but there are some issues. For example, when I go into the account section, there was a little bit of lines and it looked like the middle section has some, let's say, grouping or showed nicely. There was a border type thing, and then rows were a little bit showcased before. I really like that you can have this. And also I requested for a different type of coloring for the green ones. You could give me some suggestions or try it out, making the progress bar a little bit better. That's for the home screen account section. I think this is the improvement that you can do. Okay? Now going to the instance section. Also, one more important point, the Supabase is not loaded from the repo secrets folder. Okay? Load the Supabase. Make sure that the same account is not tapped into by multiple instances or multiple account across the other machines as well. Okay? Using the Supabase ensure that, and also the email can be another way of checking. So let's say in Supabase or email, we would check that the current email has been used or taken by any other account in last one hour or 30 minutes, something like this. That should be changeable from the settings. Okay? So based on that, if some email has been taken by some other account or instance, we skip that email. Okay? Unless we have no other email. Okay, so this is how it should be coded. Remember that. And instance section looks nice, but the greener color, I really do not appreciate that much of a green in the table section. It could be a little bit like the VS Code theme. I would appreciate that. You could improve the coloring and also the table that is there. It has a scroll, actually. It does not look good with the scroll. So you could actually reduce some options. For example, the profile name. The profile name and email that can be configured together. Right. There is no need to have two columns for this. Respect that, I believe. Okay, the path for the location, we only wanted to see the ending path rather than the starting path, because the starting path most probably same. So starting part will be dot, dot, dot. The ending part is required. Now, there's a prompts button. Prompts button can be added to the action section. Okay? As with the drop-downs. That would also save some of the spaces. Okay, so improve that table. And also in the header section of the instances, there are buttons which does not look very good. Buttons has too many rounded corners. Usually, the rounded corners would be five to six pixel for the rounded corner, not more than that. And also the rotate to the next best is terrible button. So you should fix that button using padding. Okay? And what is this rotate to next best? Which instance will apply to this? Have no clue. If I hover over, I should be able to see that, but nothing is there. Okay. The card mode is very terrible still. The card mode needs to be more compact. That means at least one row, we need to able to show four items. Okay? So try to do it like this, four items. So compact the option, and also the buttons in the card mode is still broken. The buttons does not look professional. It's been several times that I have requested. I hope you keep the request. So first you are going to include the task, put the images, create the spec, inside the spec, refer to those images, and then you start fixing this one. Okay, and also include the Supabase by the settings by running the CLI and other stuff. Okay. So I have changed the theme, but the audit color looks like very green, which is odd, actually. I think you may need to fix this for me. Yeah, I think this is enough. Please help me in fixing this, and at the end, make a minor bump and release.

---

## 2. Visual Reference Assets

- `![Rename Profile Modal](assets/screenshots/134-accounts-ui-01-rename-profile-modal.png)`
  *Defect:* Input has excessive pill radius, button text padding cramped, card background displays ending path truncation.
- `![Duplicate Profile Modal](assets/screenshots/134-accounts-ui-02-duplicate-clone-modal.png)`
  *Defect:* Navigation bar header buttons and modal form controls use oversized radii instead of 5–6px.
- `![Theme Catalogue and Quota Progress](assets/screenshots/134-accounts-ui-03-theme-catalogue-quota-bars.png)`
  *Defect:* Quota progress bars render harsh neon green gradients in dark themes; audit status badges appear glaringly green.
- `![Show All Quotas Header Toggle](assets/screenshots/134-accounts-ui-04-show-all-quotas-toggle.png)`
  *Defect:* Header toolbar buttons lack consistent border definition and row borders below are faint.
- `![Accounts Table Red Boxes](assets/screenshots/134-accounts-ui-05-accounts-table-red-boxes.png)`
  *Defect:* Red boxes highlight (1) saturated neon green quota bars, (2) missing middle section grouping around 4H and weekly quota columns, (3) weakly separated table rows.

---

## 3. High-Level Task Decomposition

```
                                  [MASTER TASK 134]
                                          |
        +---------------------------------+---------------------------------+
        |                                                                   |
        v                                                                   v
 [PHASE 1: SPECS & PLANS]                                            [PHASE 2: IMPLEMENTATION]
  - Task-01: Architecture Spec                                        - Task-02: Accounts Table Borders,
    & Subtask Decomposition                                             Grouping & Cyan/Teal Palette
                                                                      - Task-03: Supabase Auto-Discovery,
                                                                        Lease Locks & Email Cooldown
                                                                      - Task-04: Instances Table Compact,
                                                                        Merged Column & Prompts Action
                                                                      - Task-05: Button Corner Radius (5-6px)
                                                                        & Rotate Tooltip Polish
                                                                      - Task-06: Compact Card Grid (4 cols)
                                                                        & 2-Row Action Toolbars
                                                                            |
                                                                            v
                                                                     [PHASE 3: RELEASE & CI/CD]
                                                                      - Task-07: Pre-flight Verification,
                                                                        Minor Bump (v4.154.0) & @aukgit Release
```

### Detailed Subtask Descriptions:

1. **Task-01: Spec Authoring, Visual Audit & Master Planning**
   - Author `02-spec/21-app/134-accounts-ui-supabase-instance-compact/01-architecture-spec.md`.
   - Author `.ai-memory/plans/134-accounts-ui-supabase-instance-compact.md` and child subtask plans.
   - Reference the 5 uploaded screenshots strictly using relative markdown links.
   - Ground all designs in existing codebase patterns without executing git commands.

2. **Task-02: Accounts Table Borders, Middle Grouping & VS Code Cyan/Teal Palette**
   - Add explicit horizontal row divider borders: `border-b border-slate-200/90 dark:border-slate-800/90`.
   - Middle section visual grouping: Cluster 4H Model Quota and Weekly Quota columns with dedicated background tint (`bg-slate-50/50 dark:bg-slate-900/40`) and outer bounding borders (`border-x border-slate-300 dark:border-slate-700`).
   - Quota progress bar colors: Replace saturated neon green (`emerald-500`, `lime-400`, `#1af18d`) with VS Code professional cyan/teal/sky palette (`from-teal-500 via-cyan-500 to-[#38bdf8]`).
   - Audit badge modernization: In `src/pages/Audit.tsx`, replace harsh green badges with muted cyan/teal badges (`bg-cyan-50 text-cyan-700 dark:bg-cyan-950/60 dark:text-cyan-300`).

3. **Task-03: Supabase Auto-Discovery, Distributed Leases & Configurable Email Cooldown**
   - Auto-discover secrets from `D:\work\repo-secrets` (including `02-antigravity-and-event-manager\vault\supabase_config.json`, `02-antigravity-manager\vault\supabase_config.json`, `03-supabase\01-own\supabase-credentials.json`, `03-supabase\02-lovable\supabase-credentials.json`).
   - Decode Base64 payloads and populate `supabase_config.json` automatically on startup when endpoints are empty or when invoked via CLI/Settings.
   - Implement pre-switch lease lock validation in `account.rs` and `instance.rs` to intercept switches to accounts leased by foreign machines.
   - Add configurable `account_cooldown_minutes` (30–60 min default) in `AutoProfileSwitcherConfig` and `AutoSwitcherSettings.tsx`.
   - Update `auto_switcher.rs` candidate selection to skip cooling accounts unless all accounts are cooling down (deadlock prevention).

4. **Task-04: Instances Table Compacting, Merged Column & Prompts Dropdown Action**
   - Eliminate horizontal table scrollbar at >= 1200px viewports by rebalancing column widths.
   - Merge "Profile Name" and "Bound Account" into a single consolidated cell with two stacked visual tiers (Top: `#seq` + Profile Name + Active/Default tags; Bottom: Mail icon + email + Tier badge).
   - Implement path truncation to ending folder (`formatEndingFolder` -> `...\<folder>`) with hover tooltip showing full path and copy icon.
   - Remove standalone "Prompts" column or loose button, integrating it into the table action dropdown portal.

5. **Task-05: Button Corner Radius Normalization (5–6px) & Rotate Tooltip Polish**
   - Standardize all interactive buttons across Instances and Accounts to `rounded-[5px]`.
   - Fix "Rotate to Next Best" header button: adjust padding to balanced `px-3 py-1.5`, apply `rounded-[5px]`.
   - Add dynamic hover tooltip showing target active instance name and next candidate email + quota health score.

6. **Task-06: Compact Card Grid (4 Items per Row) & 2-Row Action Toolbars**
   - Set card grid container to strictly 4 columns at standard viewports: `grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3`.
   - Reorganize card action buttons into two clean, professional rows:
     - Row 1: Launch/Stop/Restart, Switch Account, Fast-Forward, Sync PID & Quota.
     - Row 2: Prompts, Settings & Sync, Clone Profile, Audit Trail, More Popover.
   - Ensure all card buttons use `rounded-[5px]` with compact padding.

7. **Task-07: Pre-Flight Verification & Minor Version Bump Release Ceremony (v4.154.0)**
   - Pre-flight quality gates: `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, `npm run build`.
   - Minor version bump: `npm run bump minor` synchronizing manifests to `4.154.0`.
   - Update `changelog.md` and `changelog_en.md` with strict `@aukgit` attribution invariant.
   - Synchronize release notes in `README.md` and `README_EN.md`.

---

## 4. Requirements Traceability Matrix

| Requirement | Spec Anchor (`02-spec/21-app/`) | Plan / Subtask Anchor (`.ai-memory/plans/`) | Source Code Targets |
|---|---|---|---|
| Accounts row borders | `01-architecture-spec.md` §4.1 | `subtasks/.../01-accounts-table-borders-grouping-and-cyan-palette.md` | `AccountTable.tsx`, `AccountRow.tsx` |
| Middle section grouping | `01-architecture-spec.md` §4.2 | `subtasks/.../01-accounts-table-borders-grouping-and-cyan-palette.md` | `AccountTable.tsx`, `AccountRow.tsx` |
| Cyan/teal progress bar | `01-architecture-spec.md` §4.3 | `subtasks/.../01-accounts-table-borders-grouping-and-cyan-palette.md` | `QuotaProgressBar.tsx`, `WaterDrainProgressBar.tsx` |
| VS Code audit badge colors | `01-architecture-spec.md` §4.3 | `subtasks/.../01-accounts-table-borders-grouping-and-cyan-palette.md` | `src/pages/Audit.tsx` |
| Supabase auto-discovery | `01-architecture-spec.md` §5.1 | `subtasks/.../02-supabase-secrets-auto-discovery-and-email-cooldown.md` | `supabase_sync.rs`, `commands/supabase.rs` |
| Cross-machine lease lock | `01-architecture-spec.md` §5.2 | `subtasks/.../02-supabase-secrets-auto-discovery-and-email-cooldown.md` | `workspace_lease_manager.rs`, `account.rs`, `instance.rs` |
| 30-60m email cooldown | `01-architecture-spec.md` §5.3 | `subtasks/.../02-supabase-secrets-auto-discovery-and-email-cooldown.md` | `auto_switcher.rs`, `models/config.rs`, `AutoSwitcherSettings.tsx` |
| Instance table no scroll | `01-architecture-spec.md` §6.1 | `subtasks/.../03-instances-table-compacting-combined-column-and-prompts.md` | `InstanceTable.tsx` |
| Merged profile & email | `01-architecture-spec.md` §6.2 | `subtasks/.../03-instances-table-compacting-combined-column-and-prompts.md` | `InstanceTable.tsx` |
| Ending path truncation | `01-architecture-spec.md` §6.3 | `subtasks/.../03-instances-table-compacting-combined-column-and-prompts.md` | `InstanceTable.tsx` |
| Prompts in dropdown | `01-architecture-spec.md` §6.4 | `subtasks/.../03-instances-table-compacting-combined-column-and-prompts.md` | `InstanceTable.tsx` |
| 5–6px button radius | `01-architecture-spec.md` §7.1 | `subtasks/.../04-button-radius-rotate-tooltip-and-card-grid.md` | `Instances.tsx`, `InstanceTable.tsx`, modals |
| Rotate button padding & tooltip | `01-architecture-spec.md` §7.2 | `subtasks/.../04-button-radius-rotate-tooltip-and-card-grid.md` | `Instances.tsx` |
| 4 cards per row grid | `01-architecture-spec.md` §8.1 | `subtasks/.../04-button-radius-rotate-tooltip-and-card-grid.md` | `Instances.tsx` |
| 2-row card action buttons | `01-architecture-spec.md` §8.2 | `subtasks/.../04-button-radius-rotate-tooltip-and-card-grid.md` | `Instances.tsx` |
| Minor release bump v4.154.0 | `01-architecture-spec.md` §9 | `subtasks/.../05-preflight-minor-bump-and-release.md` | `package.json`, `Cargo.toml`, changelogs, READMEs |

---

## 5. Multi-Agent Execution Orchestration

```
+----------------------------------------------------------------------------------------------------+
|                                    WAVE 1: SPECIFICATION & PLANNING                                |
| - Spec Author 01: Architecture Spec (01), Master Plan, Subtasks 01 & 02                            |
| - Spec Author 02: Component Spec (02), RCA (03), Subtasks 03, 04, 05                               |
+----------------------------------------------------------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                      WAVE 2: PARALLEL EXECUTION                                    |
| [WORKER 01: FRONTEND UI SPECIALIST]                 [WORKER 02: BACKEND & DISTRIBUTED SYSTEMS]     |
| - Subtask 01: Accounts Table borders, middle        - Subtask 02: Supabase secrets discovery from   |
|   grouping, cyan/teal progress bar, Audit badges      repo-secrets, distributed cross-machine lease |
| - Subtask 03: Instance table scroll removal,          pre-switch check, 30-60m email cooldown in    |
|   merged profile/email, path truncation               auto_switcher.rs, AutoSwitcherSettings.tsx    |
| - Subtask 04: Button radius 5-6px, rotate tooltip,                                                 |
|   4-item card grid, 2-row card action buttons                                                      |
+----------------------------------------------------------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                  WAVE 3: CONVERGENCE & RELEASE GATES                               |
| - Run cargo fmt -- --check and cargo clippy --all-targets --all-features                           |
| - Run npm run build                                                                                |
| - Execute npm run bump minor (v4.154.0)                                                            |
| - Validate strictly @aukgit attribution in changelog.md and changelog_en.md                        |
| - Synchronize README.md and README_EN.md release notes                                             |
+----------------------------------------------------------------------------------------------------+
```

---

## 6. Verification Checklist & Acceptance Gates

- [ ] **Gate 1 (Accounts Table Borders):** Every row has visible `border-b border-slate-200/90 dark:border-slate-800/90` and slate hover highlighting.
- [ ] **Gate 2 (Middle Section Grouping):** 4H Model Quota and Weekly Quota headers and cells are wrapped in a distinct visual group with borders and subtle background.
- [ ] **Gate 3 (VS Code Palette):** Quota progress bars show cyan/teal gradients; audit log badges use cyan/teal instead of harsh green.
- [ ] **Gate 4 (Supabase Secrets Discovery):** Endpoints auto-load from `D:\work\repo-secrets` on launch and via CLI `agm supabase load-json`.
- [ ] **Gate 5 (Cross-Machine Account Locks):** Pre-switch validation blocks switching to accounts currently leased by another active machine.
- [ ] **Gate 6 (Email Cooldown Window):** Auto-switcher skips accounts used within 30–60m across machines, falling back to oldest if all are in cooldown.
- [ ] **Gate 7 (Instance Table No Scroll):** Zero horizontal scrollbar appears on desktop resolutions (>= 1200px); Profile & Email are merged into 1 column.
- [ ] **Gate 8 (Path Truncation):** Location paths display `...\<folder>` ending directory format with hover tooltip showing full path.
- [ ] **Gate 9 (Button Radius & Rotate Tooltip):** All buttons use `rounded-[5px]`; "Rotate to Next Best" has proper padding and informative target hover tooltip.
- [ ] **Gate 10 (Card Mode 4 Cols & 2 Rows):** Card grid renders strictly 4 items per row; card actions are organized into two balanced rows.
- [ ] **Gate 11 (Pre-Flight Checks):** Rust clippy passes with zero warnings; React frontend builds cleanly without compilation errors.
- [ ] **Gate 12 (Minor Version Bump):** Version synced to `4.154.0` across all manifests; changelog strictly attributes `@aukgit`.
