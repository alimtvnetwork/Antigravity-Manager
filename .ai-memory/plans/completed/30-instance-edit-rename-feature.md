# Plan 30: Instance Profile Edit and Rename Feature

> **Plan Path:** `.ai-memory/plans/completed/30-instance-edit-rename-feature.md`
> **Status:** Completed
> **Task Origin & Inception:** User requested "Add edit option for the instance plase" with screenshot showing duplicated profiles (`Default Copy`, `Default Copy Copy`) in the Instance dropdown menu without any method to rename or edit their display names.
> **Target Release:** v4.26.0

---

## 1. Problem Statement & Requirements

### A. Missing Rename / Edit for Instance Profiles
- When duplicating or creating instance profiles, profiles ended up with default duplicate names (e.g. `Default Copy`, `Default Copy Copy`).
- Users had no way to rename an instance profile without manually modifying `instances.json` on disk.
- Requirement: Provide an in-place rename/edit mechanism across the application:
  1. **Top Navbar Quick Action:** Pencil button next to Clone and Create in `InstanceSelector.tsx` to instantly rename the currently active profile.
  2. **Selector Dropdown List:** Inline pencil button on each instance entry row so any profile can be renamed quickly.
  3. **Instances Management Page:** Pencil button on each profile card in `Instances.tsx` alongside Launch, Duplicate, Clean, and Delete.
  4. **Backend IPC & Storage:** `rename_instance` Tauri IPC command updating `instances.json` safely without disturbing credentials, settings, or workspace data.
  5. **Localization:** Trilingual support in English (`en`), Simplified Chinese (`zh`), and Traditional Chinese (`zh-TW`).

---

## 2. Key Implementations & Enhancements

### A. Backend Rust Module & IPC Command (`src-tauri/`)
- `src-tauri/src/modules/instance.rs`:
  - Implemented `rename_instance(instance_id: &str, new_name: String) -> Result<InstanceConfig, String>`.
  - Validates non-empty input, locates the target profile in the registry, updates its `name` attribute, writes `instances.json`, and returns the updated `InstanceConfig`.
- `src-tauri/src/commands/instance.rs`:
  - Exposed `#[tauri::command] pub fn rename_instance(instance_id: String, new_name: String) -> Result<InstanceConfig, String>`.
- `src-tauri/src/lib.rs`:
  - Registered `commands::rename_instance` in the Tauri IPC handler list.

### B. Frontend Service & Zustand Store (`src/`)
- `src/services/instanceService.ts`:
  - Added `renameInstance(instanceId: string, newName: string): Promise<InstanceConfig>`.
- `src/stores/useInstanceStore.ts`:
  - Added `renameInstance` action to the `InstanceState` interface and store implementation with loading states and automated registry refresh.

### C. Top Navigation Dropdown & Inline Actions (`src/components/navbar/InstanceSelector.tsx`)
- Added quick edit pencil icon button on the navbar next to Clone.
- Restructured profile items in the dropdown menu into a flex row separating profile selection from inline row actions.
- Added inline pencil button to each profile item with `e.stopPropagation()` to prevent unwanted switching when renaming.
- Built accessible Edit Modal dialog with auto-focus input and Enter key submission.

### D. Instances Management Page (`src/pages/Instances.tsx`)
- Added edit pencil button on each profile card's action row.
- Built Edit Modal dialog with input validation, error handling, and direct state synchronization.

### E. Localization (`src/locales/`)
- Added `instances.edit_modal_title`, `instances.edit_modal_desc`, `instances.edit_current`, `instances.edit_title`, and `instances.edit_placeholder` across:
  - `src/locales/en.json`
  - `src/locales/zh.json`
  - `src/locales/zh-TW.json`

---

## 3. Verification & Deliverables

- [x] Backend `rename_instance` implemented and registered in Tauri IPC.
- [x] Frontend service and Zustand store updated.
- [x] Navbar quick edit button and dropdown inline rename actions implemented.
- [x] Instances page card edit button and modal implemented.
- [x] Trilingual localization keys added.
- [x] All quality gates and repository files verified clean.
