# 101 — Instance Clone Settings & Projects, Audit Dark Theme, and UI Enhancements

## User Request (Verbatim)

> Can you please check the recent releases from AGM, this package root cause analysis? So now it can actually do the issues of the previous-- there's a conversation that was running the prompt. It can do that now. So try to understand the root cause, how it fixed it, and also write the root cause and update the AI memory folder so that any AI who reads it, they'll understand how to fix it in the future, these type of things, and this should never be happening again. Okay? Think about that. And also at the same time, there are several issues that you need to fix. For example, the email section, try to have a little bit more. Okay? In the display. The next thing is the settings About section. It takes four. Try to make it better. Okay? Make it better, compact so that it looks and follows the UI/UX concepts and principles. The next thing is that only it looks white. It does not show as the proper dark theme, so we need to fix the audit section. Also at the same time, the... So instance copy has an issue, serious issue. When we do the instance copy, first of all, the UI that comes, that is not really very professional, okay? Because of the text, the alignment and everything. Make it professional. Add more CSS3 animations. Make it look nice, okay? Make these buttons Cancel and Duplicate. I think these set of buttons are repeated many places. These buttons looks very, very poor. Try to make it compact, colorful. Have a CSS3 animation buttons. The next thing is, which is the biggest issue, when we clone that instance, the settings copied a little bit, but also at the same time, the projects are not added. That's one issue. The settings are not copied. That means, let's say, if we have the settings for the read folder, and also browser settings, all kinds of things, these actually remains not copied properly. Okay? So the color theme, most of the things are not copied properly. So I think this is a very serious issue that you need to work on. For example, if I put two instances, you can see the theme differences when it was copied, which makes it a very serious case, I think. You need to fix it properly. Also try to understand the previous codes, how it was done, how it actually solved a lot of other issues. And also when an item is highlighted or selected, it's actually green within green. It looks really, really bad. I think what I requested is a little bit dark color, blackish dark. And the highlighted one will be a little bit of yellow text so that it looks a distinguished item. Okay? So this is kind of missing. Antigravity About section improved. Audit color and other stuff fixed. Okay? Then the buttons needs to be improved, like the instance button section. Also, the instance duplicate section or copy section UI needs to be improved a lot. It looks terrible at the moment. Okay, so implement everything. First write the spec and verbally what I just said, and then start implementing. Okay? Do you understand? Make sure that you understand the previous code, how it was changed, how it does not have any bug. At the end, I want you to check CICD until it finishes as green. So you can use Git map PE-T, something like this. And also you should make a minor bump after the changes are done. Quick release node and do the bumping and everything, wherever the changes is required, apply that. Is it clear?

---

## 1. Running Prompt Resumption RCA & Prevention
- **Root Cause**:
  1. Discovery rejected conversations if `transcript.jsonl` contained only assistant steps without explicit `USER_INPUT` entries, leaving `active_prompts` empty.
  2. Missing `status` column in older `conversation_summaries` table schemas caused unhandled SQL errors, skipping whole database scans.
  3. Named instances falling back to global `.gemini` paths dropped records when the workspace path had not yet been registered in `running_projects`.
  4. Prompt re-dispatch dropped `session_id`/`conversation_id` when writing `.antigravity_resume_task.json`, preventing IDE from connecting to the previous session.
  5. The audit snapshot query sorted strictly by `created_at` with limit 50, omitting recently updated older prompts.
  6. Process ID checking continually scanned full process tables instead of verifying known PIDs, and macOS open wrappers hid child PIDs.
- **Remediation & Safeguards**:
  - Store preview text when no `USER_INPUT` is found; fallback gracefully on missing status columns.
  - Retain `session_id` and `conversation_id` in `.antigravity_resume_task.json`.
  - Maintain comprehensive RCA logs in `.ai-memory/issues/61-running-prompt-resumption-and-conversation-continuity-rca.md` and `02-spec/22-app-issues/30-running-prompt-resumption-and-conversation-continuity-rca.md`.

---

## 2. Instance Cloning: Deep Copy of Settings, Themes, and Projects
- **Root Cause in Instance Copy**:
  - `copy_instance` copied only `data_dir/User` to `target_data_dir/User`. On Windows, instances launch with `APPDATA=instances/<id>/home/AppData/Roaming`. The IDE saves theme configurations, read folder preferences, and browser parameters in `%APPDATA%\Antigravity\User\settings.json`.
  - Copying from `default` or between named instances failed to mirror `%APPDATA%\Antigravity\User\settings.json` into the target instance's home directory.
  - Workspace databases and project registrations: `running_projects` rows were only copied if already populated with matching `instance_id`. Workspaces in `workspaceStorage` were not actively detected before cloning, and default instance projects mapped with `__default__` or `default` were missed.
- **Specification Fix**:
  - In `copy_instance`:
    1. Pre-scan and detect source projects via `detect_running_projects` before cloning.
    2. Deep copy `User/settings.json`, `User/keybindings.json`, `User/snippets/`, `User/globalStorage/`, and `User/workspaceStorage/`.
    3. On Windows, copy all IDE configuration files from `source_home/AppData/Roaming/Antigravity/User/` to `target_home/AppData/Roaming/Antigravity/User/` and into `target_data_dir/User/`.
    4. Ensure `workbench.colorTheme`, read folder configs, and browser settings in `settings.json` are preserved.
    5. In `clone_instance_repo_rows`: Clone all projects whether mapped to `default` or `__default__`, ensuring the target instance inherits all registered projects and prompts.

---

## 3. Duplicate / Clone Profile Modal & CSS3 Animated Buttons
- **Root Cause**:
  - In `InstanceSelector.tsx` and `Instances.tsx`, the duplicate modal buttons (`Cancel`, `Duplicate`) were unstyled text buttons lacking backgrounds, borders, active states, and animations. The modal layout was basic and cramped.
- **Specification Fix**:
  - Redesign modal dialog:
    - Clean card surface with subtle glow and backdrop blur.
    - Polished scope radio cards (`IDE Copy (Full Environment)` vs `Profile Copy (Preferences)`).
    - Compact, colorful CSS3 animated buttons:
      - **Cancel Button**: Modern ghost/glass button with rounded-xl border, hover background transitions, subtle scale effect, and clear typography.
      - **Duplicate Button**: Vibrant gradient button (indigo-to-cyan/blue) with CSS3 shimmer animation, active push transform (`active:scale-95`), loading spinner support, and high-visibility contrast.

---

## 4. Audit Section Dark Theme Fix
- **Root Cause**:
  - `src/pages/Audit.tsx` had hardcoded `bg-white`, `border-slate-100`, `bg-slate-50`, `text-slate-950`, `text-slate-700`, `border-slate-300` across containers, tables, modals, and pagination buttons without `dark:` classes.
- **Specification Fix**:
  - Add complete dark theme support across all elements:
    - Container & cards: `bg-white dark:bg-[#0c2438] border-gray-100 dark:border-[#15334d]`.
    - Headings & text: `text-slate-900 dark:text-gray-100`, subtitle: `text-slate-500 dark:text-gray-400`.
    - Table header: `bg-slate-50 dark:bg-[#071a27] text-slate-700 dark:text-gray-300 border-slate-200 dark:border-[#15334d]`.
    - Table rows: hover states `hover:bg-slate-50/60 dark:hover:bg-[#15334d]/40`, dividers `border-slate-100 dark:border-[#15334d]/60`.
    - Audit detail modal: `bg-white dark:bg-[#0c2438] border-gray-200 dark:border-[#15334d] text-slate-900 dark:text-gray-100`.
    - Pagination buttons: `bg-white dark:bg-[#0c2438] border-slate-300 dark:border-[#15334d] text-slate-800 dark:text-gray-200 hover:bg-slate-50 dark:hover:bg-[#15334d]`.

---

## 5. Settings About Section Compact Redesign
- **Root Cause**:
  - `src/pages/Settings.tsx` rendered the About section with sprawling vertical elements, large blank gaps, oversized buttons, and disconnected card sections.
- **Specification Fix**:
  - Re-architect into a cohesive, compact card following modern UI/UX design tokens:
    - Header: App icon, title, version pill badge, subtitle and runtime stack tags in a single compact row.
    - Action bar: Compact pill links for author, telegram, repository, and sponsor with icons and subtle hover animations.
    - Update management card: Integrated horizontal card combining update channel selector (Stable / Preview) with "Check for Updates" button and inline status indicator.
    - Footer: Compact copyright and release notes link.

---

## 6. Selected / Current Account Row High Contrast
- **Root Cause**:
  - In `AccountTable.tsx` / `AccountRow.tsx`, the currently active account (`isCurrent`) used emerald green background (`bg-emerald-50/60 dark:bg-[#0c2438]`) with green badge (`text-[#43d6a2]`, `bg-[#16a97a]/25`), causing clashing "green-on-green" low contrast.
- **Specification Fix**:
  - Background: Dark blackish styling (`bg-slate-900/90 dark:bg-[#050d14]`) with distinguished border (`border-amber-400/40 dark:border-amber-400/30`).
  - Text: Distinguished yellow / gold email and name (`text-amber-300 dark:text-amber-300 font-bold`).
  - Badge: Amber CURRENT badge (`bg-amber-400/15 text-amber-300 border border-amber-400/30 font-bold`).

---

## 7. Email Section Display Enhancement
- **Specification Fix**:
  - Expand `src/pages/Email.tsx`:
    - Add connection status badges (IMAP Listener & SMTP Service active status).
    - Summary metrics bar: active watchers count, node identity, IP address, and security vault health.
    - Quick actions and informative service telemetry cards.
