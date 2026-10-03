# Architectural Specification: Instances Modernization, Supabase Secrets Auto-Discovery & Account Exclusivity Guard

**Task Identifier**: `105-instances-modernization-supabase-secrets-and-table-compactness`  
**Version**: `1.0.0`  
**Status**: `APPROVED & IN EXECUTION`  
**Maintainer**: `@aukgit`  
**Traceability**: User UI/UX feedback & screenshots (`media_1791050281941.png` - `media_1791050632091.png`)

---

## 1. User Request (Verbatim)

```text
https://prnt.sc/6NYSXKzXywu4

# High Priority Instruction

Hi there. So far it looks good in most cases, but there are some issues. For example, when I go into the account section, there was a little bit of lines and it looked like the middle section has some, let's say, grouping or showed nicely. There was a border type thing, and then rows were a little bit showcased before. I really like that you can have this. And also I requested for a different type of coloring for the green ones. You could give me some suggestions or try it out, making the progress bar a little bit better. That's for the home screen account section. I think this is the improvement that you can do. Okay? Now going to the instance section. Also, one more important point, the Supabase is not loaded from the repo secrets folder. Okay? Load the Supabase. Make sure that the same account is not tapped into by multiple instances or multiple account across the other machines as well. Okay? Using the Supabase ensure that, and also the email can be another way of checking. So let's say in Supabase or email, we would check that the current email has been used or taken by any other account in last one hour or 30 minutes, something like this. That should be changeable from the settings. Okay? So based on that, if some email has been taken by some other account or instance, we skip that email. Okay? Unless we have no other email. Okay, so this is how it should be coded. Remember that. And instance section looks nice, but the greener color, I really do not appreciate that much of a green in the table section. It could be a little bit like the VS Code theme. I would appreciate that. You could improve the coloring and also the table that is there. It has a scroll, actually. It does not look good with the scroll. So you could actually reduce some options. For example, the profile name. The profile name and email that can be configured together. Right. There is no need to have two columns for this. Respect that, I believe. Okay, the path for the location, we only wanted to see the ending path rather than the starting path, because the starting path most probably same. So starting part will be dot, dot, dot. The ending part is required. Now, there's a prompts button. Prompts button can be added to the action section. Okay? As with the drop-downs. That would also save some of the spaces. Okay, so improve that table. And also in the header section of the instances, there are buttons which does not look very good. Buttons has too many rounded corners. Usually, the rounded corners would be five to six pixel for the rounded corner, not more than that. And also the rotate to the next best is terrible button. So you should fix that button using padding. Okay? And what is this rotate to next best? Which instance will apply to this? Have no clue. If I hover over, I should be able to see that, but nothing is there. Okay. The card mode is very terrible still. The card mode needs to be more compact. That means at least one row, we need to able to show four items. Okay? So try to do it like this, four items. So compact the option, and also the buttons in the card mode is still broken. The buttons does not look professional. It's been several times that I have requested. I hope you keep the request. So first you are going to include the task, put the images, create the spec, inside the spec, refer to those images, and then you start fixing this one. Okay, and also include the Supabase by the settings by running the CLI and other stuff. Okay. So I have changed the theme, but the audit color looks like very green, which is odd, actually. I think you may need to fix this for me. Yeah, I think this is enough. Please help me in fixing this, and at the end, make a minor bump and release.
```

---

## 2. Screenshot & Visual Evidence Integration

| Screenshot Artifact | Description & Highlighted Problem |
| :--- | :--- |
| ![Instance Cards](assets/screenshots/105-instances-modernization-supabase-secrets-and-table-compactness-01.png) | **Instance Card Layout**: 2-column wide layout with broken multi-colored button rows (`Stop`, `Switch`, `FF`, `Audit`, `Sync`), excessive vertical height, and unoptimized progress bars. |
| ![Instances Toolbar](assets/screenshots/105-instances-modernization-supabase-secrets-and-table-compactness-02.png) | **Instances Top Header & Auto-Switcher Banner**: Excessive pill border radius (>16px), missing padding on "Rotate to Next Best", and missing target instance tooltip on hover. |
| ![Accounts Table](assets/screenshots/105-instances-modernization-supabase-secrets-and-table-compactness-03.png) | **Accounts Table**: Request for subtle divider border around the middle `4H / WEEKLY QUOTA` section and replacement of harsh neon green progress bars with VS Code dark IDE palette. |
| ![Instance Table Scroll](assets/screenshots/105-screenshots/105-instances-modernization-supabase-secrets-and-table-compactness-04.png) | **Instance Table Horizontal Scrollbar**: 8 columns occupying >1,158px width forcing horizontal scrolling; redundant Profile Name and Bound Account columns; full filesystem path; separate Prompts column. |
| ![Card Action Buttons](assets/screenshots/105-instances-modernization-supabase-secrets-and-table-compactness-05.png) | **Card Action Buttons Red Box**: Disjointed rainbow pastel buttons with oversized rounded corners requiring contiguous segmented styling and 5-6px corner radius (`rounded-md`). |

---

## 3. Architecture & Functional Specification

### 3.1 Backend Supabase Secrets Auto-Discovery
1. **Multi-Path Discovery Engine**:
   In `src-tauri/src/modules/supabase_sync.rs`, implement `candidate_seed_config_paths()` scanning:
   - `D:/work/repo-secrets/02-antigravity-manager/vault/supabase_config.json`
   - `D:/work/repo-secrets/vault/supabase_config.json`
   - `../repo-secrets/02-antigravity-manager/vault/supabase_config.json`
   - `../../repo-secrets/02-antigravity-manager/vault/supabase_config.json`
   - `D:/work/repo-secrets/03-supabase/02-lovable/supabase-credentials.json`
   - `D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json`
2. **Envelope Expansion & Auto-Seeding**:
   When local `%APPDATA%\antigravity-manager\supabase_config.json` or `~/.antigravity_tools/supabase_config.json` is missing or has empty endpoints, deserialize via `json_envelope::extract_payload::<SupabaseConfig>`, interpolate template variables (`${rootLovableUrl}`, `${secondaryUrl}`, `${nodeAlias}`), set `is_sync_enabled = true`, normalize URLs, and persist to local configs.

### 3.2 Account Exclusivity & Configurable Lockout Window
1. **Lockout Configuration**:
   In `src-tauri/src/models/config.rs`, add `pub account_lockout_window_minutes: u32` (default: 60) to `AutoProfileSwitcherConfig`. Expose in Settings UI dropdown (30m, 60m, 120m).
2. **Fix `workspace_lease_manager.rs` Bug**:
   Correct line 266 in `is_account_or_email_leased_by_other`: check `lease.account_email` instead of `lease.profile_name`. Enforce lockout window `(now - lease.leased_at) < lockout_window_secs`.
3. **Lease TTL Extension**:
   In `src-tauri/src/modules/instance.rs`, extend lease TTL from 90s to `account_lockout_window_minutes * 60` and acquire leases in both default and non-default instance rotation paths.
4. **Auto-Switcher Sibling & Recency Protection**:
   In `src-tauri/src/modules/auto_switcher.rs` `select_candidate_profiles`:
   - Inject `get_active_in_use_account_ids()` into effective exclusions.
   - For sibling instances: check `instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid)`. Never steal accounts bound to actively running instances.
   - Filter candidate accounts by recency: skip accounts where `now - acc.last_used < lockout_window_secs` if bound elsewhere, unless no other accounts remain.

### 3.3 Frontend Table & Card Modernization
1. **Instance Table (`InstanceTable.tsx`)**:
   - Merge `PROFILE NAME` and `BOUND ACCOUNT` into a single high-density column `Profile & Account` (Name + Badges on top, Email with mask toggle on bottom), saving 160px.
   - Truncate `FILE / DATA PATH` to `.../Roaming/Antigravity` with 1-click copy, saving 50px.
   - Move `Prompts` button into the `Actions` button capsule, eliminating the separate `PROMPTS` column and saving 110px.
   - Eliminates horizontal scrollbar across viewports.
2. **4-Column Compact Grid Mode (`Instances.tsx`)**:
   - Reconfigure grid to `grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3.5`.
   - Compact vertical padding and progress bars.
   - Rebuild bottom action buttons into clean, contiguous segmented pill capsules with 5-6px border radius (`rounded-md`).
3. **5-6px Geometry Standard Across Buttons**:
   - Standardize toolbars and action buttons from `rounded-full` / `rounded-xl` to `rounded-md` (5-6px).
   - Fix "Rotate to Next Best" padding (`px-3 py-1.5`), add hover popover/tooltip identifying the targeted instance and quota.
4. **Theme Harmony & Quota Progress Bars**:
   - In `Accounts.tsx`, re-establish subtle divider border styling around the middle `4H / WEEKLY QUOTA` section.
   - Replace neon green with refined VS Code / dark IDE cyan-teal gradients (`from-teal-500 via-cyan-500 to-sky-500`) and soft slate borders.
   - Soften audit buttons and badges from bright amber/green into calm slate/blue/amber dark IDE tones.
