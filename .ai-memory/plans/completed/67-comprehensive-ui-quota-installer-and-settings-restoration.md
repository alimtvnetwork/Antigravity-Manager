# Plan 67: Comprehensive UI, Quota Calculation, Installer, and Settings Restoration

> Traceability: Resolves regressions and polish items from upstream PR #4 merge (`634c9fad`)
> Status: COMPLETED
> Spec Reference: [02-spec/21-app/23-comprehensive-ui-quota-installer-and-settings-restoration.md](../../../02-spec/21-app/23-comprehensive-ui-quota-installer-and-settings-restoration.md)

## Subtasks Execution Summary

### Subtask 01: Quota 5H Rolling Bucket & Accounts Table Buttons
- **Status**: Completed & Verified
- **Files Modified**: `src-tauri/src/modules/quota.rs`, `src/pages/Accounts.tsx`, `src/components/accounts/AccountTable.tsx`
- **Result**: Restored canonical 5H rolling bucket calculation in `quota.rs` so that rolling hours (`4h 56m`) are displayed rather than weekly exhausted resets (`6d 9h`). Debounced quota refreshes and ensured Switch IDE / CLI buttons remain visible across screen widths.

### Subtask 02: App, Process & Shortcut Branding Alignment
- **Status**: Completed & Verified
- **Files Modified**: `src-tauri/tauri.conf.json`, `src-tauri/hooks.nsh`, `install.ps1`
- **Result**: Set `productName: "Antigravity Manager Tools"` and `mainBinaryName: "agm-alim"`. Process title displays descriptive name in Windows Task Manager, Start Menu creates `Antigravity Manager Tools.lnk` without version numbers, and Windows Add/Remove programs displays `Antigravity Manager Tools <version>`.

### Subtask 03: Instances Sequential Numbering & Quick Creation
- **Status**: Completed & Verified
- **Files Modified**: `src/pages/Instances.tsx`, `src/components/navbar/InstanceSelector.tsx`
- **Result**: Added sequential badges (`#1`, `#2`, `#3`...) to instances cards and selector items. Enlarged card padding and dimensions for enhanced readability. Added "+ New Instance" button next to search filter.

### Subtask 04: Email Export Polish, PS Prefix & Prompt Injection Fix
- **Status**: Completed & Verified
- **Files Modified**: `src/components/settings/EmailNotificationSettings.tsx`, `src/components/settings/MailboxExportModal.tsx`, `src-tauri/src/commands/email.rs`, `src-tauri/src/modules/email_inbound.rs`, `src/components/settings/ai-sample-templates-modal.tsx`
- **Result**: Added prefix normalizer stripping `powershell:`, `ps:`, `ps `, `pwsh:`, `bash:`, `sh:`, `cmd:`. Intercepted AI prompt simulation payloads (`Project: ...`) preventing shell crashes. Modernized export modal and updated footer to "Antigravity Manager Tools".

### Subtask 05: Telegram Guide & Settings Unified Backup/Restore
- **Status**: Completed & Verified
- **Files Modified**: `src/components/settings/SupabaseSyncSettings.tsx`, `src/components/settings/AutoSwitcherSettings.tsx`, `src/pages/Settings.tsx`, `src/components/navbar/NavSettings.tsx`
- **Result**: Aligned Telegram Token and Chat ID input heights. Added step-by-step Telegram Bot Setup Guide modal. Added Actions dropdown (export, import, reset) to Auto-Switcher card. Connected encrypted full system Backup & Restore button in Settings toolbar and SupabaseSyncSettings. Added Debug console toggle button to navbar settings.

### Subtask 06: Root README Overhaul & Release Page Installer Split Blocks
- **Status**: Completed & Verified
- **Files Modified**: `readme.md`, `README_EN.md`, `03-ai-scripts/29-release-orchestrator.py`, `.github/workflows/release.yml`, `docs/images/dashboard-modern.png`
- **Result**: Replaced obsolete v2.0.0 screenshot with modern English unified dashboard. Separated Direct Latest and Pinned Version one-liner commands into distinct code blocks with verified PowerShell scriptblock syntax. Removed unversioned `agm-alim-setup.exe` copy in release workflow ensuring all assets carry explicit version numbers.
