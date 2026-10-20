# Subtask 03: UI/UX Controls & Theme Modernization

**Parent Plan:** `.ai-memory/plans/fleet-system-and-ui-ux-bug-fixes.md`  
**Specification Reference:** `02-spec/21-app/fleet-system-and-ui-ux-bug-fixes/03-migration-and-verification-spec.md`  
**Target Files:**
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src/components/instances/PromptTreeView/DetailTabs.tsx`
- `src/components/instances/PromptTreeView/clipboard.ts`
- `src-tauri/src/commands/instance.rs`
- `src-tauri/src/lib/run.rs`
- `src/services/instanceService/lifecycle.ts`
- `src/pages/instances/InstancesHeader.tsx`
- `src/components/settings/EmailNotifications/SshGitmapSection.tsx`
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
- `src/components/instances/PromptTreeView/InspectorModal.tsx`
- `src/components/instances/PromptTreeView/ResultsView.tsx`

---

## 1. Objectives

1. **Standardize "Copy All (Include Images)" Button**:
   - Update button label from `"Copy All / Includes Image"` to `"Copy All (Include Images)"` in `DetailHeader.tsx`.
   - Maintain triple-MIME clipboard writing (`image/png`, `text/html`, `text/plain`) in `clipboard.ts` with canvas transcoding fallback.
2. **Restore & Relocate Details Button**:
   - Remove the cluttered Details button from the top toolbar row in `DetailHeader.tsx`.
   - Mount it inside `DetailTabs.tsx` adjacent to the View Mode toggles (`Preview`, `Raw`, `Edit`) with an active pulsating blue health dot.
3. **Implement "Start All / Open All Instances" Feature**:
   - Implement `launch_all_instances` in `src-tauri/src/commands/instance.rs` with `AuditTask` tracking.
   - Register the command in `src-tauri/src/lib/run.rs`.
   - Expose `launchAllInstances()` in `src/services/instanceService/lifecycle.ts`.
   - Add a "Start All" action button with stopped-instances counter badge in `InstancesHeader.tsx`.
4. **Modernize SSH Node Action Buttons to Compact Icons**:
   - Replace text buttons (`"Test Connection"`, `"Prompt Machine"`, `"Delete"`) with compact icon buttons (`Play`, `Send`, `Trash2`) in `SshGitmapSection.tsx`.
   - Wrap them in a clean, segmented pill capsule with hover tooltips and `aria-label` accessibility attributes.
5. **Standardize Markdown Container Dark Theme Background**:
   - Change markdown preview background to `dark:bg-[#19191c]` across `PromptInstructionView.tsx`, `InspectorModal.tsx`, and `ResultsView.tsx`.

---

## 2. Step-by-Step Implementation Plan

### Step 1: Standardize "Copy All (Include Images)" Label
- **File:** `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - In primary split-button trigger (line ~582), change text to:
    `<span>{isCopiedAllActive ? 'Copied!' : 'Copy All (Include Images)'}</span>`
  - In dropdown menu Option 1 (line ~725), change text to:
    `<span>{isCopiedAllActive ? 'Copied!' : 'Copy All (Include Images)'}</span>`
- **File:** `src/components/instances/PromptTreeView/clipboard.ts`
  - Ensure `copyPromptWithRichImages` writes triple MIME payload (`image/png`, `text/html`, `text/plain`).
  - Verify error handling routes through `useErrorStore.getState().trackWarning`.

### Step 2: Details Button Relocation with Health Indicator
- **File:** `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - Remove `{/* Segment 4: Details Button */}` block (lines 691–704).
  - Adjust surrounding button border classes to preserve visual encapsulation.
- **File:** `src/components/instances/PromptTreeView/DetailTabs.tsx`
  - Destructure `setIsDetailsModalOpen` from `props`.
  - In the left view-mode container, mount the Details button next to the View Mode toggles.
  - Include the pulsating blue health indicator (`animate-ping` dot + static core dot) and `FileText` icon.

### Step 3: Implement Native "Start All / Open All Instances" Command & UI
- **File:** `src-tauri/src/commands/instance.rs`
  - Add `pub fn launch_all_instances() -> Result<usize, crate::error::AppError>`.
  - Iterate through `instance::list_instances()`, identifying non-running instances (`!inst.is_running`).
  - Invoke `instance::launch_instance(&inst.config.id)` for each, logging failures via `tracing::warn!` without terminating the loop.
  - Wrap in `AuditTask::start(AuditAction::StartInstance, ...)`.
- **File:** `src-tauri/src/lib/run.rs`
  - Register `commands::launch_all_instances` in the `generate_handler!` macro.
- **File:** `src/services/instanceService/lifecycle.ts`
  - Export `launchAllInstances(): Promise<number>` invoking `'launch_all_instances'` with `useErrorStore` tracking.
- **File:** `src/pages/instances/InstancesHeader.tsx`
  - Import `Play` icon and `launchAllInstances`.
  - Add state `isLaunchingAll` (boolean).
  - Implement `handleLaunchAll` handler that calls `launchAllInstances()`, refreshes instances, and triggers a success toast.
  - Render "Start All" button in Segmented Group 1 with stopped instances badge (`instances.length - runningCount`).

### Step 4: SSH Action Buttons Modernization (Text to Icons)
- **File:** `src/components/settings/EmailNotifications/SshGitmapSection.tsx`
  - In remote node table row action cell (lines ~865–918):
    - Replace `"Test Connection"` text button with icon-only `Play` (or `Loader2` during test).
    - Replace `"Prompt Machine"` text button with icon-only `Send`.
    - Replace `"Delete"` text button with icon-only `Trash2` (or `Loader2` during delete).
    - Maintain `rounded-full` segmented pill capsule styling with dividers.
    - Set clear hover `title` and `aria-label` on each button.

### Step 5: Markdown Container Background Color Standardization
- **File:** `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
  - Update Preview container (line ~856) from `dark:bg-[#0c0514]/95` to `dark:bg-[#19191c]`.
  - Update Raw monospace container (line ~905) from `dark:bg-[#0c0514]/95` to `dark:bg-[#19191c]`.
- **File:** `src/components/instances/PromptTreeView/InspectorModal.tsx`
  - Update Markdown preview card (line ~459) from `dark:bg-[#100a1c]` to `dark:bg-[#19191c]`.
- **File:** `src/components/instances/PromptTreeView/ResultsView.tsx`
  - Update Results markdown container (line ~198) from `dark:bg-[#090611]` to `dark:bg-[#19191c]`.

---

## 3. Edge Cases & Safety Gates

1. **Clipboard API Restrictions**:
   - In environments where `ClipboardItem` or triple-MIME is restricted by WebView security policies, `clipboard.ts` must gracefully degrade to dual HTML/Text MIME, then standard text copy, without throwing uncaught exceptions.
2. **Partial Instance Launch Failures**:
   - If one instance fails to start (e.g. port conflict or binary missing), `launch_all_instances` must continue launching subsequent instances, log a warning, and record accurate counts in `AuditTask`.
3. **No Stopped Instances**:
   - When all instances are already running (`runningCount === instances.length`), the "Start All" button in `InstancesHeader.tsx` is automatically hidden, preventing redundant dispatching.
4. **Accessible Icon Buttons**:
   - Icon-only buttons in `SshGitmapSection.tsx` must retain explicit `aria-label` and `title` attributes so screen readers and mouse users understand their exact functionality.

---

## 4. Verification & Testing Checklist

- [ ] **Type Check & Compilation**:
  - `npm run build` passes with zero TypeScript errors.
  - `cd src-tauri && cargo clippy --all-targets --all-features` passes with zero warnings.
- [ ] **Clipboard Label & Behavior**:
  - Verify primary split button displays `"Copy All (Include Images)"`.
  - Verify dropdown option displays `"Copy All (Include Images)"`.
  - Paste into VS Code: verify clean markdown text.
  - Paste into OneNote: verify rich formatting and embedded images.
- [ ] **Details Button Relocation**:
  - Verify Details button is no longer present in `DetailHeader.tsx` top row.
  - Verify Details button is positioned in `DetailTabs.tsx` next to View Mode toggles.
  - Verify blue pulsating indicator dot pulses smoothly.
  - Clicking Details button opens the conversation and instance details modal.
- [ ] **Start All Instances**:
  - With instances stopped, verify "Start All" button is visible and shows correct count badge.
  - Click "Start All" and verify all stopped instances launch.
  - Verify audit task entry created in `task_index.db` under `StartInstance`.
- [ ] **SSH Icon Buttons**:
  - Navigate to Settings -> Email & Notifications -> SSH Gitmap section.
  - Confirm Test, Prompt, and Delete actions render as compact icons in a segmented pill.
  - Confirm hover tooltips appear as expected.
- [ ] **Theme Uniformity**:
  - In dark mode, verify markdown preview in `PromptInstructionView.tsx`, `InspectorModal.tsx`, and `ResultsView.tsx` all render with `#19191c` background.

---

## 5. Definition of Done

- All 5 feature areas implemented and verified against specifications.
- No absolute filesystem paths present in code or documentation.
- All errors properly routed through `useErrorStore` / `AppError`.
- `npm run build` and `cargo clippy` pass clean.
