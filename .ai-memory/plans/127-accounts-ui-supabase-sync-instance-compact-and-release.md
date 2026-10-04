# Execution Plan: 127-accounts-ui-supabase-sync-instance-compact-and-release

## User Request (Verbatim)
> Hi there. So far it looks good in most cases, but there are some issues. For example, when I go into the account section, there was a little bit of lines and it looked like the middle section has some, let's say, grouping or showed nicely. There was a border type thing, and then rows were a little bit showcased before. I really like that you can have this. And also I requested for a different type of coloring for the green ones. You could give me some suggestions or try it out, making the progress bar a little bit better. That's for the home screen account section. I think this is the improvement that you can do. Okay? Now going to the instance section. Also, one more important point, the Supabase is not loaded from the repo secrets folder. Okay? Load the Supabase. Make sure that the same account is not tapped into by multiple instances or multiple account across the other machines as well. Okay? Using the Supabase ensure that, and also the email can be another way of checking. So let's say in Supabase or email, we would check that the current email has been used or taken by any other account in last one hour or 30 minutes, something like this. That should be changeable from the settings. Okay? So based on that, if some email has been taken by some other account or instance, we skip that email. Okay? Unless we have no other email. Okay, so this is how it should be coded. Remember that. And instance section looks nice, but the greener color, I really do not appreciate that much of a green in the table section. It could be a little bit like the VS Code theme. I would appreciate that. You could improve the coloring and also the table that is there. Look at the screenshot in the table section. You have to scroll to the right. We do not want that. For example, the profile name and the bound email can be put together in one section. Just look at the size of the table. You don't have to spread things around. And also the folder, only the ending path is enough, like which project is that. So that would be enough. Also, we had that button, the prompt button. It's not necessarily needed to be separately there. It can be merged in the capsule button that is there. And the other buttons like the card format, you can have 4 items per row. The buttons look bad. In card mode, you can have 2 rows of buttons instead of 1 row, and with the rounded 5 or 6px. Even in the card format and everywhere, the rounded border of the buttons should be 5 to 6px. Also the Rotate to Next Best button: make sure that the button has proper padding. If there's an instance selected, it should rotate to the next best for that instance. If no instance selected, on hover tell the user which instance it will rotate to (or the active/default instance). And release: minor bump and release ceremony at the end.

## Visual Reference Assets
- ![Accounts Styling 01](assets/screenshots/127-accounts-styling-01.png)
- ![Accounts Styling 02](assets/screenshots/127-accounts-styling-02.png)
- ![Accounts Styling 03](assets/screenshots/127-accounts-styling-03.png)
- ![Accounts Styling 04](assets/screenshots/127-accounts-styling-04.png)

## Core Architectural Objectives

### 1. Account Section UI Modernization (Task-01)
- **QuotaProgressBar & WaterDrainProgressBar**:
  - Replace harsh neon green `#1af18d`, `#10b981`, `#059669` with calm VS Code teal/cyan/sky palette (`from-teal-500 via-cyan-500 to-sky-500`).
  - Milestone checkpoint nodes: replace neon shadow `shadow-[0_0_8px_rgba(26,241,141,0.6)]` with soft cyan `bg-cyan-500 border-cyan-400 shadow-[0_0_6px_rgba(6,182,212,0.4)]`.
- **AccountTable & AccountRow**:
  - Crisp border rows: `border-b border-slate-200/80 dark:border-slate-800/80` with focused/current highlight styling.
  - Soften audit/status badges (`DISABLED`, `FORBIDDEN`, `PROXY_DISABLED`, `VALIDATION_BLOCKED`, running indicator) from harsh green to muted VS Code slate/teal/cyan.

### 2. Instance Table Compacting & Scrollbar Elimination (Task-02)
- Compact column widths to guarantee zero horizontal scrollbar on standard 1280px+ displays:
  - Tighten table capsule button horizontal padding to `px-1.5`.
  - Tighten cell padding to `px-2`.
  - Single column for Profile Name and Bound Email with proper truncation.
  - Path truncation: `formatShortPath(fullPath)` (`...\<parent>\<leaf>`) with max-w constraint (`max-w-[140px]`).
  - Segmented capsule includes the Prompts button (`Layers` icon).
  - Modernize launch spinner and sync buttons from green/emerald to VS Code cyan/teal.

### 3. Instance Cards & Buttons Polish (Task-03)
- Standardize button border radii strictly to 5–6px (`rounded-[5px]`).
- Card grid: 4 cards per row (`xl:grid-cols-4`).
- Action buttons in card mode: clean 2-row layout (Row 1: 5 lifecycle/rotation buttons; Row 2: 6 tools/settings buttons) with `rounded-[5px]`.
- "Rotate to Next Best" header button: ensure `px-3 py-1.5 rounded-[5px]` with dynamic hover tooltip explaining whether it rotates the selected instance or the active default instance.

### 4. Supabase Auto-Discovery from Repo-Secrets (Task-04)
- Probing `D:\work\repo-secrets\03-supabase\01-own\supabase-credentials.json` and `REPO_SECRETS_DIR`.
- Auto-seed endpoints when config is empty, normalizing trailing slashes and ensuring valid PostgREST connectivity.
- Parity across CLI tools (`agm supabase` and `antigravity-manager.exe supabase`).

### 5. Multi-Instance Exclusivity & Email Usage Cooldown (Task-05)
- Supabase distributed leasing (`workspace_leases` table) ensures no concurrent account usage across instances and cross-machine nodes.
- `account_cooldown_minutes` (default: 60 mins, configurable in settings 15–120 min) added to `AutoProfileSwitcherConfig`.
- Candidate selection: skips accounts leased or used within the cooldown window, with two-tier graceful fallback if all accounts are cooling down.
- Expose setting in `AutoSwitcherSettings.tsx` and sync in `config.ts`.

### 6. Verification & Release Ceremony (Task-06)
- File-scoped checks, secrets gate, zero build/test ban during routine turns.
- Minor version bump: `npm run bump:minor`.
- Atomic GitMap push: `gitmap cpf "<module> - <summary>"`.

## Multi-Agent Execution Waves
- **Wave 1 (Planning & Specs)**:
  - Subagent 1: Spec 01 (Architecture) + Subtasks 01 & 02
  - Subagent 2: Spec 02 (Component & Backend) + Subtasks 03 & 04
- **Wave 2 (Execution)**:
  - Worker 01: Frontend UI Refactoring (QuotaProgressBar, AccountTable, InstanceTable, Instances.tsx)
  - Worker 02: Backend Rust & Supabase (supabase_sync.rs, auto_switcher.rs, config.rs, AutoSwitcherSettings.tsx)
- **Wave 3 (Consolidation & Release)**:
  - Lead: Verification, Minor Bump Ceremony, Atomic Push
