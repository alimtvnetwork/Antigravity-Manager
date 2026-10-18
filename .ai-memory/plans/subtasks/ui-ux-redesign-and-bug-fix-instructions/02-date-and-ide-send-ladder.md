# Subtask Implementation Plan: Date Formatting, Prompt Tree Visibility & Closed IDE Stabilization Ladder

> **Subtask ID:** `02-date-and-ide-send-ladder`  
> **Parent Slug:** `ui-ux-redesign-and-bug-fix-instructions`  
> **Spec Reference:** [02-spec/21-app/ui-ux-redesign-and-bug-fix-instructions/02-date-ide-send-spec.md](../../../../02-spec/21-app/ui-ux-redesign-and-bug-fix-instructions/02-date-ide-send-spec.md)  
> **Target Files:**
> - `src/utils/date.ts`
> - `src/types/config.ts`
> - `src/components/settings/GeneralSettingsTab.tsx`
> - `src/pages/Settings/tabs/GeneralTab.tsx`
> - `src-tauri/capabilities/default.json`
> - `src/components/instances/PromptTreeView/clipboard.ts`
> - `src/components/instances/PromptTreeView/PromptMarkdownPreview.tsx`
> - `src/components/instances/PromptTreeView/conversationGuards.ts`
> - `src-tauri/src/modules/repo_db/tree_phases.rs`
> - `src-tauri/src/modules/instance/smart_run.rs`
> - `src-tauri/src/modules/repo_db/agy.rs`  
> **Status:** `[QUEUED]`  
> **Author:** `@aukgit`  

---

## 1. Context & Objectives

During end-to-end user testing of Antigravity-Manager, three critical functional defects were identified:
1. **Timestamp Inconsistencies & Clutter:** Users require timestamps formatted as `DD-MMM-YYYY HH:mm` (e.g. `18-Oct-2026 21:40`) in 24-hour clock using the local Windows timezone, configurable via Settings. Duration displays must suppress seconds when hours or minutes exist (e.g., `18h 27m` instead of `18h 27m 14s`). Relative time indicators (e.g., "ran 5 minutes ago", "ran 1 day ago") must accompany displayed timestamps. All database stores must strictly preserve ISO 8601 UTC0 / Unix epoch integers.
2. **Prompt Tree Image Blanks & Missing Queued/Running Projects:** Images embedded in markdown prompt nodes fail to render because Tauri v2 lacks `"core:asset:default"` capability and relative image paths are not resolved against project repository roots. Furthermore, when prompts are running or queued, an empty left sidebar occurs when `tree_phases.rs` only scans `conversation_summaries.db`, and `conversationGuards.ts` inadvertently marks queued prompts as stale.
3. **Closed IDE Send Failure:** When prompts are dispatched to a closed IDE, the system opens the application but immediately attempts dispatch after a rigid 800ms pause, causing prompts to get lost during Electron window initialization. An adaptive 5-second stabilization ladder is required to confirm process liveness and window readiness before dispatching prompts.

---

## 2. Step-by-Step Implementation Plan

### Step 1: Centralized Date Formatting & Duration Logic (`src/utils/date.ts`)
1. **Format Date-Time (`formatDateTime`):**
   - Accept `value?: number | string | Date | null` and optional `formatMode?: '24h' | '12h' | 'iso'`.
   - Default to `24h`: format as `${day}-${month}-${year} ${hours}:${minutes}` (e.g. `18-Oct-2026 21:40`), four-digit year, 24-hour hour without AM/PM.
   - For `12h`: format as `${day}-${month}-${year} ${hours12}:${minutes} ${ampm}` (e.g. `18-Oct-2026 09:40 PM`).
   - For `iso`: format as localized ISO string.
   - Month representation: 3-character uppercase abbreviation (`Jan`, `Feb`, `Mar`, `Apr`, `May`, `Jun`, `Jul`, `Aug`, `Sep`, `Oct`, `Nov`, `Dec`).
   - Use client's local timezone (standard `Date` getMethods or `Intl.DateTimeFormat`).
2. **Relative Time Calculation (`formatRelativeTime`):**
   - Implement `formatRelativeTime(value?: number | string | Date | null, options?: { prefixRan?: boolean }): string`.
   - Calculate `deltaMs = Date.now() - timestampMs`.
   - Format outputs:
     - `deltaMs < 45_000` -> `"just now"` (or `"ran just now"`)
     - `deltaMs < 90_000` -> `"1 minute ago"` (or `"ran 1 minute ago"`)
     - `deltaMs < 45 * 60_000` -> `${Math.round(deltaMs / 60_000)} minutes ago`
     - `deltaMs < 90 * 60_000` -> `"1 hour ago"`
     - `deltaMs < 24 * 3600_000` -> `${Math.round(deltaMs / 3600_000)} hours ago`
     - `deltaMs < 48 * 3600_000` -> `"1 day ago"`
     - `deltaMs < 30 * 86400_000` -> `${Math.round(deltaMs / 86400_000)} days ago`
     - Otherwise -> `${Math.round(deltaMs / (30 * 86400_000))} months ago`
3. **Duration Second Suppression (`formatDuration`):**
   - Standardize `formatDuration(secs: number): string`:
     - If `secs < 60`: return `${secs}s` (e.g., `45s`, `0s`)
     - If `secs >= 60 && secs < 3600`:
       - `mins = Math.floor(secs / 60)`
       - return `${mins}m` (Omit seconds!)
     - If `secs >= 3600`:
       - `hrs = Math.floor(secs / 3600)`
       - `mins = Math.floor((secs % 3600) / 60)`
       - If `mins === 0`: return `${hrs}h`
       - Otherwise: return `${hrs}h ${mins}m` (Omit seconds!)
4. **Export Utilities:** Export `formatDateTime`, `formatDateOnly`, `formatRelativeTime`, and `formatDuration`.

---

### Step 2: Settings Configuration & Persistence (`src/types/config.ts`, `GeneralSettingsTab.tsx`, `GeneralTab.tsx`)
1. **Config Definition in `src/types/config.ts`:**
   - Add `date_format_mode?: '24h' | '12h' | 'iso';` to `AppConfig`.
2. **Settings Component (`src/components/settings/GeneralSettingsTab.tsx`):**
   - Create or update `GeneralSettingsTab.tsx` providing a unified section:
     - Label: "Date & Time Format"
     - Subtitle: "Choose the display format for timestamps across prompt trees and telemetry."
     - Dropdown / Segmented pill selector for `'24h'`, `'12h'`, `'iso'`.
     - Live preview capsule showing formatted current time.
     - Auto-save via `saveConfig({ ...config, date_format_mode: selected })`.
3. **Integration in `src/pages/Settings/tabs/GeneralTab.tsx`:**
   - Embed or import the Date Format section into `GeneralTab.tsx`.

---

### Step 3: Tauri Asset Capabilities & Relative Image Resolution
1. **Tauri Capabilities (`src-tauri/capabilities/default.json`):**
   - Add `"core:asset:default"` to `"permissions"` list.
   - Verify desktop schema validity.
2. **Image Path Resolution (`src/components/instances/PromptTreeView/clipboard.ts`):**
   - Update `resolveImageSrc(src: string, repoPath?: string): string`.
   - If `src` is not an absolute URI (`http`, `data:`, `blob:`, `file:`, `/`, or `X:\`), resolve against `repoPath`:
     - `cleanPath = `${repoPath.replace(/[/\\]+$/, '')}/${cleanPath.replace(/^[./\\]+/, '')}``
   - Pass resolved absolute path into `convertFileSrc(cleanPath)`.
3. **Markdown Rendering (`src/components/instances/PromptTreeView/PromptMarkdownPreview.tsx`):**
   - Add `repoPath?: string` to `PromptMarkdownPreviewProps`.
   - In token parser, wrap markdown image URLs with `resolveImageSrc(imgSrc, repoPath)`.
   - Wire `repoPath` from parent components (`PromptInstructionView.tsx`, `PromptTreeViewModal.tsx`).

---

### Step 4: Queued & Running Prompt Tree Visibility
1. **Conversation Guard Protection (`src/components/instances/PromptTreeView/conversationGuards.ts`):**
   - In `isStaleOrEmptyConversation(conv: AgmConversationNode)`:
     - Check: `if (Boolean(conv.is_running) || Boolean(conv.is_queued)) return false;`
     - Guarantee that queued prompts with content are protected and never collapsed or filtered out.
2. **Active Prompts Synthesis in Backend (`src-tauri/src/modules/repo_db/tree_phases.rs`):**
   - In `discover_fallback_projects`:
     - Query `active_prompts` in `repo_prompts.db`:
       ```sql
       SELECT instance_id, repo_path, prompt_content, status, updated_at
       FROM active_prompts
       WHERE status IN ('running', 'in_flight', 'dispatched', 'queued')
       ORDER BY created_at ASC, id ASC
       ```
     - For each row, check if `(norm_inst, normalize_path_for_compare(&repo_path))` is in `seen_fb`.
     - If not seen, synthesize a `RunningProject` with `is_running: status == "running" || status == "in_flight"`.
     - Insert into `seen_fb` and append to `projects`.
     - Prevents empty sidebar when active/queued prompts exist.

---

### Step 5: Closed IDE Adaptive 5-Second Stabilization Ladder
1. **Adaptive Ladder in `src-tauri/src/modules/instance/smart_run.rs`:**
   - In `ensure_instance_running_smart(instance_id: &str, workspace_path: Option<&str>)`:
     - If the instance is confirmed offline:
       - Log cold launch initiation.
       - Launch the instance process.
       - Execute adaptive stabilization ladder:
         - Initial pause: 1,000ms.
         - Loop up to 8 iterations with 500ms intervals (max 5,000ms total budget).
         - In each iteration: refresh process cache; call `is_instance_process_running_smart(&canonical_id)`.
         - When primary PID is verified alive, check window readiness or DevTools availability.
         - Break early when process is confirmed stable.
       - Return result indicating that cold launch occurred and elapsed stabilization time.
2. **Send Dispatch Coordination in `src-tauri/src/modules/repo_db/agy.rs`:**
   - In `send_prompt_now_for_instance`:
     - Inspect cold-start status from `ensure_instance_running_smart`.
     - If cold-started, add a 1,500ms settlement pause to allow IDE workbench extensions to load before dispatch.
     - Write `.antigravity_resume_task.json` snapshots.
     - Call `spawn_or_replace_prompt_via_agy` with retry fallback to CDP insertion.
     - Log full diagnostic telemetry: `[ColdStartDispatch] Instance '{}' ready; prompt dispatched successfully`.

---

## 3. Disjoint Target File Ownership

| File Path | Responsibility |
|---|---|
| `src/utils/date.ts` | Centralized `formatDateTime` (24h/12h), `formatRelativeTime`, and `formatDuration` (second suppression) |
| `src/types/config.ts` | Add `date_format_mode` to `AppConfig` |
| `src/components/settings/GeneralSettingsTab.tsx` | UI setting toggle for date format with live preview |
| `src/pages/Settings/tabs/GeneralTab.tsx` | Embed date format selector in General Settings page |
| `src-tauri/capabilities/default.json` | Add `core:asset:default` permission for Tauri v2 WebView asset loading |
| `src/components/instances/PromptTreeView/clipboard.ts` | Resolve relative image paths against `repoPath` in `resolveImageSrc` |
| `src/components/instances/PromptTreeView/PromptMarkdownPreview.tsx` | Pass `repoPath` into `resolveImageSrc` for markdown image tokens |
| `src/components/instances/PromptTreeView/conversationGuards.ts` | Protect `is_queued` and `is_running` conversations from stale classification |
| `src-tauri/src/modules/repo_db/tree_phases.rs` | Synthesize `RunningProject` entries from `active_prompts` in fallback discovery |
| `src-tauri/src/modules/instance/smart_run.rs` | Implement adaptive 5-second process stabilization ladder on cold start |
| `src-tauri/src/modules/repo_db/agy.rs` | Coordinate stabilization delay before prompt dispatch on cold-started IDEs |

---

## 4. Edge Cases & Resiliency Matrix

1. **Relative Path Traversal in Images:**
   - If markdown specifies `../../image.png`, ensure normalization does not escape outside the project root without bounds checking.
2. **Corrupted Timestamps:**
   - `formatDateTime` and `formatRelativeTime` must handle `NaN`, negative numbers, unparseable strings, and `undefined` by returning fallback hyphen `"-"`.
3. **IDE Launch Failures on Windows:**
   - If an IDE executable fails to launch due to Job Object or binary path issues, the 5-second stabilization ladder times out gracefully and returns an informative `AppError::Process` rather than panicking or deadlocking.
4. **Rapid Multiple Sends:**
   - Multiple prompt dispatches in quick succession must honor the FIFO queue in SQLite (`ORDER BY created_at ASC, id ASC`), ensuring subsequent prompts are queued cleanly without colliding with cold-start stabilization.

---

## 5. Verification Checklist

- [ ] `formatDateTime` formats `18-Oct-2026 21:40` in 24-hour mode.
- [ ] `formatDuration(120)` outputs `2m` (no seconds).
- [ ] `formatDuration(3665)` outputs `1h 1m` (no seconds).
- [ ] `formatDuration(42)` outputs `42s`.
- [ ] `formatRelativeTime` outputs `"ran 5 minutes ago"` for a 5-minute delta.
- [ ] Settings tab toggle changes the displayed format and persists in `gui_config.json`.
- [ ] Markdown preview renders local images via `asset://` protocol using `core:asset:default`.
- [ ] Relative markdown images (e.g. `screenshots/1.png`) resolve against `repo_path`.
- [ ] Prompt tree left panel shows projects when prompts are queued or in-flight.
- [ ] Queued prompts are never marked as stale or empty.
- [ ] Cold IDE launch waits up to 5 seconds for process and window stabilization before dispatching prompts.
