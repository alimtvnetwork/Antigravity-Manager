# Subtask 03: Telegram Inbound Backup, Multi-Image Composite PNG Clipboard, SSH Settings Icons, 30s Sync Dropdown Modernization, and Start All Instances

- **Subtask ID:** `03-telegram-backup-image-copy-and-controls`
- **Parent Plan:** `.ai-memory/plans/telegram-backup-and-fleet-management-enhancements.md`
- **Specification Reference:** `02-spec/21-app/telegram-backup-and-fleet-management-enhancements/03-migration-and-verification-spec.md`
- **Status:** QUEUED
- **Author:** System Architecture & Fleet Verification Working Group

---

## 1. Objectives & Intent

Execute the engineering implementation and verification protocol for:
1. Expanding Telegram inbound prompt backup routing to recognize `/backup`, `/backup_prompt`, `/backup_prompts`, `/backup_running`, `/brp`, parsing target instances dynamically, querying multimodal prompt images, and formatting rich HTML reports.
2. Eliminating WebView2 canvas cross-origin security exceptions (`Tainted canvases may not be exported`) by resolving relative image paths against `repoPath`, fetching via native backend `read_image_as_base64`, bounding composite canvas dimensions, and upgrading PowerShell clipboard writes with a 10-retry overload (`SetDataObject($data, $true, 10, 100)`).
3. Modernizing remote instance table actions in `SshGitmapSection.tsx` from verbose text (`<span>Prompt This Instance</span>`) into a compact 3-button segmented pill capsule (`Send`, `Play`, `Copy`) with descriptive tooltips.
4. Upgrading `ModalHeader.tsx` Segment 4 with interactive hover transitions and a live emerald pulsing dot indicator (`animate-ping`) when auto-sync is active.
5. Permanently mounting the `Start All` button in `InstancesHeader.tsx` in a disabled state when all instances are running to eliminate toolbar layout shifts.

---

## 2. Owned Files Breakdown

| File Path | Subsystem | Action Type | Responsibility |
| :--- | :--- | :--- | :--- |
| `src-tauri/src/modules/telegram_inbound/processing.rs` | Telegram Inbound | Modification | Broaden top-level match ladder and conversational parser for all backup command aliases. |
| `src-tauri/src/modules/telegram_inbound/commands.rs` | Telegram Inbound | Modification | Implement dynamic instance extraction, multimodal image querying, and structured Telegram HTML reporting. |
| `src/components/instances/PromptTreeView/clipboard.ts` | Frontend Clipboard | Modification | Resolve relative attachment URIs against `repoPath`, read via `read_image_as_base64`, and bound canvas dimensions. |
| `src-tauri/src/commands/cmd_files.rs` | Native Clipboard | Modification | Upgrade Windows Forms `SetDataObject` to 4-argument 10-retry overload (`10, 100`). |
| `src/components/settings/EmailNotifications/SshGitmapSection.tsx` | SSH Settings UI | Modification | Replace verbose text button with compact segmented pill capsule (`Send`, `Play`, `Copy`) with tooltips. |
| `src/components/instances/PromptTreeView/ModalHeader.tsx` | Tree View Header | Modification | Add hover styling to Segment 4 and inject live emerald pulsing dot indicator when auto-sync is enabled. |
| `src/pages/instances/InstancesHeader.tsx` | Instances Toolbar | Modification | Permanently mount `Start All` button in disabled state when all instances are running. |

---

## 3. Step-by-Step Task Breakdown

### Task 1: Telegram Inbound Backup Command Route Expansion
- **File:** `src-tauri/src/modules/telegram_inbound/processing.rs`
- **Implementation:**
  1. Expand command matching ladder:
     ```rust
     "backup" | "backpack" | "backup_prompt" | "backup_prompts" | "backup_running" | "brp" => {
         Some(execute_backup_command(rest))
     }
     ```
  2. Add natural language conversational fallbacks in the wildcard matching section:
     ```rust
     if lower_full.starts_with("backup prompt")
         || lower_full.starts_with("backup prompts")
         || lower_full.starts_with("backup running")
         || lower_full.starts_with("brp ")
     {
         let sub = lower_full.trim();
         return Some(execute_backup_command(sub));
     }
     ```

### Task 2: Dynamic Instance Parsing, Multimodal Image Querying, & Rich HTML Formatting
- **File:** `src-tauri/src/modules/telegram_inbound/commands.rs`
- **Implementation:**
  1. In `pub fn execute_backup_command(args_str: &str) -> String`:
     - Tokenize `args_str`:
       - Handle subcommands `ls`, `list`, `status` with batch summaries.
       - Handle subcommands `restore`, `rrp`, `resend` with dynamic instance extraction.
       - Default action: extract target instance from arguments (e.g. `brp default`, `/backup custom-inst`). Default to `"default"` if empty.
  2. Call `repo_db::backup_running_prompts(&target_inst)` and `backup_prompts_db::backup_active_running_prompts(Some(&target_inst), None)`.
  3. Extract multimodal attachments:
     - Iterate through returned `PromptBackupRecord` list.
     - Inspect `p.has_images` and `p.images_payload`.
     - Count total attached images across backed up prompts.
  4. Generate rich HTML report via `clean_for_telegram_html`:
     - Report target instance, short batch ID (8 chars), total prompt count, multimodal image count, and ISO timestamp.
     - Append italicized hint instructing users on `/restore <instance>` syntax.

### Task 3: Multi-Image Composite PNG Clipboard Engine
- **File:** `src/components/instances/PromptTreeView/clipboard.ts`
- **Implementation:**
  1. Update `loadImageElement(uri: string, repoPath?: string): Promise<HTMLImageElement | null>`:
     - Check if `uri` is a relative path; if so and `repoPath` is provided, combine with proper path delimiter.
     - If local file exists, call `invoke<string>('read_image_as_base64', { path: resolvedPath })` to obtain an untainted base64 data URI before loading into `new Image()`.
  2. Update `stitchImagesToCompositePng(imageUris: string[], repoPath?: string): Promise<Blob | null>`:
     - Pass `repoPath` to `loadImageElement`.
     - Enforce canvas dimension bounds:
       ```typescript
       const MAX_COMPOSITE_WIDTH = 4096;
       const MAX_COMPOSITE_HEIGHT = 16384;
       ```
     - If total stitched height exceeds `MAX_COMPOSITE_HEIGHT`, proportionally scale down drawing dimensions preserving aspect ratio.
  3. Update `copyAllImagesToClipboard(imageUris: string[], repoPath?: string, onSuccess?, onError?)`:
     - Pass `repoPath` through to `stitchImagesToCompositePng`.

### Task 4: PowerShell Clipboard 10-Retry Overload
- **File:** `src-tauri/src/commands/cmd_files.rs`
- **Implementation:**
  1. In `copy_image_to_clipboard(path: String)` under `#[cfg(target_os = "windows")]`:
  2. Update PowerShell command script:
     ```powershell
     [System.Windows.Forms.Clipboard]::SetDataObject($data, $true, 10, 100);
     ```
  3. Replace the single-attempt call with this 4-argument overload (10 retries, 100ms pause) to eliminate `0x800401D0` external lock exceptions.

### Task 5: SSH Settings Segmented Pill & Tooltip Icons
- **File:** `src/components/settings/EmailNotifications/SshGitmapSection.tsx`
- **Implementation:**
  1. Replace the verbose `<span>Prompt This Instance</span>` button in the remote instance table with a compact dark-glass segmented pill:
     - Segment 1: `Send` icon button (`title="Prompt This Instance"`). Sets target and focuses `#remote-prompt-input` or dispatches immediately if prompt text is present.
     - Segment 2: `Play` icon button (`title="Launch / Run Instance"`). Dispatches `agm instances start <id>` or triggers instance run.
     - Segment 3: `Copy` icon button (`title="Copy Target Specifier & SSH Command"`). Copies `<alias>:<id>` and formatted `agm ssh run ...` command to clipboard.
  2. Maintain responsive table layout without line wrapping or horizontal overflow.

### Task 6: Sync 30s Dropdown UI Modernization & Pulsing Indicator
- **File:** `src/components/instances/PromptTreeView/ModalHeader.tsx`
- **Implementation:**
  1. In Segment 4, apply interactive hover styles matching adjacent toolbar buttons:
     `hover:bg-slate-200/80 dark:hover:bg-[#40285e]/90 transition-colors cursor-pointer rounded-none`
  2. When `syncInterval !== 'off'`, render a pulsing green dot indicator:
     ```tsx
     {syncInterval !== 'off' && (
         <span className="relative flex h-2 w-2 mr-0.5">
             <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
             <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
         </span>
     )}
     ```
  3. Preserve transparent borderless inline `<select>` styling.

### Task 7: Permanently Mounted Start All Instances Button
- **File:** `src/pages/instances/InstancesHeader.tsx`
- **Implementation:**
  1. Remove `{instances.length - runningCount > 0 && ...}` conditional unmounting guard.
  2. Compute `stoppedCount = Math.max(0, instances.length - runningCount)`.
  3. Permanently mount the `Start All` button in the segmented pill capsule.
  4. When `stoppedCount === 0`:
     - Apply `disabled={isLoading || isLaunchingAll || stoppedCount === 0}`.
     - Apply `disabled:opacity-40 disabled:cursor-not-allowed` styles.
     - Set `title="All instances are currently running"`.
     - Render badge showing `0`.

---

## 4. Invariants & Implementation Guardrails

1. **No Silent Errors Invariant (`AGENTS.md`)**:
   - In Rust, never swallow errors with bare `let _ = ...` or unrecorded `.ok()`. Route errors to `record_ignored(result, "action_name")` accompanied by a `// Justification:` comment explaining why continuation is safe.
   - In TypeScript, wrap asynchronous Tauri invocations and clipboard calls with `useErrorStore.getState().trackWarning(err, { source, triggerAction })`.
2. **Strictly Relative Git Paths Invariant**:
   - Zero absolute filesystem paths in specs, plans, code comments, commit messages, or output.
3. **FIFO Ordering for SQLite Prompt Queries**:
   - All prompt restore, backup listing, and queue processing queries across SQLite databases must enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`.
4. **Minimalist & Contextual UI Invariant**:
   - Wrap adjacent toolbar controls into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, dark-glass styling). Inline selects must remain borderless and transparent.

---

## 5. Verification Protocol & Quality Gates

### Step 1: Pre-flight Code Compilation & Linting Gates
```bash
# Rust code formatting gate
cd src-tauri && cargo fmt -- --check

# Rust compiler and clippy gate
cd src-tauri && cargo clippy --all-targets --all-features

# Frontend TypeScript and bundle build gate
npm run build
```

### Step 2: Subsystem Verification Scenarios
1. **Telegram Inbound Backup**: Verify that `/backup`, `/backup_prompt`, `/backup_prompts`, `/backup_running`, and `/brp` return rich HTML reports with instance names, batch IDs, prompt counts, and multimodal image counts.
2. **Multi-Image Composite PNG Clipboard**: Verify that clicking "Copy All Images" in PromptTreeView on prompts containing multiple relative image attachments does not throw canvas tainting errors, and pastes cleanly as genuine binary PNG into OneNote or Paint.
3. **SSH Settings Tooltip Icons**: Verify that the remote instance table renders the compact segmented pill capsule with Send, Play, and Copy icons, tooltips appear on hover, and each action executes reliably.
4. **Sync 30s Dropdown**: Verify that selecting 30s auto-sync activates the green pulsing dot indicator, and hover transitions render smoothly.
5. **Start All Instances Button**: Verify that when all instances are started, the button remains permanently mounted in the toolbar in a disabled state without layout shifts.

### Step 3: GitMap Atomic Commit Gate
Commit changes using GitMap standard hyphen format:
```bash
gitmap cpb "telegram - prompt backup multimodal clipboard and control modernization"
```
*(Worker rule: Only execute commit via the designated release orchestrator / lead agent).*
