# Subtask Plan: Instance Prompt Bleeding, Standalone JSON v2.0 & Updater Detection

- **Subtask Slug**: `ui-and-functionality-enhancement-instructions`
- **Subtask Plan File**: `.ai-memory/plans/subtasks/ui-and-functionality-enhancement-instructions/03-instance-prompt-and-updater.md`
- **Parent Plan**: `.ai-memory/plans/ui-and-functionality-enhancement-instructions.md`
- **Spec Reference**: `02-spec/21-app/ui-and-functionality-enhancement-instructions/03-migration-and-verification-spec.md`
- **Status**: READY FOR IMPLEMENTATION
- **Assigned Worker**: Worker 03 / Spec Author 03
- **Coverage**: Task-04 (Instance Prompt Tracking & PID Isolation), Task-06 (Standalone JSON Import/Export & Overlay Z-Index), Task-07 (Pinned Version Script & Update Version Detection RCA)

---

## 1. Overview & High-Level Objectives

This subtask plan operationalizes the architectural and migration specifications defined in `02-spec/21-app/ui-and-functionality-enhancement-instructions/03-migration-and-verification-spec.md`:

1. **Eliminate False Liveness & Cross-Instance Prompt Bleed**:
   - Refactor `liveness.rs` and `tree_collect.rs` to enforce Explicit Idle Supremacy over unreset `not_fully_idle` database counters.
   - Refactor `tree_phases.rs::query_tree_projects` so that passing `target: None` sweeps all registered instances instead of only `"default"`.
   - Update `instanceCardData.ts` to filter active running tasks by genuine conversation ownership (`c.instance_id === inst.config.id`), eliminating duplicate prompt display on shared repository roots.
2. **Stabilize Modal Overlays & Standalone JSON v2.0**:
   - Elevate secondary modal stacking to `z-[100005]` via React Portals (`createPortal(..., document.body)`) to prevent clipping and focus-trapping inside `PromptTreeViewModal` (`z-[99999]`).
   - Implement the Two-Tier JSON Envelope v2.0 schema with embedded base64 multimodal assets and relative `${workDir}` path normalization for portable cross-machine export/import.
3. **Resolve Updater Version Detection RCA & Enforce Pinned Release Invariant**:
   - Patch `install.ps1::Sync-InstalledAppsRegistryBranding` to write `DisplayVersion` to Windows Registry uninstall keys, unfreezing version numbers beyond `v4.190`.
   - Reorder `install.ps1::Get-InstalledVersion` to inspect live running process and disk PE executable `ProductVersion` before checking registry keys.
   - Lockdown `install.ps1`'s multi-version download queue when `-Version` is specified, strictly prohibiting drift to unrequested releases.

---

## 2. Step-by-Step Implementation Roadmap

```
+---------------------------------------------------------------------------------------------------+
|                                  IMPLEMENTATION SEQUENCE                                          |
+---------------------------------------------------------------------------------------------------+
| Task 3.1: Backend liveness.rs & tree_collect.rs Idle Supremacy Refactor                           |
| Task 3.2: Backend tree_phases.rs Multi-Instance Project Sweep                                     |
| Task 3.3: Frontend instanceCardData.ts Conversation-Level Ownership Gate                          |
| Task 3.4: Modal Z-Index Elevation (z-[100005]) & React Portal Mounting                            |
| Task 3.5: Standalone JSON Envelope v2.0 Multimodal Serialization & Re-hydration                   |
| Task 3.6: install.ps1 Registry DisplayVersion Sync & PE Header Primacy                            |
| Task 3.7: install.ps1 Pinned Version Queue Lockdown & Automated E2E Verification                  |
+---------------------------------------------------------------------------------------------------+
```

---

### Task 3.1: Backend `liveness.rs` and `tree_collect.rs` Idle Supremacy Refactor

#### Target Files
- `src-tauri/src/modules/repo_db/liveness.rs`
- `src-tauri/src/modules/repo_db/tree_collect.rs`

#### Detailed Changes
1. In `src-tauri/src/modules/repo_db/liveness.rs` (Lines 278–291):
   - Replace the legacy "not_fully_idle supremacy rule" with the "Explicit Idle Supremacy rule".
   - When `status.contains("CASCADE_RUN_STATUS_IDLE") || status.contains("IDLE") || status.contains("COMPLETED") || status.contains("FAILED") || status.contains("CANCELLED")`, immediately mark the conversation turn as idle (`continue`), regardless of whether `not_fully_idle` is `1` or `0`.
   - Retain the adaptive 10-minute thinking window (600s) *only* for active non-idle statuses where `not_fully_idle > 0`.
2. In `src-tauri/src/modules/repo_db/tree_collect.rs` (Lines 167–189):
   - Invert the evaluation ladder: check `is_explicit_idle` *before* evaluating `not_fully_idle > 0`.
   - Compute `has_idle_status = status.contains("IDLE") || has_terminal_status`.
   - If `has_idle_status` is true, immediately assign `is_conv_running = false`.
   - Only assign `is_conv_running = true` if `!has_idle_status && is_owning_inst_alive && not_fully_idle > 0 && is_recent`.

#### Code Snippet (`tree_collect.rs`)
```rust
let has_terminal_status = status.contains("COMPLETED")
    || status.contains("FAILED")
    || status.contains("CANCELLED");
let has_idle_status = status.contains("IDLE") || has_terminal_status;

let inspection = inspect_conversation_transcript(base, &cid);

let is_conv_running = if has_idle_status {
    // Explicit Idle Supremacy: unreset not_fully_idle artifacts must not override terminal status
    false
} else if is_owning_inst_alive && not_fully_idle > 0 && is_recent {
    true
} else if is_owning_inst_alive
    && inspection.is_recent_active
    && !inspection.is_terminal_done
{
    true
} else {
    false
};
```

---

### Task 3.2: Backend `tree_phases.rs` Multi-Instance Project Sweep

#### Target Files
- `src-tauri/src/modules/repo_db/tree_phases.rs`

#### Detailed Changes
1. Inspect `query_tree_projects(target: Option<&str>)` (Lines 31–34).
2. If `target` is `Some(inst_id)`, invoke `detect_running_projects(inst_id)`.
3. If `target` is `None`:
   - Load the active instance registry via `crate::modules::instance::load_registry()`.
   - Iterate through `reg.instances` and execute `detect_running_projects(&inst.id)` for each instance.
   - If registry fails to load, fallback safely to `detect_running_projects("default")` with warning logged via `crate::error::record_ignored`.

#### Code Snippet (`tree_phases.rs`)
```rust
pub(crate) fn query_tree_projects(target: Option<&str>) -> Vec<RunningProject> {
    if let Some(target_id) = target {
        let _ = super::detection::detect_running_projects(target_id);
    } else {
        match crate::modules::instance::load_registry() {
            Ok(reg) => {
                for inst in &reg.instances {
                    let _ = super::detection::detect_running_projects(&inst.id);
                }
            }
            Err(e) => {
                crate::error::record_ignored(
                    Err::<(), _>(e),
                    "load registry during universal query_tree_projects sweep",
                );
                let _ = super::detection::detect_running_projects("default");
            }
        }
    }
    // Proceed to list_running_projects() filtering...
}
```

---

### Task 3.3: Frontend `instanceCardData.ts` Conversation-Level Ownership Gate

#### Target Files
- `src/pages/instances/instanceCardData.ts`
- `src/pages/instances/instancePageUtils.ts`

#### Detailed Changes
1. In `src/pages/instances/instancePageUtils.ts`:
   - Export helper `isConversationOwnedByInstance(convInstId: string | undefined | null, instConfig: { id: string; is_default?: boolean }): boolean`.
   - Normalize default aliases: `""`, `"default"`, `"__default__"`.
   - Strictly match non-default instance IDs: `convInstId === instConfig.id`.
2. In `src/pages/instances/instanceCardData.ts` (Lines 76–87):
   - Update `hasActiveTask` calculation:
   - Inside `(node.conversations || []).some(...)`, ensure `isConversationOwnedByInstance(c.instance_id, inst.config)` evaluates to `true`.
   - If a conversation turn belongs to Instance 2, Instance 1's card must ignore it even if both instances reference the same workspace root directory.

#### Code Snippet (`instanceCardData.ts`)
```typescript
const hasActiveTask =
    Boolean(inst.is_running) &&
    runningTreeNodes.some((node) => {
        const isInstanceMatch = isNodeOwnedByInstance(node, inst.config);
        const hasGenuineRunningConv = (node.conversations || []).some(
            (c) =>
                Boolean(c.is_running) &&
                isConversationOwnedByInstance(c.instance_id, inst.config) &&
                !isGhostConversation(c) &&
                !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
        );
        return isInstanceMatch && Boolean(node.is_running) && hasGenuineRunningConv;
    });
```

---

### Task 3.4: Modal Z-Index Elevation (`z-[100005]`) & React Portal Mounting

#### Target Files
- `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
- `src/components/instances/PromptTreeView/PromptJsonImportExportModal.tsx`
- `src/components/instances/PromptTreeView/PromptImagePreviewModal.tsx`
- `src/components/instances/PromptTreeView/DetailsModal.tsx`

#### Detailed Changes
1. Verify `PromptTreeViewModal.tsx` maintains base modal backdrop at `z-[99999]`.
2. For all nested child dialogs opened from Prompt Tree View:
   - Wrap the dialog root container with React's `createPortal(..., document.body)`.
   - Update outer backdrop container styling to `fixed inset-0 z-[100005] flex items-center justify-center p-4 bg-black/75 backdrop-blur-md overflow-y-auto`.
   - Ensure the inner modal card specifies `relative w-full max-w-2xl max-h-[90vh] flex flex-col overflow-hidden` with `overflow-y-auto` on scrollable body sections.
   - Attach click outside handlers (`onClick={onClose}`) on the backdrop and `onClick={(e) => e.stopPropagation()}` on the card container.
   - Bind `keydown` listener for `Escape` key dismissal.

---

### Task 3.5: Standalone JSON Envelope v2.0 Multimodal Serialization & Re-hydration

#### Target Files
- `src-tauri/src/modules/json_envelope.rs`
- `src-tauri/src/modules/repo_db/prompt_export.rs`
- `src-tauri/src/modules/repo_db/prompt_import.rs`
- `src/components/instances/PromptTreeView/usePromptTreeCoreActions.ts`

#### Detailed Changes
1. In `src-tauri/src/modules/json_envelope.rs`:
   - Register envelope schema identifier `"agm/prompt-tree-export"` (Version `"2.0"`).
   - Implement `export_standalone_prompt_tree(cid: &str, embed_assets: bool) -> Result<String, String>`:
     * When `embed_assets` is `true`: Read attached images from disk, encode them to `data:<mime>;base64,<payload>`, and populate `attachments[].base64_payload`.
     * Replace absolute directory paths with relative variable `${workDir}`.
2. In `src-tauri/src/modules/repo_db/prompt_import.rs`:
   - Ingest Two-Tier JSON Envelope v2.0.
   - Resolve `${workDir}` against the destination instance's active workspace or target directory.
   - Hydrate base64 data URIs into `.agm/cache/images/` or workspace storage, linking paths into `repo_prompts.db`.
3. In `usePromptTreeCoreActions.ts`:
   - Add toggle in the export UI: `Standalone (Embed Images & Portable Paths)` vs `Linked (Relative References)`.
   - Provide copy to clipboard and file download triggers with user notifications.

---

### Task 3.6: `install.ps1` Registry `DisplayVersion` Sync & PE Header Primacy

#### Target Files
- `install.ps1`

#### Detailed Changes
1. Patch `Sync-InstalledAppsRegistryBranding` (Lines 464–503):
   - Ensure `$currentVer` is derived from `$resolvedVer` or file PE version.
   - Within `Set-ItemProperty` loop for `agm-alim` and `Antigravity Manager Tools`:
     ```powershell
     if ($currentVer) {
         Set-ItemProperty -Path $fullPath -Name "DisplayVersion" -Value $currentVer -Force -ErrorAction SilentlyContinue
     }
     ```
2. Overhaul `Get-InstalledVersion` (Lines 927–990):
   - Stage 1: Inspect live running processes (`agm-alim`, `Antigravity Tools`) via `Get-Process` and extract `ProductVersion` from process binary path.
   - Stage 2: Inspect candidate binaries in `$InstallDir` via `[System.Diagnostics.FileVersionInfo]::GetVersionInfo`.
   - Stage 3: Inspect previous directories including `Programs\Antigravity Manager Tools`.
   - Stage 4: Read registry `DisplayVersion` as a last resort only if no on-disk binary exists.

---

### Task 3.7: `install.ps1` Pinned Version Queue Lockdown & Automated E2E Verification

#### Target Files
- `install.ps1`
- `scripts/test-prompt-liveness-and-updater-e2e.ps1`

#### Detailed Changes
1. In `install.ps1` (Lines 1545–1558):
   - When `$isPinned` is `true`, enforce `$versionQueue = @($cleanPinned)`.
   - Prohibit appending candidate versions or known fallbacks (`4.65.0`, etc.).
   - If download or verification fails, exit immediately with **Exit Code 1**; never fall through to existing unpinned binaries or latest release.
2. Author `scripts/test-prompt-liveness-and-updater-e2e.ps1`:
   - Gate 1: Assert `tree_collect.rs` evaluates `CASCADE_RUN_STATUS_IDLE` with `not_fully_idle = 1` as idle (`is_running = false`).
   - Gate 2: Assert `query_tree_projects(None)` detects running projects across multiple instances.
   - Gate 3: Assert `install.ps1 -CheckUpdate` returns current version matching active binary, not `4.190`.
   - Gate 4: Assert Windows Registry `DisplayVersion` updates to target release.
   - Gate 5: Assert `install.ps1 -Version "0.0.1-nonexistent"` exits with code 1 without drifting.

---

## 3. File Inventory & Modification Scope

| Relative File Path | Primary Subsystem | Modification Summary |
| :--- | :--- | :--- |
| `src-tauri/src/modules/repo_db/liveness.rs` | Backend Liveness | Invert idle supremacy; ignore unreset `not_fully_idle` when status is idle. |
| `src-tauri/src/modules/repo_db/tree_collect.rs` | Backend Tree Collect | Invert `is_conv_running` logic; enforce terminal/idle status supremacy. |
| `src-tauri/src/modules/repo_db/tree_phases.rs` | Backend Discovery | Universal multi-instance `detect_running_projects` loop when target is None. |
| `src/pages/instances/instanceCardData.ts` | Frontend Instances | Gate `hasActiveTask` by `c.instance_id === inst.config.id`. |
| `src/pages/instances/instancePageUtils.ts` | Frontend Utils | Add `isConversationOwnedByInstance` with default alias normalization. |
| `src/components/instances/PromptTreeView/PromptJsonImportExportModal.tsx` | Frontend Modals | Mount via `createPortal` at `z-[100005]` with `overflow-y-auto`. |
| `src/components/instances/PromptTreeView/PromptImagePreviewModal.tsx` | Frontend Modals | Mount via `createPortal` at `z-[100005]`. |
| `src-tauri/src/modules/json_envelope.rs` | Backend JSON Envelope | Standalone v2.0 schema with base64 image encoding and `${workDir}` mapping. |
| `install.ps1` | Installer & Updater | Add `DisplayVersion` to registry branding; PE ProductVersion inspection primacy. |
| `scripts/test-prompt-liveness-and-updater-e2e.ps1` | Quality & E2E Testing | Automated PowerShell test suite verifying idle supremacy and updater contracts. |

---

## 4. Verification & Pre-Flight Gate Commands

Run targeted local checks to ensure complete adherence to project invariants:

```powershell
# 1. Rust code formatting and linting
cd src-tauri
cargo fmt -- --check
cargo clippy --all-targets --all-features
cargo test --test per_instance_prompt_liveness_test

# 2. Frontend compilation and TypeScript checks
cd ..
npm run build

# 3. Automated PowerShell E2E verification
powershell -ExecutionPolicy Bypass -File scripts/test-prompt-liveness-and-updater-e2e.ps1
```

---

## 5. Acceptance Invariant Checklist

- [ ] `CASCADE_RUN_STATUS_IDLE` with `not_fully_idle = 1` evaluates to `is_running: false` across all database scans.
- [ ] Multi-instance setups with shared repository directories isolate running prompt indicators to the owning instance only.
- [ ] Opening JSON import/export modal renders completely over `PromptTreeViewModal` without backdrop clipping or buried inputs.
- [ ] Exported standalone JSON v2.0 files can be copied to another machine and imported with all attached images intact.
- [ ] Running `install.ps1 -CheckUpdate` or viewing the update modal displays the genuine installed app version, not `v4.190`.
- [ ] Windows Registry `DisplayVersion` in `HKCU` and `HKLM` matches the current installed release version.
- [ ] Pinned version installations strictly reject unrequested release fallbacks upon download failure.
- [ ] No absolute paths or `file:///` URIs exist in documentation, tests, or persistent records.
