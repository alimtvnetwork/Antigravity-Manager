# Specification: 115 — Instance Cloning, Security Presets, Deep Sync & Card UI Polish

## 1. Overview & Problem Definition

In user testing and visual telemetry (Plan 115):
1. **Incomplete Cloning Bug**: Cloning an instance / account / EXE duplicates the directory structure but drops `settings.json`, `workspaceStorage` projects, color themes, and Antigravity security presets.
2. **Broken Modal UI**: The "Instance Settings & Deep Sync" modal (`InstanceSettingsModal.tsx`) suffers from layout breakage where buttons (`Copy Now`, `Copy Folders`) overlap dropdowns and trigger an ugly horizontal scrollbar. It uses excessive wordy text instead of compact, high-precision icons.
3. **Native OS Delete Dialog**: Deleting an instance opens a native Windows message box instead of a seamless in-app Dark/Glass UI modal.
4. **No Card Progress on Actions**: Clicking Play (launch) or Stop does not disable the card or row, allowing duplicate clicks without visible progress feedback.

---

## 2. Technical Architecture & Non-Negotiables

### 2.1 Backend Cloning & Deep Sync (`src-tauri/src/modules/instance.rs`)
- When copying an instance with `copy_instance_with_options`:
  - **Settings & Themes**: Copy `User/settings.json`, `User/keybindings.json`, `User/globalStorage/state.vscdb` (theme and workspace preferences).
  - **Workspaces & Projects**: Recursively copy `User/workspaceStorage/` so all projects opened in the source profile are immediately available in the clone.
  - **Security Presets**: Copy Antigravity security presets (`User/security_presets.json` or `.gemini/policies` / `antigravity_policies.json`).
  - **CLI / Headless Parity**: Ensure identical execution whether invoked via GUI IPC or CLI `agm instance copy`.

### 2.2 Redesigned "Instance Settings & Deep Sync" Modal (`InstanceSettingsModal.tsx`)
- Eliminate horizontal scrolling (`overflow-x-hidden`).
- Refactor the two action cards into clean flex-col/grid containers where dropdowns and action buttons sit neatly stacked or in responsive button rows with proper spacing (`gap-2`).
- Replace verbose button text with recognizable Lucide icons (`Copy`, `Palette`, `FolderSync`, `ClipboardCopy`, `Download`, `Upload`) and concise labels.

### 2.3 Bespoke In-App Delete Confirmation Modal (`InstanceDeleteModal.tsx`)
- Replace Tauri native dialog `ask()` / `confirm()` with a custom in-app modal matching the Antigravity glass aesthetic.
- Displays target instance name, sequence number, email, and explicit confirmation button with red glowing styling.

### 2.4 Interactive Card Action Progress & Locking (`src/pages/Instances.tsx`)
- Add `actionState: Record<string, 'launching' | 'stopping' | 'switching' | null>`.
- When an action is initiated, set the state, disable all interactive buttons on that card/row (`disabled={Boolean(actionState[inst.config.id])}` or `pointer-events-none opacity-70`), and show a subtle centered progress badge or spinner.
- Clear action state once the IPC promise resolves or rejects.
