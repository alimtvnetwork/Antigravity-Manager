# UI Account & Instance Compacting, VS Code Theming, Supabase Secrets Loading, & Multi-Instance Account Lease Isolation Specification

## 1. Original User Request (Verbatim)

> Hi there. So far it looks good in most cases, but there are some issues. For example, when I go into the account section, there was a little bit of lines and it looked like the middle section has some, let's say, grouping or showed nicely. There was a border type thing, and then rows were a little bit showcased before. I really like that you can have this. And also I requested for a different type of coloring for the green ones. You could give me some suggestions or try it out, making the progress bar a little bit better. That's for the home screen account section. I think this is the improvement that you can do. Okay? Now going to the instance section. Also, one more important point, the Supabase is not loaded from the repo secrets folder. Okay? Load the Supabase. Make sure that the same account is not tapped into by multiple instances or multiple account across the other machines as well. Okay? Using the Supabase ensure that, and also the email can be another way of checking. So let's say in Supabase or email, we would check that the current email has been used or taken by any other account in last one hour or 30 minutes, something like this. That should be changeable from the settings. Okay? So based on that, if some email has been taken by some other account or instance, we skip that email. Okay? Unless we have no other email. Okay, so this is how it should be coded. Remember that. And instance section looks nice, but the greener color, I really do not appreciate that much of a green in the table section. It could be a little bit like the VS Code theme. I would appreciate that. You could improve the coloring and also the table that is there. It has a scroll, actually. It does not look good with the scroll. So you could actually reduce some options. For example, the profile name. The profile name and email that can be configured together. Right. There is no need to have two columns for this. Respect that, I believe. Okay, the path for the location, we only wanted to see the ending path rather than the starting path, because the starting path most probably same. So starting part will be dot, dot, dot. The ending part is required. Now, there's a prompts button. Prompts button can be added to the action section. Okay? As with the drop-downs. That would also save some of the spaces. Okay, so improve that table. And also in the header section of the instances, there are buttons which does not look very good. Buttons has too many rounded corners. Usually, the rounded corners would be five to six pixel for the rounded corner, not more than that. And also the rotate to the next best is terrible button. So you should fix that button using padding. Okay? And what is this rotate to next best? Which instance will apply to this? Have no clue. If I hover over, I should be able to see that, but nothing is there. Okay. The card mode is very terrible still. The card mode needs to be more compact. That means at least one row, we need to able to show four items. Okay? So try to do it like this, four items. So compact the option, and also the buttons in the card mode is still broken. The buttons does not look professional. It's been several times that I have requested. I hope you keep the request. So first you are going to include the task, put the images, create the spec, inside the spec, refer to those images, and then you start fixing this one. Okay, and also include the Supabase by the settings by running the CLI and other stuff. Okay. So I have changed the theme, but the audit color looks like very green, which is odd, actually. I think you may need to fix this for me. Yeah, I think this is enough. Please help me in fixing this, and at the end, make a minor bump and release.

---

## 2. Preserved Visual Evidence & Screenshot Analysis

The user provided visual evidence illustrating UI deficiencies in the Accounts section and Instances page:

### 2.1 Accounts Section: Borders, Grouping, and Harsh Green Progress Bars
![Account Section Grouping and Quota Colors](file:///d:/work/Antigravity-Manager/assets/screenshots/106-01-account-section-grouping-and-quota-colors.png)
*Figure 2.1: Accounts table currently lacks subtle borders/dividers and rows have jarring contrast. The quota progress bars use saturated neon green (`emerald-500` / `lime-400`) rather than modern VS Code teal/cyan/sky tones.*

### 2.2 Instance Card Mode: Broken Button Layout and Path Length Overflow
![Instance Card Buttons and Path](file:///d:/work/Antigravity-Manager/assets/screenshots/106-02-instance-card-buttons-and-path.png)
*Figure 2.2: Action buttons in card mode wrap haphazardly without visual hierarchy or consistent 5–6px border radius. Full absolute paths overflow card bounds.*

### 2.3 Instance Header: "Rotate to Next Best" Button Padding & Tooltip Deficit
![Instance Header Rotate Next Best Button](file:///d:/work/Antigravity-Manager/assets/screenshots/106-03-instance-header-rotate-next-best-button.png)
*Figure 2.3: The "Rotate to Next Best" header button suffers from disproportionate pill padding, lacks contextual indication of which active instance it affects, and lacks a hover tooltip.*

### 2.4 Instance Cards: Single/Dual Column Stack Lacking Compactness
![Instance Card Stack Compact Grid](file:///d:/work/Antigravity-Manager/assets/screenshots/106-04-instance-card-stack-compact-grid.png)
*Figure 2.4: Cards are rendered in an overly wide 1–2 column layout (`xl:grid-cols-2`), wasting horizontal screen real estate. The user requests 4 cards per row.*

---

## 3. Architecture & Functional Specifications

### 3.1 Accounts Section Visual Styling & VS Code Themed Quota Bars
1. **Row Borders & Subtle Grouping**:
   - In `src/components/accounts/AccountTable.tsx` and `AccountRow.tsx`, add clean row borders `border-b border-slate-200/80 dark:border-slate-800/80`.
   - Replace aggressive pitch black / bright yellow hover states (`#070b10` / `#f5d76e`) with subtle VS Code navy/slate styling: `hover:bg-slate-50/80 dark:hover:bg-[#0f273d]/60` with a subtle left accent line `border-l-2 border-l-blue-500/70`.
2. **Quota Progress Bar VS Code Palette**:
   - In `src/components/accounts/QuotaItem.tsx`, `AccountRow.tsx`, and `AccountCard.tsx`:
     - Replace neon green `from-emerald-500 via-teal-400 to-cyan-400` with VS Code professional cyan/teal/sky palette:
       - Healthy (>= 50%): `bg-gradient-to-r from-teal-500 via-cyan-500 to-sky-500` with dark background track `bg-slate-200 dark:bg-slate-800/80`.
       - Warning (20% - 49%): `bg-gradient-to-r from-amber-500 to-amber-400`.
       - Exhausted (< 20%): `bg-gradient-to-r from-rose-500 to-rose-400`.
     - Soften quota numerical badges from harsh green to muted cyan text (`text-cyan-600 dark:text-cyan-400`).

### 3.2 Supabase Repo-Secrets Auto-Discovery Ladder
1. **Discovery Hierarchy**:
   - In `src-tauri/src/modules/supabase_sync.rs` and `supabase_client.rs`:
     - When `supabase_config.json` has empty `endpoints`, systematically probe:
       1. `REPO_SECRETS_DIR` environment variable.
       2. `D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json`
       3. `D:/work/repo-secrets/03-supabase/02-lovable/supabase-credentials.json`
       4. `C:/work/repo-secrets/...`, `../repo-secrets/...`, `../../repo-secrets/...`, `./repo-secrets/...`
       5. `dirs::home_dir()/repo-secrets/...`
2. **Payload Parsing**:
   - Ingest JSON envelopes matching `"type": "agm/supabase-credentials"`.
   - Decode Base64 payloads (`endpoint`, `token`, `service`, `role`), normalize URLs, assign role `root` or `secondary`, and auto-persist into `supabase_config.json`.

### 3.3 Multi-Instance & Cross-Machine Account Isolation
1. **Distributed Lease Locking**:
   - Every instance switch or profile launch acquires a lease in Supabase `workspace_leases` table.
   - Extend lease TTL from short 90s to match configurable cooldown (e.g. 1800s - 3600s).
   - Before binding or switching to an account, check `is_account_or_email_leased_by_other(acc_id, email)`:
     - If leased by another active node or instance, strictly reject/skip that account.
2. **Configurable Email Cooldown Window**:
   - In `src-tauri/src/models/config.rs` (`AutoProfileSwitcherConfig`):
     - Add `account_cooldown_minutes: u32` (default: 60, options: 15, 30, 45, 60, 120).
   - In `src-tauri/src/modules/auto_switcher.rs`:
     - Filter candidate accounts: an account is in cooldown if `now - acc.last_used < cooldown_seconds` OR remote lease was established within `cooldown_seconds`.
     - **Graceful Fallback**: If non-cooldown candidates exist, pick exclusively from non-cooldown accounts. If and only if ALL eligible accounts are in cooldown, fall back to the oldest cooldown account to prevent deadlocks.
3. **Settings UI Parity**:
   - In `src/types/config.ts` and `src/components/settings/AutoSwitcherSettings.tsx`, provide a dropdown/input for "Account Reuse Cooldown (Minutes)".

### 3.4 Instance Table Scroll Elimination & Field Compacting
1. **Combined "Profile & Account" Column**:
   - Merge "Profile Name" and "Bound Account" columns into a single compact column:
     - Top line: Profile name badge (`font-semibold text-slate-800 dark:text-slate-100`).
     - Bottom line: Bound email (`text-xs text-slate-500 font-mono`) with account status indicator.
2. **Ending Path Truncation**:
   - Implement `formatEndingPath(path)`:
     - If path length > 28 chars, format as `...\<parent>\<leaf>` (e.g. `...\AppData\Roaming\Antigravity`).
     - Show full path in tooltip on hover.
3. **Prompts Action in Dropdown/Capsule**:
   - Remove standalone "Prompts" button taking up horizontal space.
   - Integrate "Prompts" action into the segmented table action dropdown/capsule alongside Launch, Stop, Switch, and Settings.

### 3.5 Button Styling & 5–6px Border Radius Standard
1. **Radius Enforcement**:
   - Eliminate bulbous pill buttons (`rounded-full`, `rounded-2xl`, `rounded-xl`).
   - Standardize all buttons across Instances and Accounts to 5–6px: `rounded-[5px]` or `rounded-md`.
2. **"Rotate to Next Best" Header Button**:
   - Change padding to balanced `px-3 py-1.5`.
   - Apply `rounded-[5px]`.
   - Add dynamic tooltip showing the target instance: `Rotates active instance (#<id> <name>) to the highest health candidate`.
3. **Softened Audit Button Colors**:
   - Replace harsh neon green with subtle VS Code slate styling:
     - `bg-slate-100 dark:bg-slate-800/80 text-slate-700 dark:text-slate-300 border-slate-300 dark:border-slate-700 hover:bg-slate-200 dark:hover:bg-slate-700`.

### 3.6 Instance Card Mode 4-per-Row Compact Grid
1. **Grid Layout**:
   - Update `src/pages/Instances.tsx` card container to:
     `grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3`
2. **2-Row Professional Card Action Toolbar**:
   - Row 1 (Primary Lifecycle): Stop/Launch button, Switch Account, Fast-Forward, Audit, Sync.
   - Row 2 (Utilities & Danger): Prompts, Settings, Clone, Executable, Wipe, Delete.
   - All card buttons styled with compact `px-2 py-1 text-xs rounded-[5px]`.

### 3.7 CLI Parity for Supabase Commands
1. **`agm supabase` & `cli.rs` Integration**:
   - Ensure `agm supabase status`, `agm supabase sync`, `agm supabase load-json`, `agm supabase list-leases`, and `agm supabase set-config` function identically.
   - Add CLI support for `antigravity-manager.exe supabase` in `src-tauri/src/modules/cli.rs`.

---

## 4. Verification & Testing Matrix

| Component | Target Verification | Gate |
|---|---|---|
| Accounts Section | Row borders visible, VS Code hover highlight, teal/cyan quota bar | Visual & DOM inspection |
| Instance Table | No horizontal scroll at 1280px+; Profile & Email merged; ending paths rendered | DOM inspection |
| Header Buttons | Rotate to Next Best has 5–6px radius, balanced padding, and hover tooltip | Visual & DOM inspection |
| Card Grid | 4 cards per row at >=1280px; 2-row button toolbar with 5–6px radius | Visual & CSS inspection |
| Audit Buttons | VS Code slate background instead of harsh green | Visual & CSS inspection |
| Supabase Secrets | Auto-loads from `D:\work\repo-secrets\03-supabase\01-own\supabase-credentials.json` | Rust unit / sync verification |
| Account Isolation | Email cooldown check skips accounts leased in last 30/60m unless all cooling down | Rust logic verification |
| CLI Parity | `agm supabase status` and `agm supabase sync` run cleanly | CLI output verification |
| Build & Pre-flight | `cargo fmt`, `cargo clippy`, `npm run build` pass without error | CI Pre-flight gate |
