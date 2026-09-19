# Plan Completed: UI Error Manage, Top Padding, Accounts Actions Reorder, and Instance Smart Double-Play

## Execution Summary
- **Origin / Start Details:** Prompt Version 2.2.0, Parent Task N-Step Continuous Loop (N=250). Initiated from user request to reduce top padding, fix window minimize/restore blank canvas issue, implement error management UI with left-side badge and raw stack trace fallbacks, reorder Accounts table action buttons (Refresh 1st, Switch/Terminal 2nd), set default tab to Accounts, add instance profile Double Play button with pre-calculated rotation candidates, and fix the Windows taskbar icon.
- **Total Execution Steps / Loops:** 12 steps across 5 subtasks.
- **Status:** COMPLETED
- **Target Version:** v4.28.1
- **Completion Date:** 2026-09-19

---

## Deliverables & Completed Work

### 1. Top Padding Reduction & Default Landing Tab (Subtask 01)
- `src/App.tsx`: Updated root router to redirect `/` to `/accounts` by default on app launch. Added `/dashboard` route.
- `src/components/navbar/Navbar.tsx`: Positioned Accounts navigation item first. Reduced top drag region padding from `pt-9` to `pt-3`.
- `src/components/layout/Layout.tsx`: Reduced drag strip height from `h-9` to `h-4`.
- Global page top padding compacted to `px-4 sm:px-6 pt-2 pb-4` across `Accounts.tsx`, `Instances.tsx`, `Dashboard.tsx`, `Settings.tsx`, `TokenStats.tsx`, `Security.tsx`, `Monitor.tsx`, `UserToken.tsx`, `ApiKeyFun.tsx`, and `ApiProxy.tsx`.

### 2. Accounts Table Action Icons Reordering (Subtask 02)
- Per `media_1789821101716.png`:
  - Position 1: Refresh action button (`RefreshCw`).
  - Position 2: Switch & Terminal action button group (`ArrowRightLeft` dropdown).
  - Position 3+: Remaining secondary actions (Info, Fingerprint, Download, Toggle, Delete).
- Updated in both `src/components/accounts/AccountRow.tsx` (table view) and `src/components/accounts/AccountCard.tsx` (card view).

### 3. Universal Error Management UI, Stack Trace Viewer & History Drawer (Subtask 03)
- `src/components/errors/error-queue-badge.tsx`: Left-side indicator badge beside the navbar brand logo showing real-time error badge count with timer debouncing (single-click opens recent error history drawer; double-click opens error inspection modal directly on the Stack Trace tab).
- `src/components/errors/error-history-drawer.tsx`: Slide-over drawer displaying recent error events, timestamps, error codes, and quick inspection trigger opening directly to stack trace.
- `src/components/errors/error-modal.tsx`: Refactored stack trace and backend tabs to fall back gracefully to formatted raw stack traces and diagnostic details. Defaults directly to the Stack Trace tab on open, and includes a quick navigation button from the Overview tab. Added "Copy Stack" button.
- `src/components/debug/DebugConsole.tsx`: Double-clicking any log row captures it into `useErrorStore` and opens `ErrorModal` directly to the stack trace tab.

### 4. Instance Profile Double Play Button & Smart Profile Rotation (Subtask 04)
- `src/services/instanceService.ts`: Added `findBestRotationProfile` algorithm that prioritizes least-recently-used idle profiles (e.g. 15h ago > 30m ago) in batches of 3, verifying that candidates are not currently running. Added `formatTimeAgo` helper.
- `src/stores/useInstanceStore.ts`: Added `rotateToNextBestProfile` action that finds the best candidate, sets active profile, updates `last_used` timestamp in SQLite storage, and launches the instance.
- `src/components/navbar/InstanceSelector.tsx`: Added Double Play (`FastForward`) button with pre-calculated hover candidate tooltip in the top action bar and in each row of the dropdown menu. Moved Edit/Rename, Duplicate, and Delete icons next to the instance selector.
- `src/pages/Instances.tsx`: Added top action icons (Edit, Duplicate, Delete) right beside the instance profile title in the card header. Added Double Play button with hover candidate tooltip to the card actions bar.

### 5. Window Minimize / Restore Blank Canvas Fix & Taskbar Icon (Subtask 05)
- `src-tauri/tauri.conf.json`: Changed `"transparent": true` to `"transparent": false`, resolving WebView2 DirectComposition surface discard lock on Windows taskbar minimize/restore.
- Win32 Unminimize enforcement: Added `window.unminimize()` before `window.show()` across all call sites in `src-tauri/src/modules/tray.rs`, `src-tauri/src/lib.rs` (single-instance and reopen handlers), and `src-tauri/src/commands/mod.rs` (`show_main_window`).
- `src-tauri/build.rs`: Updated Windows resource script `comctl6.rc` to embed `icons/icon.ico` as resource ID 1 and IDI_APPLICATION (32512) alongside Common Controls v6 manifest, restoring Windows Explorer and taskbar executable icon linking. Formatted cleanly with `cargo fmt`.
- `src-tauri/src/lib.rs` & `src-tauri/src/commands/mod.rs`: Explicitly load embedded `icons/icon.png` and invoke `window.set_icon(...)` during application setup and in `show_main_window` to ensure Win32 `WM_SETICON` assigns the custom AGM icon to the taskbar and Alt-Tab switcher. Added `window-restored` event emission on focus.
- `src/components/layout/Layout.tsx`: Added window focus and `visibilitychange` listeners to dispatch synthetic `resize` events when unminimized, forcing WebView2 to re-render cleanly. Fixed mixed-polarity boolean conditions.

---

## Verification
- TypeScript typecheck passed cleanly (`python 03-ai-scripts/34-typescript-checker.py`).
- Rust code formatting validated cleanly (`cargo fmt --check`).
- Line endings normalized and validated cleanly (`04-newline-fixer.py`).
- Strict coding guidelines verified: strictly lowercase filenames, relative git paths, implicit booleans only.
