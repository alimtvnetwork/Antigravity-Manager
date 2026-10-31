# Migration, UI/UX Controls & Verification Specification

> **Specification Reference:** `02-spec/21-app/fleet-system-and-ui-ux-bug-fixes/03-migration-and-verification-spec.md`  
> **Parent Initiative:** `fleet-system-and-ui-ux-bug-fixes`  
> **Author:** `@aukgit`  
> **Status:** `[DRAFT / APPROVED]`

---

## 1. Overview & Architectural Scope

This specification establishes the migration protocols, UI/UX control modernizations, theme standardization, backend command contracts, and comprehensive verification checklist for the **Fleet System & UI/UX Bug Fixes** milestone in Antigravity-Manager (AGM).

The scope encompasses 5 targeted improvements across frontend UI components, clipboard workflows, backend instance orchestration, and visual theme consistency:

1. **"Copy All (Include Images)" Button Label & Multi-MIME Standardization**:
   - Standardize primary toolbar split-button label and menu items to `"Copy All (Include Images)"` in `src/components/instances/PromptTreeView/DetailHeader.tsx`.
   - Maintain triple-MIME payload composition (`image/png`, `text/html`, `text/plain`) in `src/components/instances/PromptTreeView/clipboard.ts` with canvas transcoding to guarantee compatibility across VS Code, Cursor, Microsoft OneNote, and external web paste targets.
2. **Details Button Restoration & Relocation**:
   - Decouple the "Details" action button from the crowded top header row of `src/components/instances/PromptTreeView/DetailHeader.tsx`.
   - Mount the restored "Details" button cleanly inside the secondary control strip of `src/components/instances/PromptTreeView/DetailTabs.tsx` adjacent to the View Mode toggles (`Preview`, `Raw`, `Edit`), adorned with an active pulsating health indicator for real-time visual feedback.
3. **"Start All / Open All Instances" Orchestration**:
   - Implement native backend command `launch_all_instances` in `src-tauri/src/commands/instance.rs` with task history audit tracking (`AuditAction::StartInstance`).
   - Register command in Tauri dispatch table at `src-tauri/src/lib/run.rs`.
   - Expose strongly typed client wrapper in `src/services/instanceService/lifecycle.ts`.
   - Render a high-visibility `"Start All"` / `"Open All"` button equipped with a stopped-instance counter badge in `src/pages/instances/InstancesHeader.tsx`.
4. **SSH Settings UI Text-to-Icon Modernization**:
   - Modernize the action column inside `src/components/settings/EmailNotifications/SshGitmapSection.tsx`.
   - Transform textual buttons (`"Test Connection"`, `"Prompt Machine"`, `"Delete"`) into a compact, segmented dark-glass capsule composed of icon-only action triggers (`Play`, `Send`, `Trash2`) with contextual hover tooltips.
5. **Markdown Default View Background Color Standardization**:
   - Standardize the dark theme background color of markdown viewing containers to `dark:bg-[#19191c]`.
   - Align across `src/components/instances/PromptTreeView/PromptInstructionView.tsx`, `src/components/instances/PromptTreeView/InspectorModal.tsx`, and `src/components/instances/PromptTreeView/ResultsView.tsx`.
6. **Testing and Verification Checklist**:
   - Define exact automated pre-flight checks, type-checking passes, clippy gates, and manual UI validation procedures.

---

## 2. UI/UX Controls & Layout Architecture

### 2.1 "Copy All (Include Images)" Button & Multi-MIME Clipboard

#### 2.1.1 Label Normalization
In `src/components/instances/PromptTreeView/DetailHeader.tsx`, the split-button label is currently rendered as `"Copy All / Includes Image"`. This must be normalized to `"Copy All (Include Images)"` across:
- Primary button trigger label (when not showing copied state).
- Dropdown menu Item 1 label and secondary metadata badge.

```tsx
// Before:
<span>{isCopiedAllActive ? 'Copied!' : 'Copy All / Includes Image'}</span>

// After:
<span>{isCopiedAllActive ? 'Copied!' : 'Copy All (Include Images)'}</span>
```

#### 2.1.2 Multi-MIME Composition Invariant
In `src/components/instances/PromptTreeView/clipboard.ts`, `copyPromptWithRichImages` ensures triple-MIME clipboard item writes:
1. `image/png`: True binary PNG blob extracted from the first image attachment or canvas-transcoded from Base64/SVG/WebP to prevent raw byte collisions.
2. `text/html`: Full rich HTML document containing inline Base64 `<img>` tags, styled `<pre><code>` blocks, and paragraph wrappers for Microsoft OneNote and Word.
3. `text/plain`: Clean UTF-8 markdown text with relative or data URI links for developer IDEs and text editors.

Fallback ladder:
```text
Triple MIME ('image/png', 'text/html', 'text/plain')
  ↳ Failure: Dual MIME ('text/html', 'text/plain')
      ↳ Failure: Text Clipboard API (navigator.clipboard.writeText)
          ↳ Failure: execCommand('copy') textarea bridge
```

---

### 2.2 Details Button Restoration & Relocation

#### 2.2.1 Problem Statement
In `DetailHeader.tsx`, the Details button (`FileText` icon + `"Details"`) was nested inside the prompt action split button group alongside rerun counters and suffix dropdowns. This caused toolbar overcrowding, horizontal scrollbar popping on medium displays, and visual collision with the new conversation input.

#### 2.2.2 Architectural Relocation
1. **Remove from `DetailHeader.tsx`**:
   - Delete Segment 4 (`{/* Segment 4: Details Button */}`) from `DetailHeader.tsx` lines 691–704.
   - Adjust preceding button borders to avoid trailing border gaps.
2. **Mount in `DetailTabs.tsx`**:
   - Destructure `setIsDetailsModalOpen` in `DetailTabs.tsx` props.
   - Insert adjacent to the View Mode capsule (`Preview`, `Raw`, `Edit`) on the left side of the secondary toolbar.
   - Wrap inside a cohesive segmented glass capsule with a pulsating health indicator dot:

```tsx
{/* Details Button with Pulsating Indicator */}
{setIsDetailsModalOpen && (
    <button
        type="button"
        onClick={() => setIsDetailsModalOpen(true)}
        className="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[11px] font-semibold text-slate-700 dark:text-slate-200 bg-slate-100/90 dark:bg-[#120a1f]/90 border border-slate-200 dark:border-[#40285e] hover:bg-slate-200/80 dark:hover:bg-[#40285e] transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer shadow-2xs"
        aria-label="Open Conversation & Instance Details"
        title="View Turn Metrics, Metadata & Runtime Details"
    >
        <span className="relative flex h-2 w-2 shrink-0">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-blue-400 opacity-75" />
            <span className="relative inline-flex rounded-full h-2 w-2 bg-blue-500" />
        </span>
        <FileText className="w-3.5 h-3.5 text-blue-500 shrink-0" />
        <span>Details</span>
    </button>
)}
```

---

### 2.3 "Start All / Open All Instances" Orchestration

#### 2.3.1 Backend Command Implementation
Add `launch_all_instances` to `src-tauri/src/commands/instance.rs`:

```rust
#[tauri::command]
pub fn launch_all_instances() -> Result<usize, crate::error::AppError> {
    let mut audit = crate::modules::task_history_db::AuditTask::start(
        crate::modules::audit_action::AuditAction::StartInstance,
        "Launch all stopped instances",
        None,
    );

    let instances = instance::list_instances()
        .map_err(|e| crate::error::AppError::Custom(e))?;

    let mut launched_count: usize = 0;
    for inst in instances {
        if !inst.is_running {
            match instance::launch_instance(&inst.config.id) {
                Ok(_) => {
                    launched_count += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        "[Instance] Failed to launch instance '{}': {}",
                        inst.config.id,
                        e
                    );
                }
            }
        }
    }

    audit.succeed(&format!("Launched {} stopped instance(s)", launched_count));
    Ok(launched_count)
}
```

#### 2.3.2 Tauri Registration & Frontend Service Binding
1. **Registration**: In `src-tauri/src/lib/run.rs`, append `commands::launch_all_instances` to the `.invoke_handler(tauri::generate_handler![...])` command list.
2. **Lifecycle Service**: In `src/services/instanceService/lifecycle.ts`, export `launchAllInstances`:

```typescript
export async function launchAllInstances(): Promise<number> {
    try {
        return await invoke<number>('launch_all_instances');
    } catch (e: any) {
        const captured = useErrorStore.getState().captureError(e, {
            source: 'instanceService.launchAllInstances',
            endpoint: 'launch_all_instances',
            triggerAction: 'launch_all_instances',
        });
        useErrorStore.getState().openErrorModal(captured);
        throw e;
    }
}
```

#### 2.3.3 UI Trigger in `InstancesHeader.tsx`
Add the `"Start All"` button into Segmented Group 1 of `src/pages/instances/InstancesHeader.tsx`, between `runningBadgeRef` and `Refresh`:

```tsx
{/* Start All / Open All Button with Badge */}
{instances.length > runningCount && (
    <button
        type="button"
        onClick={handleLaunchAll}
        disabled={isLoading || isLaunchingAll}
        className="flex items-center gap-1.5 px-3 py-1 text-xs font-semibold text-emerald-700 dark:text-emerald-400 hover:bg-emerald-100/50 dark:hover:bg-emerald-950/40 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
        title={`Launch all ${instances.length - runningCount} stopped Antigravity instance(s)`}
    >
        <Play className={cn('w-3.5 h-3.5 text-emerald-500', isLaunchingAll && 'animate-spin')} />
        <span>Start All</span>
        <span className="px-1.5 py-0.2 rounded-full text-[10px] font-mono font-bold bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30">
            {instances.length - runningCount}
        </span>
    </button>
)}
```

---

### 2.4 SSH Settings UI Text-to-Icon Modernization

#### 2.4.1 Problem Statement
In `src/components/settings/EmailNotifications/SshGitmapSection.tsx`, the action column for remote nodes renders wide text buttons (`"Test Connection"`, `"Prompt Machine"`, `"Delete"`), forcing the table to overflow horizontally on smaller desktop screens.

#### 2.4.2 Icon Button Modernization
Replace textual labels with compact icon-only buttons with accessible `aria-label` attributes and tooltip `title` properties:

```tsx
<div className="inline-flex items-center rounded-full border border-gray-200 dark:border-slate-700 bg-white dark:bg-slate-900 p-0.5 shadow-2xs divide-x divide-gray-200 dark:divide-slate-700">
    {/* 1. Test Connection */}
    <button
        type="button"
        onClick={() => handleTestNode(node.alias)}
        disabled={testingNodeAlias === node.alias}
        className="p-1.5 text-gray-700 dark:text-slate-200 hover:bg-gray-100 dark:hover:bg-slate-800 rounded-l-full flex items-center justify-center cursor-pointer disabled:opacity-50 transition-colors"
        title="Test SSH connectivity"
        aria-label={`Test SSH connectivity to ${node.alias}`}
    >
        {testingNodeAlias === node.alias ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin text-emerald-500" />
        ) : (
            <Play className="w-3.5 h-3.5 text-emerald-500" />
        )}
    </button>

    {/* 2. Prompt Machine */}
    <button
        type="button"
        onClick={() => {
            setTargetMachine(node.alias);
            const matching = clusterBalances.filter(
                (item) => item.machine_alias.toLowerCase() === node.alias.toLowerCase()
            );
            const cleanInst = matching.length > 0 ? (matching[0].instance_id || 'default') : 'default';
            setTargetInstance(cleanInst);
            showToast(`Selected ${node.alias}:${cleanInst} for prompting`, 'info');
            const promptInput = document.getElementById('remote-prompt-input');
            if (promptInput) {
                promptInput.scrollIntoView({ behavior: 'smooth', block: 'center' });
                promptInput.focus();
            }
        }}
        className="p-1.5 text-sky-600 dark:text-sky-400 hover:bg-sky-50 dark:hover:bg-sky-950/40 flex items-center justify-center cursor-pointer transition-colors"
        title="Target this machine in the Prompt Dispatcher below"
        aria-label={`Target ${node.alias} in Prompt Dispatcher`}
    >
        <Send className="w-3.5 h-3.5" />
    </button>

    {/* 3. Delete */}
    <button
        type="button"
        onClick={() => handleDeleteNode(node.alias)}
        disabled={deletingNodeAlias === node.alias}
        className="p-1.5 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 rounded-r-full flex items-center justify-center cursor-pointer disabled:opacity-50 transition-colors"
        title="Delete SSH node"
        aria-label={`Delete SSH node ${node.alias}`}
    >
        {deletingNodeAlias === node.alias ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin text-rose-500" />
        ) : (
            <Trash2 className="w-3.5 h-3.5" />
        )}
    </button>
</div>
```

---

### 2.5 Markdown Default View Background Color Standardization

Standardize the dark theme background color of markdown containers across the 3 primary display components to `dark:bg-[#19191c]`:

| Component Path | Element | Existing Dark Class | Standardized Dark Class |
|---|---|---|---|
| `src/components/instances/PromptTreeView/PromptInstructionView.tsx` | Preview Container (L856) | `dark:bg-[#0c0514]/95` | `dark:bg-[#19191c]` |
| `src/components/instances/PromptTreeView/PromptInstructionView.tsx` | Raw Monospace Container (L905) | `dark:bg-[#0c0514]/95` | `dark:bg-[#19191c]` |
| `src/components/instances/PromptTreeView/InspectorModal.tsx` | Markdown Preview Card (L459) | `dark:bg-[#100a1c]` | `dark:bg-[#19191c]` |
| `src/components/instances/PromptTreeView/ResultsView.tsx` | Output Markdown Container (L198) | `dark:bg-[#090611]` | `dark:bg-[#19191c]` |

This eliminates distracting purple and black luminance shifts between the prompt view tabs, modal inspector, and execution output blocks.

---

## 3. Backward Compatibility & System Invariants

All changes in this specification adhere strictly to AGM architectural invariants:

### 3.1 Strict Relative Git Paths
- No absolute paths (`C:\...`, `/home/...`, `file:///...`) are stored in configurations, databases, or UI labels.
- All file targets are strictly repository-relative (`src/...`, `src-tauri/...`, `02-spec/...`).

### 3.2 Positive Boolean Field Naming
- State variables must maintain positive boolean prefixes:
  - `isLaunchingAll: boolean` (NOT `launching_all_disabled`)
  - `isCopiedAllActive: boolean` (NOT `unCopied`)
  - `hasImages: boolean` (NOT `noImages`)

### 3.3 Zero Silent Errors
- In Rust (`src-tauri/src/commands/instance.rs`), any error from `instance::launch_instance` is tracked via `tracing::warn!` and reflected in `AuditTask`. Fallible I/O must never use bare `let _ = ...` without justification.
- In TypeScript (`lifecycle.ts`, `InstancesHeader.tsx`), all caught exceptions must route through `useErrorStore.getState().captureError` or `trackWarning`.

### 3.4 Job Breakaway Fallback
- `launch_instance` internal process spawning must preserve the Windows Job Object breakaway fallback ladder (`spawn_with_job_breakaway_fallback`).

---

## 4. Verification Matrix

| Req # | Feature / Area | Target Files | Expected Behavior | Verification Procedure |
|---|---|---|---|---|
| **REQ-01** | Copy All Label Standardization | `DetailHeader.tsx`<br>`clipboard.ts` | Split button renders `"Copy All (Include Images)"`. Triple MIME item written to clipboard with PNG transcoding. | 1. Open prompt with image.<br>2. Verify button label matches exact text.<br>3. Click button and paste into VS Code (gets clean markdown) and OneNote (gets image + HTML). |
| **REQ-02** | Details Button Restoration | `DetailHeader.tsx`<br>`DetailTabs.tsx` | Details button removed from header row; mounted in `DetailTabs` beside view modes with pulsating blue health dot. | 1. Open prompt view; confirm top header row has no Details button.<br>2. Confirm Details button renders beside Preview/Raw/Edit with pulsating dot.<br>3. Click button; verify Details modal opens. |
| **REQ-03** | Start All Instances Button | `commands/instance.rs`<br>`run.rs`<br>`lifecycle.ts`<br>`InstancesHeader.tsx` | `launch_all_instances` command launches all stopped instances, records audit task, and button displays stopped count badge. | 1. Stop 2 instances.<br>2. Verify "Start All" button appears with badge "2".<br>3. Click "Start All"; verify both instances transition to running and badge disappears. |
| **REQ-04** | SSH UI Text-to-Icon Modernization | `SshGitmapSection.tsx` | Text buttons replaced with compact Play, Send, Trash2 icon buttons with hover tooltips in a segmented capsule. | 1. Navigate to Settings -> Email & Notifications -> SSH Gitmap section.<br>2. Inspect remote machine table.<br>3. Verify compact icon capsule renders with hover tooltips and action handlers function correctly. |
| **REQ-05** | Markdown Background Color Standardization | `PromptInstructionView.tsx`<br>`InspectorModal.tsx`<br>`ResultsView.tsx` | Dark mode markdown viewing containers render with background `#19191c`. | 1. Toggle dark theme.<br>2. Inspect DOM background class for PromptInstructionView, InspectorModal, and ResultsView.<br>3. Verify all 3 use `dark:bg-[#19191c]`. |

---

## 5. Pre-flight Testing & Verification Commands

Execute the following commands in the workspace root to ensure code quality and prevent CI/CD failures:

```powershell
# 1. Frontend type-check & production build
npm run build

# 2. Rust syntax formatting gate
cd src-tauri
cargo fmt -- --check

# 3. Rust Clippy gate (covers compilation and lint checks)
cargo clippy --all-targets --all-features
cd ..

# 4. Verification of command registry
# Confirm launch_all_instances is properly mapped in run.rs
```

---

*Maintained by @aukgit*
